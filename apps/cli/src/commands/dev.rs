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
                let is_ignored = event.paths.iter().any(|p| {
                    p.components().any(|c| {
                        let s = c.as_os_str();
                        s == "target" || s == "node_modules" || s == "dist" || s == ".git"
                    })
                });

                if !is_ignored && (event.kind.is_modify() || event.kind.is_create()) {
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
        let mut wasm_path = self.wasm.clone();

        // If we find one, we let it build and tell us where the Wasm is.
        if let Ok(adapter) = crate::adapters::get_adapter(&self.path) {
            cliclack::log::info(format!("Detected language: {}", adapter.name()))?;

            match adapter
                .build(&self.path, crate::adapters::BuildOptions::default())
                .await
            {
                Ok(path) => {
                    wasm_path = path;
                    cliclack::log::success(format!("Build successful: {:?}", wasm_path))?;
                }
                Err(e) => {
                    cliclack::log::error(format!("Build failed: {}", e))?;
                    anyhow::bail!("Build failed");
                }
            }
        } else {
            cliclack::log::warning("No language adapter detected. Watching static Wasm file...")?;
        }

        // We now have a valid wasm_path (either from build or arg)

        let resolve_cmd = ResolveCommand {
            plugin: wasm_path.clone(),
            dry_run: true,
            project_root: Some(self.path.clone()),
            yes: true,
        };

        if let Err(e) = resolve_cmd.execute().await {
            cliclack::log::error(format!("Resolution failed: {}", e))?;
        }

        Ok(())
    }
}
