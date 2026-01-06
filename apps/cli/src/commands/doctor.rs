use anyhow::Result;
use clap::Parser;
use wasmtime::component::{Component, Linker};
use wasmtime::{Config, Engine, Store};

use crate::host::bindings::Plugin;
use crate::host::state::HostState;

#[derive(Parser, Debug)]
pub struct DoctorCommand {
    /// Optional project path
    #[arg(short, long)]
    pub path: Option<std::path::PathBuf>,
}

impl DoctorCommand {
    pub async fn execute(self) -> Result<()> {
        cliclack::intro(console::style("EnvArchitect Doctor").bold())?;
        cliclack::log::info("Initializing embedded Physician...")?;

        let root = self
            .path
            .clone()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or(std::path::PathBuf::from(".")));

        let absolute_root = std::fs::canonicalize(&root).unwrap_or(root.clone());

        if let Ok(adapter) = crate::adapters::get_adapter(&absolute_root) {
            cliclack::log::info(format!(
                "Checking toolchain for language: {}",
                adapter.name()
            ))?;
            match adapter.check_health().await {
                Ok(_) => {
                    cliclack::log::success(format!("Toolchain for {} is healthy.", adapter.name()))?
                }
                Err(e) => {
                    cliclack::log::error(format!("Toolchain issues: {}", e))?;
                }
            }
        } else {
            cliclack::log::warning(
                "No supported project language detected. Skipping toolchain check.",
            )?;
        }

        let mut config = Config::new();
        config.wasm_component_model(true);
        config.async_support(true);

        let engine = Engine::new(&config)?;
        let mut linker: Linker<HostState> = Linker::new(&engine);

        wasmtime_wasi::add_to_linker_async(&mut linker)?;
        crate::host::bindings::Plugin::add_to_linker(&mut linker, |state: &mut HostState| state)?;

        // For Doctor, we grant all read-only capabilities by default + exec
        // We want it to be powerful enough to check everything.
        let allowed_caps = vec![
            "fs-read".to_string(),
            "sys-exec".to_string(),
            "ui-interact".to_string(),
            "core.env".to_string(), // Internal marker if needed
        ];

        let host_state = HostState::new(allowed_caps, None, None);
        let mut store = Store::new(&engine, host_state);

        // EMBEDDING THE DOCTOR PLUGIN HERE
        // Relative to this file: apps/cli/src/commands/doctor.rs
        // We need to go up to root: ../../../../
        // const DOCTOR_WASM: &[u8] =
        //     include_bytes!("../../../../target/wasm32-wasip1/debug/env_plugin_doctor.wasm");
        const DOCTOR_WASM: &[u8] = &[]; // TODO: Restore when doctor plugin is built

        if DOCTOR_WASM.is_empty() {
            cliclack::log::warning("Doctor plugin not built. Skipping Wasm diagnostics.")?;
            return Ok(());
        }

        let component = Component::new(&engine, DOCTOR_WASM)?;
        let plugin = Plugin::instantiate_async(&mut store, &component, &linker).await?;

        // We use the 'validate' hook or 'resolve' hook. Doctor is currently wired to print on 'resolve'.

        let allowed_caps_json = serde_json::to_string(&vec![
            serde_json::json!({ "fs-read": ["/"] }),
            serde_json::json!({ "sys-exec": ["*"] }),
            serde_json::json!("ui-interact"),
        ])?;

        let context = crate::host::bindings::ResolutionContext {
            target_os: std::env::consts::OS.to_string(),
            target_arch: std::env::consts::ARCH.to_string(),
            project_root: absolute_root.to_string_lossy().to_string(),
            env_vars_json: "{}".to_string(),
            system_tools_json: "{}".to_string(),
            configuration_json: "{}".to_string(),
            allowed_capabilities_json: allowed_caps_json,
        };

        match plugin.call_resolve(&mut store, &context).await? {
            Ok(_) => {
                cliclack::log::success("Diagnostics Complete.")?;
            }
            Err(e) => {
                cliclack::log::error(format!("Plugin Execution Failed: {}", e))?;
            }
        }

        Ok(())
    }
}
