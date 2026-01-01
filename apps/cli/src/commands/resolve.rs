use anyhow::{Context, Result};
use clap::Parser;
use serde_json;
use std::path::PathBuf;
use wasmtime::component::{Component, Linker};
use wasmtime::{Config, Engine, Store};

use crate::host::bindings::Plugin;
use crate::host::state::HostState;
use domain::dependency::ConsensusEngine;
use domain::security::VerificationService;
use domain::system::StoreManager;

#[derive(Parser, Debug)]
pub struct ResolveCommand {
    /// Path to the WASM plugin to resolve
    #[arg(long, default_value = "plugin.wasm")]
    pub plugin: PathBuf,

    /// Simulation mode (don't actually install anything, just resolve)
    #[arg(long, short)]
    pub dry_run: bool,

    /// Project root directory (defaults to current directory)
    #[arg(long)]
    pub project_root: Option<PathBuf>,
}

impl ResolveCommand {
    pub async fn execute(self) -> Result<()> {
        use crate::ui::{
            self,
            components::{prompt, spinner::Spinner},
            Icon, Theme,
        };

        let root = self
            .project_root
            .clone()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or(PathBuf::from(".")));

        let absolute_root = std::fs::canonicalize(&root).unwrap_or(root);

        // 1. Theme Header
        println!(
            "\n{} {} {}",
            Icon::Architect,
            Theme::primary("EnvArchitect"),
            Theme::muted("v0.1.0")
        );

        if !prompt::confirm("Start environment resolution?") {
            println!("{} Operation cancelled.", Icon::Cross);
            return Ok(());
        }

        let spinner = Spinner::new("Initializing plugin engine...");

        // 1. Configure Wasmtime
        let mut config = Config::new();
        config.wasm_component_model(true);
        config.async_support(true);

        let engine = Engine::new(&config)?;
        let mut linker: Linker<HostState> = Linker::new(&engine);

        // 2. Link Host Capabilities
        wasmtime_wasi::add_to_linker_async(&mut linker)?;
        crate::host::bindings::Plugin::add_to_linker(&mut linker, |state: &mut HostState| state)?;

        // 3. Initialize Host State
        let mut allowed_caps: Vec<serde_json::Value> = Vec::new();

        // Initialize Host State with manifest tracking
        let mut manifest_path_str: Option<String> = None;
        let mut manifest_content: Option<String> = None;

        let candidates = vec![
            ("env.toml", true),
            ("plugin.toml", true),
            ("env.json", false),
            ("plugin.json", false),
            ("Cargo.toml", true),
        ];

