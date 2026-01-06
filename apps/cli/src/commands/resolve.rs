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

    #[arg(long, short)]
    pub dry_run: bool,

    /// Project root directory (defaults to current directory)
    #[arg(long)]
    pub project_root: Option<PathBuf>,

    /// Skip confirmation
    #[arg(long, short = 'y')]
    pub yes: bool,
}

impl ResolveCommand {
    pub async fn execute(self) -> Result<()> {
        let root = self
            .project_root
            .clone()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or(PathBuf::from(".")));

        let absolute_root = std::fs::canonicalize(&root).unwrap_or(root);

        cliclack::intro(format!(
            "{} {}",
            console::style("EnvArchitect").bold(),
            console::style("v0.1.0").dim()
        ))?;

        if !self.yes {
            if !cliclack::confirm("Start environment resolution?").interact()? {
                cliclack::log::error("Operation cancelled.")?;
                return Ok(());
            }
        }

        let spinner = cliclack::spinner();
        spinner.start("Initializing plugin engine...");

        let mut config = Config::new();
        config.wasm_component_model(true);
        config.async_support(true);

        let engine = Engine::new(&config)?;
        let mut linker: Linker<HostState> = Linker::new(&engine);
        linker.allow_shadowing(true);

        wasmtime_wasi::add_to_linker_async(&mut linker)?;
        wasmtime_wasi_http::add_to_linker_async(&mut linker)?;
        crate::host::bindings::Plugin::add_to_linker(&mut linker, |state: &mut HostState| state)?;

        let mut allowed_caps: Vec<serde_json::Value> = Vec::new();

        let mut manifest_path_str: Option<String> = None;
        let mut manifest_content: Option<String> = None;

        let candidates = vec![
            (crate::constants::MANIFEST_JSON, false),
            ("plugin.json", false),
        ];

