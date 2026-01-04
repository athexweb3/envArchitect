use proc_macro::TokenStream;
use quote::quote;
use std::path::Path;
use syn::{parse_macro_input, ItemStruct};

// Macro logic implementation.
// Attempts to locate plugin configuration from various sources and injects
// the necessary WIT bindings and adapter code.

#[proc_macro_attribute]
pub fn plugin(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_struct = parse_macro_input!(input as ItemStruct);
    let struct_name = &input_struct.ident;

    // Configuration Priority Order:
    // 1. env.toml
    // 2. plugin.toml
    // 3. env.json
    // 4. plugin.json
    // 5. Cargo.toml [package.metadata.plugin]

    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
    let args_path = Path::new(&manifest_dir);

    struct ConfigSource {
        path: std::path::PathBuf,
        kind: ConfigKind,
    }

    enum ConfigKind {
        Toml,
        Json,
        Cargo,
    }

    let candidates = vec![
        ConfigSource {
            path: args_path.join("env.toml"),
            kind: ConfigKind::Toml,
        },
        ConfigSource {
            path: args_path.join("plugin.toml"),
            kind: ConfigKind::Toml,
        },
        ConfigSource {
            path: args_path.join("env.json"),
            kind: ConfigKind::Json,
        },
        ConfigSource {
            path: args_path.join("plugin.json"),
            kind: ConfigKind::Json,
        },
        ConfigSource {
            path: args_path.join("Cargo.toml"),
            kind: ConfigKind::Cargo,
        },
    ];

    let mut found_config = None;

    for candidate in &candidates {
        if candidate.path.exists() {
            match candidate.kind {
                ConfigKind::Toml => {
                    // Validate basic TOML structure (optional, but good for sanity)
                    if std::fs::read_to_string(&candidate.path).is_ok() {
                        found_config = Some(candidate);
                        break;
                    }
                }
                ConfigKind::Json => {
                    if let Ok(content) = std::fs::read_to_string(&candidate.path) {
                        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                            if json.get("project").is_some() {
                                found_config = Some(candidate);
                                break;
                            }
                        }
                    }
                }
                ConfigKind::Cargo => {
                    if let Ok(content) = std::fs::read_to_string(&candidate.path) {
                        if let Ok(value) = content.parse::<toml::Value>() {
                            if let Some(pkg) = value.get("package") {
                                if let Some(meta) = pkg.get("metadata") {
                                    if meta.get("plugin").is_some() {
                                        found_config = Some(candidate);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if found_config.is_none() {
        let msg = format!(
            "Missing plugin configuration.\n\
             Checked priority order:\n\
             1. env.toml\n\
             2. plugin.toml\n\
             3. env.json (must have 'project' section)\n\
             4. plugin.json (must have 'project' section)\n\
             5. Cargo.toml (must have [package.metadata.plugin])\n\
             Manifest Dir: {}",
            manifest_dir
        );
        return syn::Error::new_spanned(&input_struct, msg)
            .to_compile_error()
            .into();
    }

    // 2. Load WIT Definition at compile time (relative to this macro crate)
    let wit_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../wit/plugin.wit");
    let wit_content = std::fs::read_to_string(&wit_path)
        .unwrap_or_else(|e| panic!("Failed to read WIT file at {:?}: {}", wit_path, e));

    let chosen_path = found_config.unwrap().path.to_string_lossy().into_owned();
    let dependency_tracker = quote! {
        const _: &[u8] = include_bytes!(#chosen_path);
    };

    let expanded = quote! {
        #input_struct

        // Generate bindings from the central WIT file (injected as inline string)
        wit_bindgen::generate!({
            inline: #wit_content,
            world: "plugin",
        });

        // Adapter struct to bridge WIT exports to SDK traits
        struct __EnvArchitectAdapter;

        impl Guest for __EnvArchitectAdapter {
            fn validate(manifest_json: String) -> Vec<String> {
                use env_architect_sdk::prelude::*;
                let manifest: serde_json::Value = serde_json::from_str(&manifest_json).unwrap_or_default();
                let plugin = #struct_name::default();
                env_architect_sdk::futures::executor::block_on(
                    PluginHandler::validate(&plugin, &manifest)
                ).unwrap_or_else(|e| vec![e.to_string()])
            }

            fn resolve(context_json: String) -> Result<ResolutionOutput, String> {
                 use env_architect_sdk::prelude::*;

                 let context: ResolutionContext = serde_json::from_str(&context_json)
                    .map_err(|e| format!("Context Parse Error: {}", e))?;

                 // Inject allowed capabilities into thread-local scope
                 // We map the structured Capability enum to simple strings (e.g. "ui-interact")
                 let active_caps_strings: Vec<String> = context.allowed_capabilities.iter().map(|cap| {
                     use env_architect_sdk::contract::reexports::Capability;
                     match cap {
                         Capability::Network(_) => "network".to_string(),
                         Capability::FsRead(_) => "fs-read".to_string(),
                         Capability::FsWrite(_) => "fs-write".to_string(),
                         Capability::Device(_) => "device".to_string(),
                         Capability::UiInteract => "ui-interact".to_string(),
                         Capability::UiSecret => "ui-secret".to_string(),
                         Capability::ServiceControl(_) => "service-control".to_string(),
                         Capability::SysExec(_) => "sys-exec".to_string(),
                         Capability::EnvRead(_) => "env-read".to_string(),
                     }
                 }).collect();
                 env_architect_sdk::api::context::set_active_capabilities(active_caps_strings);

                 let plugin = #struct_name::default();

                 let (plan, state) = env_architect_sdk::futures::executor::block_on(
                     PluginHandler::resolve(&plugin, &context)
                 ).map_err(|e| e.to_string())?;

                 let plan_json = serde_json::to_string(&plan)
                    .map_err(|e| format!("Plan Serialization Error: {}", e))?;

                 Ok(ResolutionOutput {
                     plan_json,
                     state,
                 })
            }

            fn install(context: InstallationContext) -> Result<(), String> {
                use env_architect_sdk::prelude::*;
                let plan: InstallPlan = serde_json::from_str(&context.plan_json)
                    .map_err(|e| format!("Plan Parse Error: {}", e))?;

                let plugin = #struct_name::default();
                env_architect_sdk::futures::executor::block_on(
                    PluginHandler::install(&plugin, &plan, context.state)
                ).map_err(|e| e.to_string())
            }
        }

        export!(__EnvArchitectAdapter);

        // Track dependency ensuring rebuilds on config change
        #dependency_tracker
    };

    TokenStream::from(expanded)
}
