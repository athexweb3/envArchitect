use crate::commands::resolve::ResolveCommand;
use crate::ui::{self, Icon};
use anyhow::Result;
use clap::Parser;
use notify::{Event, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc::channel;
use std::time::Duration;

#[derive(Parser, Debug, Clone)]
pub struct DevCommand {
    /// Path to the plugin directory or source file
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Wasm file to watch (if not providing a directory to build)
    #[arg(long, default_value = "target/wasm32-wasi/debug/simple_plugin.wasm")]
    pub wasm: PathBuf,
}

impl DevCommand {
    pub async fn execute(self) -> Result<()> {
        ui::println(format!(
            "{} {} v{}",
            Icon::Architect,
            "EnvArchitect Dev Mode",
            env!("CARGO_PKG_VERSION")
        ));
        ui::info(format!("Watching path: {:?}", self.path));

        // Initial run
        if let Err(e) = self.run_iteration().await {
            ui::error(format!("Initial run failed: {}", e));
        }

        // Setup watcher
        let (tx, rx) = channel();
        let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            if let Ok(event) = res {
                let is_target = event
                    .paths
                    .iter()
                    .any(|p| p.components().any(|c| c.as_os_str() == "target"));

                if !is_target && (event.kind.is_modify() || event.kind.is_create()) {
                    let _ = tx.send(());
                }
            }
        })?;

        watcher.watch(&self.path, RecursiveMode::Recursive)?;

        // Loop and wait for events
        loop {
            // Wait for event with debounce
            if rx.recv().is_ok() {
                // Debounce: wait a bit for more events
                tokio::time::sleep(Duration::from_millis(500)).await;
                // Clear any pending events
                while rx.try_recv().is_ok() {}

                ui::println(format!(
                    "\n{} Change detected! Re-running plugin...",
                    Icon::Rocket
                ));
                if let Err(e) = self.run_iteration().await {
                    ui::error(format!("Dev iteration failed: {}", e));
                }
            }
        }
    }

    async fn run_iteration(&self) -> Result<()> {
        // 1. Rebuild if it's a Rust project (simple check)
        if self.path.join("Cargo.toml").exists() {
            let spinner = ui::components::spinner::Spinner::new("Hot Reload");
            spinner.set_message("Detecting project type...");

            spinner.set_message("Building Wasm binary (Rust)...");

            // Try wasm32-wasip1 first as it's the newer standard, fall back to wasm32-wasi
            let mut status = std::process::Command::new("cargo")
                .arg("build")
                .arg("--target")
                .arg("wasm32-wasip1")
                .current_dir(&self.path)
                .status();

            if status.is_err() || !status.as_ref().unwrap().success() {
                spinner.set_message("wasm32-wasip1 failed, trying legacy wasm32-wasi...");
                status = std::process::Command::new("cargo")
                    .arg("build")
                    .arg("--target")
                    .arg("wasm32-wasi")
                    .current_dir(&self.path)
                    .status();
            }

            if let Ok(s) = status {
                if !s.success() {
                    anyhow::bail!("Cargo build failed");
                }
            } else {
                anyhow::bail!("Failed to execute cargo build");
            }
        }

        // 2. Run Resolution
        let mut wasm_path = self.wasm.clone();

        // If it's a Rust project, we might need to adjust the path based on the target we just built
        if self.path.join("Cargo.toml").exists() {
            let cargo_toml_path = self.path.join("Cargo.toml");
            let cargo_content = std::fs::read_to_string(&cargo_toml_path)?;
            let cargo_value: toml::Value = cargo_content.parse()?;

            let package_name = cargo_value
                .get("package")
                .and_then(|p| p.get("name"))
                .and_then(|n| n.as_str())
                .unwrap_or("plugin");

            let wasm_filename = format!("{}.wasm", package_name.replace('-', "_"));

            // In a workspace, target is usually at the root.
            // Let's try multiple places:
            let possible_paths = [
                PathBuf::from("target/wasm32-wasip1/debug").join(&wasm_filename),
                PathBuf::from("target/wasm32-wasi/debug").join(&wasm_filename),
                self.path
                    .join("target/wasm32-wasip1/debug")
                    .join(&wasm_filename),
                self.path
                    .join("target/wasm32-wasi/debug")
                    .join(&wasm_filename),
                self.wasm.clone(),
            ];

            for path in &possible_paths {
                if path.exists() {
                    wasm_path = path.clone();
                    break;
                }
            }
        }

        // 3. Componentize! (The secret sauce)
        // If we just built a Rust project, we have a core module. We need to make it a component.
        // We'll output to a new file with .component.wasm extension
        let component_path = if wasm_path.extension().map_or(false, |e| e == "wasm") {
            wasm_path.with_extension("component.wasm")
        } else {
            wasm_path.clone()
        };

        ui::info(format!("Using Wasm plugin: {:?}", wasm_path));
        ui::info(format!("Component output path: {:?}", component_path));

        if self.path.join("Cargo.toml").exists() {
            let spinner = ui::components::spinner::Spinner::new("Componentizing");
            spinner.set_message("Checking adapters...");

            // Step 0: Ensure WASI Adapter exists
            // We need this to bridge wasm32-wasip1 imports to the Component Model.
            let adapter_dir = self.path.join("target/adapters");
            let adapter_path = adapter_dir.join("wasi_snapshot_preview1.reactor.wasm");

            if !adapter_path.exists() {
                spinner.set_message("Downloading WASI adapter...");
                std::fs::create_dir_all(&adapter_dir).ok();

                let _ = std::process::Command::new("curl")
                    .arg("-L")
                    .arg("-s") // Silent
                    .arg("-o")
                    .arg(&adapter_path)
                    .arg("https://github.com/bytecodealliance/wasmtime/releases/download/v25.0.0/wasi_snapshot_preview1.reactor.wasm")
                    .status();
                spinner.set_message("Using cached WASI adapter...");
            }

            spinner.set_message("Stripping symbols...");
            let stripped_path = wasm_path.with_extension("stripped.wasm");
            let _ = std::process::Command::new("wasm-tools")
                .arg("strip")
                .arg("-a")
                .arg(&wasm_path)
                .arg("-o")
                .arg(&stripped_path)
                .output();

            // We use the stripped path if it exists, otherwise fallback to original
            if stripped_path.exists() {
                wasm_path = stripped_path.clone();
            }

            // Step 2: Embed WIT
            let mut wit_path = PathBuf::from("packages/sdks/wit/plugin.wit");
            if !wit_path.exists() {
                wit_path = self.path.join("../../packages/sdks/wit/plugin.wit");
            }
            if !wit_path.exists() {
                wit_path = PathBuf::from("../packages/sdks/wit/plugin.wit");
            }

            spinner.set_message("Embedding WIT interface...");
            let embedded_path = wasm_path.with_extension("embed.wasm");

            let embed_status = std::process::Command::new("wasm-tools")
                .arg("component")
                .arg("embed")
                .arg(&wit_path)
                .arg(&wasm_path)
                .arg("-o")
                .arg(&embedded_path)
                .arg("--world")
                .arg("plugin")
                .status();

            if embed_status.is_err() || !embed_status.as_ref().unwrap().success() {
                spinner.fail("WIT Embedding failed.");
            } else {
                // Step 3: Componentize
                spinner.set_message("Creating component...");
                let status = std::process::Command::new("wasm-tools")
                    .arg("component")
                    .arg("new")
                    .arg(&embedded_path)
                    .arg("-o")
                    .arg(&component_path)
                    .arg("--adapt")
                    .arg(format!("wasi_snapshot_preview1={}", adapter_path.display()))
                    .status();

                match status {
                    Ok(s) if s.success() => {
                        spinner.success("Component created.");
                        wasm_path = component_path;
                        // cleanup intermediates
                        let _ = std::fs::remove_file(embedded_path);
                        if stripped_path.exists() {
                            let _ = std::fs::remove_file(stripped_path);
                        }
                    }
                    Ok(s) => {
                        spinner.fail(format!("Componentization failed: {}", s));
                    }
                    Err(e) => {
                        spinner.fail(format!("Failed to run wasm-tools: {}", e));
                    }
                }
            }
        }

        let resolve_cmd = ResolveCommand {
            plugin: wasm_path,
            dry_run: true,
            project_root: Some(self.path.clone()),
        };

        resolve_cmd.execute().await?;

        Ok(())
    }
}