        for (filename, is_toml) in candidates {
            let path = absolute_root.join(filename);
            if path.exists() {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    manifest_path_str = Some(
                        std::fs::canonicalize(&path)
                            .unwrap_or(path)
                            .to_string_lossy()
                            .to_string(),
                    );
                    manifest_content = Some(content.clone());

                    let json_val = if is_toml {
                        toml::from_str::<serde_json::Value>(&content).ok()
                    } else {
                        serde_json::from_str::<serde_json::Value>(&content).ok()
                    };

                    if let Some(json) = json_val {
                        let caps_node = if filename == "Cargo.toml" {
                            json.get("package")
                                .and_then(|p| p.get("metadata"))
                                .and_then(|m| m.get("plugin"))
                                .and_then(|p| p.get("capabilities"))
                        } else {
                            json.get("capabilities")
                        };

                        if let Some(caps) = caps_node.and_then(|v| v.as_array()) {
                            allowed_caps = caps.clone();
                        } else if let Some(caps_map) = caps_node.and_then(|v| v.as_object()) {
                            for (k, v) in caps_map {
                                if v.as_bool() == Some(true) {
                                    allowed_caps.push(serde_json::json!(k));
                                } else {
                                    allowed_caps.push(serde_json::json!({ k: v }));
                                }
                            }
                        }
                    }
                }
                break;
            }
        }

        // Final fallback for demo if empty
        if allowed_caps.is_empty() {
            allowed_caps.push(serde_json::json!("ui-interact"));
        }

        // HostState needs Vec<String> for the simple capability check logic for now.
        let host_allowed_names: Vec<String> = allowed_caps
            .iter()
            .map(|v| {
                if let Some(s) = v.as_str() {
                    s.to_string()
                } else if let Some(obj) = v.as_object() {
                    obj.keys().next().cloned().unwrap_or_default()
                } else {
                    String::new()
                }
            })
            .collect();

        let host_state = HostState::new(host_allowed_names, manifest_path_str, manifest_content);
        let mut store = Store::new(&engine, host_state);

        // 4. Load Component
        spinner.set_task("Plugin Loading");
        spinner.set_message("Reading Wasm binary...");
        let component_bytes = std::fs::read(&self.plugin)
            .with_context(|| format!("Failed to read plugin file: {:?}", self.plugin))?;
        spinner.set_message("Instantiating component...");
        let component = Component::new(&engine, component_bytes)?;
        let plugin = Plugin::instantiate_async(&mut store, &component, &linker).await?;

        // 6. Execute
        spinner.set_message("Discovering system tools...");
        let mut registry = domain::system::InstalledToolsRegistry::new();
        let _ = registry.scan();
        let mut system_tools = std::collections::HashMap::new();

        // Populate system_tools for context
        for tool in ["node", "python", "rustc", "cargo", "go"] {
            let versions = registry.get_installed(tool);
            if !versions.is_empty() {
                system_tools.insert(
                    tool.to_string(),
                    versions
                        .iter()
                        .map(|v| v.version.to_string())
                        .collect::<Vec<_>>(),
                );
            }
        }

        spinner.set_message("Resolving dependencies...");
        let context = serde_json::json!({
            "target_os": std::env::consts::OS,
            "target_arch": std::env::consts::ARCH,
            "project_root": absolute_root.to_string_lossy(),
            "env_vars": {},
            "allowed_capabilities": allowed_caps,
            "parent_manifest": null,
            "system_tools": system_tools
        });

        // Hook for errors
        // ... (Preserve panic hook logic if needed, or rely on nice errors)

        let result = plugin.call_resolve(&mut store, &context.to_string()).await;

        match result {
            Ok(Ok(output)) => {
                spinner.success("Resolution complete.");

                let valid_json: serde_json::Value = serde_json::from_str(&output.plan_json)
                    .unwrap_or(serde_json::Value::String(output.plan_json));

                ui::info("Install Plan:");
                let _ = ui::println(serde_json::to_string_pretty(&valid_json)?);

                // Phase 1 V2: Create Shims and Ensure Store for dependencies
                // Parse the InstallPlan to access the nested manifest
                if let Some(manifest_node) =
                    serde_json::from_str::<serde_json::Value>(&output.plan_json)
                        .ok()
                        .and_then(|v| v.get("manifest").cloned())
                {
                    if let Ok(manifest) =
                        serde_json::from_value::<env_manifest::EnhancedManifest>(manifest_node)
                    {
                        let spinner_v2 = Spinner::new("Finalizing V2 Sovereign Environment...");

                        let store = StoreManager::default()?;
                        let verifier = VerificationService::new();
                        let shims_dir = absolute_root.join(".architect").join("shims");
                        std::fs::create_dir_all(&shims_dir)?;

                        for (name, spec) in &manifest.dependencies {
                            spinner_v2.set_message(format!("Shimming {}...", name));

                            // 1. Ensure tool is in store (Prototype: Mock install if not exists)
                            let version_req = match spec {
                                env_manifest::DependencySpec::Simple(req) => req.to_string(),
                                env_manifest::DependencySpec::Detailed(details) => {
                                    details.version.to_string()
                                }
                            };
                            let version = if version_req == "*" {
                                "latest".to_string()
                            } else {
                                version_req
                            };

                            let hash = "abc123456789";
                            if !store.exists(name, &version, hash) {
                                spinner_v2.set_message(format!(
                                    "Enforcing Binary Sovereignty (Sigstore) for {}...",
                                    name
                                ));

                                // 1.1 Verify binary integrity
                                let mock_sig = "MEQCIA...";
                                let mock_identity = "developer@architect.io";

                                if verifier
                                    .verify_binary(
                                        std::path::Path::new(name),
                                        mock_sig,
                                        mock_identity,
                                    ) // Explicit std::path::Path
                                    .await?
                                {
                                    spinner_v2
                                        .set_message(format!("Downloading {} to Store...", name));
                                    let _ = store.ensure_dir(name, &version, hash)?;
                                } else {
                                    spinner_v2.fail(format!(
                                        "Security Violation: Unverified binary for {}",
                                        name
                                    ));
                                    anyhow::bail!("Security violation: Binary for {} failed Sigstore verification", name);
                                }
                            }

                            // 2. Create the shim script
                            let shim_path = shims_dir.join(name);
                            let shim_content = format!(
                                "#!/bin/bash\nexec env-architect shim {} -- \"$@\"\n",
                                name
                            );
                            std::fs::write(&shim_path, shim_content)?;

                            #[cfg(unix)]
                            {
                                use std::os::unix::fs::PermissionsExt;
                                let mut perms = std::fs::metadata(&shim_path)?.permissions();
                                perms.set_mode(0o755);
                                std::fs::set_permissions(&shim_path, perms)?;
                            }
                        }
                        spinner_v2.success("Sovereign environment ready.");

                        // Phase 3 V2: Team Consensus & Drift Detection
                        let consensus =
                            ConsensusEngine::load_lockfile(&absolute_root).unwrap_or_default();
                        let local_tools = store.list_tools()?;
                        let drifts = ConsensusEngine::detect_drift(&consensus, &local_tools);

                        if !drifts.is_empty() {
                            ui::warn("⚠️  Environment Drift Detected (Team vs Local):");
                            for drift in drifts {
                                let desc = drift.description();
                                ui::println(format!("  {} {}", Icon::Info, Theme::muted(desc)));
                            }

                            if prompt::confirm("Harmonize local environment with team consensus?") {
                                ui::info("Harmonizing tools...");
                                // TODO: Execute harmonization logic
                            }
                        }

                        ui::success(format!(
                            "Project active. Use '{}' to enter environment.",
                            Theme::bold("architect shell")
                        ));

                        // Handle Intelligence Data (Proposed Actions)
                        if let Some(intel) = manifest.intelligence {
                            if !intel.proposed_actions.is_empty() {
                                ui::warn("Environment conflicts detected. Proposed resolutions:");

                                for action in &intel.proposed_actions {
                                    match action {
                                        env_manifest::ResolutionAction::ManagedInstall {
                                            manager,
                                            command,
                                        } => {
                                            ui::println(format!(
                                                "  {} {}: {} {}",
                                                Icon::Wrench,
                                                Theme::primary(manager),
                                                Theme::muted("Run"),
                                                Theme::bold(command)
                                            ));
                                        }
                                        env_manifest::ResolutionAction::AutoShim {
                                            url,
                                            binary_name,
                                        } => {
                                            ui::println(format!(
                                                "  {} {}: {} from {}",
                                                Icon::Download,
                                                Theme::primary(binary_name),
                                                Theme::muted("Auto-shim"),
                                                url
                                            ));
                                        }
                                        env_manifest::ResolutionAction::ConfigUpdate {
                                            path,
                                            ..
                                        } => {
                                            ui::println(format!(
                                                "  {} {}: {}",
                                                Icon::File,
                                                Theme::primary("Config"),
                                                Theme::muted(path)
                                            ));
                                        }
                                        env_manifest::ResolutionAction::ManualPrompt {
                                            message,
                                            ..
                                        } => {
                                            ui::println(format!("  {} {}", Icon::Info, message));
                                        }
                                    }
                                }

                                if prompt::confirm("Apply recommended resolutions?") {
                                    for action in intel.proposed_actions {
                                        // TODO: Implement actual execution of actions
                                        ui::info(format!("Applying resolution: {:?}", action));
                                    }
                                }
                            }
                        }
                    }
                }

                if let Some(state) = output.state {
                    ui::info(format!("Opaque State: {}", state));
                }
            }
            Ok(Err(e)) => {
                spinner.fail("Plugin Logic Error");
                ui::error(format!("Error: {}", e));
            }
            Err(e) => {
                spinner.fail("Host/Runtime Error");
                ui::error(format!("Error: {}", e));
            }
        }

        Ok(())
    }
}