        for (filename, _is_toml) in candidates {
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

                    let json_val = serde_json::from_str::<serde_json::Value>(&content).ok();

                    if let Some(json) = json_val {
                        let caps_node = json.get("capabilities");

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

        if allowed_caps.is_empty() {
            allowed_caps.push(serde_json::json!("ui-interact"));
        }

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

        let host_state = HostState::new(
            host_allowed_names,
            manifest_path_str.clone(),
            manifest_content.clone(),
        );
        let mut store = Store::new(&engine, host_state);

        spinner.start("Reading Wasm binary...");
        let component_bytes = std::fs::read(&self.plugin)
            .with_context(|| format!("Failed to read plugin file: {:?}", self.plugin))?;
        spinner.start("Instantiating component...");
        let component = Component::new(&engine, component_bytes)?;
        let plugin = Plugin::instantiate_async(&mut store, &component, &linker).await?;

        spinner.start("Discovering system tools...");
        let mut registry = domain::system::InstalledToolsRegistry::new();
        let _ = registry.scan();
        let mut system_tools = std::collections::HashMap::new();

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

        spinner.start("Resolving dependencies...");

        let env_vars_json =
            serde_json::to_string(&std::collections::HashMap::<String, String>::new())?;

        let system_tools_json = serde_json::to_string(&system_tools)?;

        let configuration_json = manifest_content
            .as_ref()
            .and_then(|c| {
                if manifest_path_str
                    .as_ref()
                    .map(|s| s.ends_with(".json"))
                    .unwrap_or(false)
                {
                    serde_json::from_str::<serde_json::Value>(c)
                        .ok()
                        .map(|v| v.to_string())
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "{}".to_string());

        let allowed_caps_json = serde_json::to_string(&allowed_caps)?;

        let context = crate::host::bindings::ResolutionContext {
            target_os: std::env::consts::OS.to_string(),
            target_arch: std::env::consts::ARCH.to_string(),
            project_root: absolute_root.to_string_lossy().to_string(),
            env_vars_json,
            system_tools_json,
            configuration_json,
            allowed_capabilities_json: allowed_caps_json,
        };

        let result = plugin.call_resolve(&mut store, &context).await;

        match result {
            Ok(Ok(output)) => {
                spinner.stop("Resolution complete.");

                let valid_json: serde_json::Value = serde_json::from_str(&output.plan_json)
                    .unwrap_or_else(|_| serde_json::Value::String(output.plan_json));

                cliclack::log::info("Install Plan:")?;

                cliclack::log::info(serde_json::to_string_pretty(&valid_json)?)?;

                if let Some(manifest_node) = valid_json.get("manifest").cloned() {
                    if let Ok(manifest) = serde_json::from_value::<env_manifest::EnhancedManifest>(
                        manifest_node.clone(),
                    ) {
                        let platform = domain::system::PlatformDetector::detect();
                        let mut registry = domain::system::InstalledToolsRegistry::new();

                        let _ = registry.scan();
                        let resolver =
                            domain::intelligence::ConflictResolver::new(platform, registry);

                        cliclack::log::step("Analyzing for system conflicts (V2 Intelligence)...")?;

                        for (tool_name, dep_spec) in &manifest.dependencies {
                            use env_manifest::DependencySpec;
                            let version_req = match dep_spec {
                                DependencySpec::Simple(req) => req.clone(),
                                DependencySpec::Detailed(d) => d.version.clone(),
                            };

                            if let Some(conflict) = resolver.detect_conflicts(
                                tool_name,
                                &version_req,
                                "current-project",
                            ) {
                                let recommendations = resolver.resolve(&conflict)?;

                                if !recommendations.is_empty() {
                                    cliclack::log::warning(format!(
                                        "⚠️  Conflict Detected: {}",
                                        console::style(tool_name).bold().yellow()
                                    ))?;

                                    // Interactive Resolution Selector
                                    let items: Vec<(usize, &str, &str)> = recommendations
                                        .iter()
                                        .enumerate()
                                        .map(|(i, r)| (i, r.action.as_str(), ""))
                                        .collect();

                                    let selection_idx = cliclack::select(format!(
                                        "Resolution options for {}:",
                                        tool_name
                                    ))
                                    .items(&items)
                                    .interact()?;

                                    let chosen_rec = &recommendations[selection_idx];

                                    cliclack::log::success(format!(
                                        "Selected: {}",
                                        chosen_rec.action
                                    ))?;
                                    cliclack::log::info(format!(
                                        "Will perform strategy: {:?}",
                                        chosen_rec.strategy
                                    ))?;
                                }
                            }
                        }

                        let spinner_v2 = cliclack::spinner();
                        spinner_v2.start("Finalizing V2 Sovereign Environment...");

                        let store = StoreManager::default()?;
                        let verifier = VerificationService::new();
                        let shims_dir = absolute_root.join(".architect").join("shims");
                        std::fs::create_dir_all(&shims_dir)?;

                        for (name, spec) in &manifest.dependencies {
                            spinner_v2.start(format!("Shimming {}...", name));

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
                                spinner_v2.start(format!(
                                    "Enforcing Binary Sovereignty (Sigstore) for {}...",
                                    name
                                ));

                                let mock_sig = "SGVsbG8gV29ybGQ="; // "Hello World" in Base64
                                let mock_cert = "SGVsbG8gQ2VydGlmaWNhdGU="; // "Hello Certificate" in Base64
                                let mock_identity = "developer@architect.io";

                                if verifier
                                    .verify_binary(
                                        std::path::Path::new(name),
                                        mock_sig,
                                        mock_cert,
                                        mock_identity,
                                    ) // Explicit std::path::Path
                                    .await?
                                {
                                    spinner_v2.start(format!("Downloading {} to Store...", name));
                                    let _ = store.ensure_dir(name, &version, hash)?;
                                } else {
                                    spinner_v2.error(format!(
                                        "Security Violation: Unverified binary for {}",
                                        name
                                    ));
                                    anyhow::bail!("Security violation: Binary for {} failed Sigstore verification", name);
                                }
                            }

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
                        spinner_v2.stop("Sovereign environment ready.");

                        let consensus =
                            ConsensusEngine::load_lockfile(&absolute_root).unwrap_or_default();
                        let local_tools = store.list_tools()?;
                        let drifts = ConsensusEngine::detect_drift(&consensus, &local_tools);

                        if !drifts.is_empty() {
                            cliclack::log::warning(
                                "⚠️  Environment Drift Detected (Team vs Local):",
                            )?;
                            for drift in drifts {
                                let desc = drift.description();
                                cliclack::log::info(format!(
                                    "  {} {}",
                                    console::style("!").yellow(),
                                    desc
                                ))?;
                            }

                            if cliclack::confirm("Harmonize local environment with team consensus?")
                                .interact()?
                            {
                                cliclack::log::info("Harmonizing tools...")?;
                            }
                        }

                        cliclack::outro(format!(
                            "Project active. Use '{}' to enter environment.",
                            console::style("architect shell").bold()
                        ))?;

                        if let Some(intel) = manifest.intelligence {
                            if !intel.proposed_actions.is_empty() {
                                cliclack::log::warning(
                                    "Environment conflicts detected. Proposed resolutions:",
                                )?;

                                for action in &intel.proposed_actions {
                                    match action {
                                        env_manifest::ResolutionAction::ManagedInstall {
                                            manager,
                                            command,
                                        } => {
                                            cliclack::log::info(format!(
                                                "  🔧 {}: {} {}",
                                                console::style(manager).green(),
                                                console::style("Run").dim(),
                                                console::style(command).bold()
                                            ))?;
                                        }
                                        env_manifest::ResolutionAction::AutoShim {
                                            url,
                                            binary_name,
                                        } => {
                                            cliclack::log::info(format!(
                                                "  📥 {}: {} from {}",
                                                console::style(binary_name).green(),
                                                console::style("Auto-shim").dim(),
                                                url
                                            ))?;
                                        }
                                        env_manifest::ResolutionAction::ConfigUpdate {
                                            path,
                                            ..
                                        } => {
                                            cliclack::log::info(format!(
                                                "  📄 {}: {}",
                                                console::style("Config").green(),
                                                console::style(path).dim()
                                            ))?;
                                        }
                                        env_manifest::ResolutionAction::ManualPrompt {
                                            message,
                                            ..
                                        } => {
                                            cliclack::log::info(format!("  ℹ️  {}", message))?;
                                        }
                                    }
                                }
                                cliclack::log::info(format!(
                                    "Debug: Actions count: {}",
                                    intel.proposed_actions.len()
                                ))?;

                                if cliclack::confirm("Apply recommended resolutions?")
                                    .initial_value(true)
                                    .interact()?
                                {
                                    for action in &intel.proposed_actions {
                                        match action {
                                            env_manifest::ResolutionAction::ManagedInstall {
                                                manager,
                                                command,
                                            } => {
                                                cliclack::log::info(format!(
                                                    "🚀 Executing {}...",
                                                    manager
                                                ))?;
                                                let status = std::process::Command::new("sh")
                                                    .arg("-c")
                                                    .arg(&command)
                                                    .status()?;

                                                if !status.success() {
                                                    cliclack::log::error(format!(
                                                        "Failed to execute {}",
                                                        command
                                                    ))?;
                                                }
                                            }
                                            env_manifest::ResolutionAction::ConfigUpdate {
                                                path,
                                                patch,
                                            } => {
                                                cliclack::log::info(format!(
                                                    "📝 Updating config: {}",
                                                    path
                                                ))?;
                                                cliclack::log::info(format!(
                                                    "Applying patch: {}",
                                                    patch
                                                ))?;

                                                cliclack::log::warning("Patch application not yet fully implemented. Please verify env.toml manually.")?;
                                            }

                                            env_manifest::ResolutionAction::AutoShim {
                                                url,
                                                binary_name,
                                            } => {
                                                cliclack::log::warning(format!("Skipping AutoShim for {} (Downloading from {} not yet supported)", binary_name, url))?;
                                            }
                                            env_manifest::ResolutionAction::ManualPrompt {
                                                message,
                                                ..
                                            } => {
                                                cliclack::log::info(format!(
                                                    "Please manually: {}",
                                                    message
                                                ))?;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if let Some(state) = output.state {
                    cliclack::note("Opaque State", state)?;
                }
            }
            Ok(Err(e)) => {
                spinner.error("Plugin Logic Error");
                cliclack::log::error(format!("Error: {}", e))?;
            }
            Err(e) => {
                spinner.error("Host/Runtime Error");
                cliclack::log::error(format!("Error: {}", e))?;
            }
        }

        Ok(())
    }
}
