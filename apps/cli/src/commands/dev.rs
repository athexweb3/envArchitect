use crate::commands::resolve::ResolveCommand;
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
        cliclack::intro(format!(
            "{} {}",
            console::style("EnvArchitect Dev Mode").bold(),
            console::style(env!("CARGO_PKG_VERSION")).dim()
        ))?;

        cliclack::log::info(format!("Watching path: {:?}", self.path))?;

        // Initial run
        if let Err(e) = self.run_iteration().await {
            cliclack::log::error(format!("Initial run failed: {}", e))?;
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

                cliclack::log::step("Change detected! Re-running plugin...")?;

                if let Err(e) = self.run_iteration().await {
                    cliclack::log::error(format!("Dev iteration failed: {}", e))?;
                }
            }
        }
    }

    async fn run_iteration(&self) -> Result<()> {
        // 1. Rebuild if it's a Rust project (simple check)
        if self.path.join("Cargo.toml").exists() {
            let spinner = cliclack::spinner();
            spinner.start("Detecting project type...");

            spinner.start("Building Wasm binary (Rust)...");

            // Try wasm32-wasip1 first as it's the newer standard, fall back to wasm32-wasi
            let mut status = std::process::Command::new("cargo")
                .arg("build")
                .arg("--target")
                .arg("wasm32-wasip1")
                .current_dir(&self.path)
                .status();

            if status.is_err() || !status.as_ref().unwrap().success() {
                spinner.start("wasm32-wasip1 failed, trying legacy wasm32-wasi...");
                status = std::process::Command::new("cargo")
                    .arg("build")
                    .arg("--target")
                    .arg("wasm32-wasi")
                    .current_dir(&self.path)
                    .status();
            }

            if let Ok(s) = status {
                if !s.success() {
                    spinner.error("Cargo build failed");
                    anyhow::bail!("Cargo build failed");
                } else {
                    spinner.stop("Build complete");
                }
            } else {
                spinner.error("Failed to execute cargo build");
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
                // 1. Standard local target
                PathBuf::from("target/wasm32-wasip1/debug").join(&wasm_filename),
                PathBuf::from("target/wasm32-wasi/debug").join(&wasm_filename),
                // 2. Relative to plugin path (if it's a standalone crate)
                self.path
                    .join("target/wasm32-wasip1/debug")
                    .join(&wasm_filename),
                self.path
                    .join("target/wasm32-wasi/debug")
                    .join(&wasm_filename),
                // 3. Workspace root (likely case for monorepo)
                // Assuming we are 2 levels deep in examples/demo, or plugin is 3 levels deep in packages/plugins/python
                // We try a few hops up from the plugin path
                self.path
                    .join("../../target/wasm32-wasip1/debug")
                    .join(&wasm_filename),
                self.path
                    .join("../../target/wasm32-wasi/debug")
                    .join(&wasm_filename),
                self.path
                    .join("../../../target/wasm32-wasip1/debug")
                    .join(&wasm_filename),
                self.path
                    .join("../../../target/wasm32-wasi/debug")
                    .join(&wasm_filename),
                // 4. Default fallback
                self.wasm.clone(),
            ];

            for path in &possible_paths {
                if path.exists() {
                    wasm_path = path.clone();
                    cliclack::log::info(format!("Found Wasm binary at: {:?}", path))?;
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

        cliclack::log::info(format!("Using Wasm plugin: {:?}", wasm_path))?;
        cliclack::log::info(format!("Component output path: {:?}", component_path))?;

        if self.path.join("Cargo.toml").exists() {
            let spinner = cliclack::spinner();
            spinner.start("Checking adapters...");

            // Step 0: Ensure WASI Adapter exists
            // We need this to bridge wasm32-wasip1 imports to the Component Model.
            let adapter_dir = self.path.join("target/adapters");
            let adapter_path = adapter_dir.join("wasi_snapshot_preview1.reactor.wasm");

            if !adapter_path.exists() {
                spinner.start("Downloading WASI adapter...");
                std::fs::create_dir_all(&adapter_dir).ok();

                let _ = std::process::Command::new("curl")
                    .arg("-L")
                    .arg("-s") // Silent
                    .arg("-o")
                    .arg(&adapter_path)
                    .arg("https://github.com/bytecodealliance/wasmtime/releases/download/v25.0.0/wasi_snapshot_preview1.reactor.wasm")
                    .status();
                spinner.start("Using cached WASI adapter...");
            }

            spinner.start("Stripping symbols...");
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
            // Robustly find workspace root to locate WIT
            let mut wit_path = PathBuf::from("packages/sdks/wit/plugin.wit");

            // Try explicit lookup based on common depth patterns
            let candidates = [
                PathBuf::from("packages/sdks/wit/plugin.wit"),
                PathBuf::from("../packages/sdks/wit/plugin.wit"),
                PathBuf::from("../../packages/sdks/wit/plugin.wit"),
                PathBuf::from("../../../packages/sdks/wit/plugin.wit"),
                // Try resolving relative to the plugin path itself
                self.path.join("../../sdks/wit/plugin.wit"),
                self.path.join("../../../sdks/wit/plugin.wit"),
            ];

            for p in &candidates {
                if p.exists() {
                    wit_path = p.clone();
                    break;
                }
            }

            spinner.start("Embedding WIT interface...");
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
                spinner.error("WIT Embedding failed.");
            } else {
                // Step 3: Componentize
                spinner.start("Creating component...");
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
                        spinner.stop("Component created.");
                        wasm_path = component_path;
                        // cleanup intermediates
                        let _ = std::fs::remove_file(embedded_path);
                        if stripped_path.exists() {
                            let _ = std::fs::remove_file(stripped_path);
                        }
                    }
                    Ok(s) => {
                        spinner.error(format!("Componentization failed: {}", s));
                    }
                    Err(e) => {
                        spinner.error(format!("Failed to run wasm-tools: {}", e));
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
