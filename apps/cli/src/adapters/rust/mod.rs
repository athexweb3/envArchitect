use super::{PluginAdapter, PluginMetadata};
use anyhow::{Context, Result};
use async_trait::async_trait;
use std::path::{Path, PathBuf};

mod build;
mod metadata;

pub struct RustAdapter;

impl RustAdapter {
    pub fn new() -> Self {
        Self
    }

    pub fn matches(dir: &Path) -> bool {
        dir.join("Cargo.toml").exists()
    }
}

#[async_trait]
impl PluginAdapter for RustAdapter {
    fn name(&self) -> &str {
        "rust"
    }

    fn matches(&self, dir: &Path) -> bool {
        Self::matches(dir)
    }

    async fn build(&self, dir: &Path, options: super::BuildOptions) -> Result<PathBuf> {
        build::build_rust_project(dir, options).await
    }

    async fn watch(&self, dir: &Path) -> Result<()> {
        let (tx, rx) = std::sync::mpsc::channel();
        let mut watcher =
            notify::recommended_watcher(move |res: notify::Result<notify::Event>| {
                if let Ok(event) = res {
                    if event.kind.is_modify() || event.kind.is_create() {
                        let is_target = event
                            .paths
                            .iter()
                            .any(|p| p.components().any(|c| c.as_os_str() == "target"));
                        if !is_target {
                            let _ = tx.send(());
                        }
                    }
                }
            })?;

        use notify::Watcher;
        watcher.watch(dir, notify::RecursiveMode::Recursive)?;

        cliclack::log::info(format!("Watching Rust project at {:?}", dir))?;

        if let Err(e) = self.build(dir, super::BuildOptions::default()).await {
            cliclack::log::error(format!("Build failed: {}", e))?;
        }

        loop {
            if rx.recv().is_ok() {
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                while rx.try_recv().is_ok() {}

                cliclack::log::step("Change detected...")?;
                if let Err(e) = self.build(dir, super::BuildOptions::default()).await {
                    cliclack::log::error(format!("Build failed: {}", e))?;
                } else {
                    cliclack::log::success("Build complete.")?;
                }
            }
        }
    }

    fn metadata(&self, dir: &Path) -> Result<PluginMetadata> {
        metadata::extract_metadata(dir)
    }

    async fn generate_sbom(&self, _dir: &Path) -> Result<PathBuf> {
        Err(anyhow::anyhow!("Rust sbom not yet implemented"))
    }

    async fn check_health(&self) -> Result<()> {
        let spinner = cliclack::spinner();
        spinner.start("Checking Rust toolchain...");

        if std::process::Command::new("cargo")
            .arg("--version")
            .output()
            .is_err()
        {
            spinner.stop("Failed");
            anyhow::bail!("Cargo not found. Please install Rust: https://rustup.rs");
        }

        let output = std::process::Command::new("rustc")
            .arg("--print")
            .arg("target-list")
            .output()
            .context("Failed to check targets")?;

        let targets = String::from_utf8_lossy(&output.stdout);
        if !targets.contains("wasm32-wasip1") && !targets.contains("wasm32-wasi") {
            spinner.stop("Failed");
            anyhow::bail!("Missing Wasm targets. Run: rustup target add wasm32-wasip1");
        }

        spinner.stop("Rust toolchain OK");
        Ok(())
    }

    async fn scaffold(&self, dir: &Path, _name: &str) -> Result<()> {
        let spinner = cliclack::spinner();
        spinner.start("Scaffolding Rust project...");

        if std::process::Command::new("cargo")
            .arg("init")
            .arg("--lib")
            .current_dir(dir)
            .output()
            .is_err()
        {
            spinner.stop("Failed");
            anyhow::bail!("Failed to run cargo init");
        }

        spinner.stop("Rust project created");
        Ok(())
    }
}
