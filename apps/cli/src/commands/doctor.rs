use anyhow::{Result};
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

        let absolute_root = std::fs::canonicalize(&root).unwrap_or(root);

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

        // 4. Load Embedded Component
        // EMBEDDING THE DOCTOR PLUGIN HERE
        // Relative to this file: apps/cli/src/commands/doctor.rs
        // We need to go up to root: ../../../../
        const DOCTOR_WASM: &[u8] = include_bytes!(
            "../../../../target/wasm32-wasip1/debug/env_plugin_doctor.component.wasm"
        );

        let component = Component::new(&engine, DOCTOR_WASM)?;
        let plugin = Plugin::instantiate_async(&mut store, &component, &linker).await?;

        // 5. Execute Validation
        // We use the 'validate' hook or 'resolve' hook. Doctor is currently wired to print on 'resolve'.
        // Let's call verify/resolve.

        let context = serde_json::json!({
            "target_os": std::env::consts::OS,
            "target_arch": std::env::consts::ARCH,
            "project_root": absolute_root.to_string_lossy(),
            "env_vars": {},
            "allowed_capabilities": [
                { "fs-read": ["/"] },
                { "sys-exec": ["*"] },
                "ui-interact"
            ],
            "parent_manifest": null,
            "system_tools": {}
        });

        // Ignoring output plan, just running for side-effects (diagnostics printing)
        // Ignoring output plan, just running for side-effects (diagnostics printing)
        match plugin
            .call_resolve(&mut store, &context.to_string())
            .await?
        {
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
