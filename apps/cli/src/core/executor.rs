use anyhow::{Context, Result};
use std::path::PathBuf;
use wasmtime::component::{Component, Linker};
use wasmtime::{Config, Engine, Store};

use crate::host::bindings::Plugin;
use crate::host::state::HostState;

pub struct SystemExecutor;

impl SystemExecutor {
    pub async fn install(plugin_path: &PathBuf) -> Result<()> {
        let spinner = cliclack::spinner();
        spinner.start("Initializing system plugin...");

        let mut config = Config::new();
        config.wasm_component_model(true);
        config.async_support(true);

        let engine = Engine::new(&config)?;
        let mut linker: Linker<HostState> = Linker::new(&engine);

        wasmtime_wasi::add_to_linker_async(&mut linker)?;
        crate::host::bindings::Plugin::add_to_linker(&mut linker, |state: &mut HostState| state)?;

        let allowed_caps = vec![
            "ui-interact".to_string(),
            "ui-secret".to_string(),
            "fs-read".to_string(),
            "sys-exec".to_string(), // Future: allow plugin to check current versions
            "env-read".to_string(),
        ];

        // Host state for system install doesn't need a manifest
        let host_state = HostState::new(allowed_caps, None, None);
        let mut store = Store::new(&engine, host_state);

        // spinner.set_message("Reading plugin...");
        let component_bytes = std::fs::read(plugin_path)
            .with_context(|| format!("Failed to read plugin file: {:?}", plugin_path))?;

        let component = Component::new(&engine, component_bytes)?;
        let plugin = Plugin::instantiate_async(&mut store, &component, &linker).await?;

        spinner.stop("Plugin loaded.");

        let allowed_caps_json = serde_json::to_string(&vec![
            serde_json::json!("ui-interact"),
            serde_json::json!({ "sys-exec": ["*"] }),
            serde_json::json!({ "env-read": ["*"] }),
            serde_json::json!({ "fs-read": ["*"] }),
        ])?;

        let context = crate::host::bindings::ResolutionContext {
            target_os: std::env::consts::OS.to_string(),
            target_arch: std::env::consts::ARCH.to_string(),
            project_root: "/".to_string(),
            env_vars_json: "{}".to_string(),
            system_tools_json: "{}".to_string(),
            configuration_json: "{}".to_string(),
            allowed_capabilities_json: allowed_caps_json,
        };

        let result = plugin.call_resolve(&mut store, &context).await?;

        match result {
            Ok(output) => {
                let valid_json: serde_json::Value = serde_json::from_str(&output.plan_json)
                    .unwrap_or_else(|_| serde_json::Value::String(output.plan_json.clone()));

                // Check for instructions
                let mut instructions: Vec<String> = Vec::new();

                if let Some(plan) = valid_json.as_object() {
                    if let Some(instr_array) = plan.get("instructions").and_then(|i| i.as_array()) {
                        for instr in instr_array {
                            if let Some(s) = instr.as_str() {
                                instructions.push(s.to_string());
                            }
                        }
                    }
                }

                if !instructions.is_empty() {
                    cliclack::log::step("Executing installation plan:")?;

                    for cmd in instructions {
                        cliclack::log::info(format!("Running: {}", console::style(&cmd).dim()))?;

                        let spinner_cmd = cliclack::spinner();
                        spinner_cmd.start("Processing...");

                        // Capture output to hide logs unless error
                        let output_res = std::process::Command::new("bash")
                            .arg("-c")
                            .arg(&cmd)
                            .output()
                            .context("Failed to execute shell command")?;

                        if !output_res.status.success() {
                            spinner_cmd.error("Execution failed.");

                            // Print captured stderr/stdout for debugging
                            if !output_res.stdout.is_empty() {
                                cliclack::log::info(String::from_utf8_lossy(&output_res.stdout))?;
                            }
                            if !output_res.stderr.is_empty() {
                                cliclack::log::error(String::from_utf8_lossy(&output_res.stderr))?;
                            }

                            return Err(anyhow::anyhow!("Command failed: {}", cmd));
                        }

                        spinner_cmd.stop("Completed.");
                    }

                    if let Some(state) = output.state {
                        cliclack::log::info(format!("Success: {}", state))?;
                    }
                } else {
                    cliclack::log::warning("Plugin returned no installation instructions.")?;
                }
            }
            Err(e) => {
                cliclack::log::error(format!("Plugin Resolved Error: {}", e))?;
                return Err(anyhow::anyhow!("Plugin logic failed: {}", e));
            }
        }

        Ok(())
    }
}
