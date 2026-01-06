use super::{BuildOptions, PluginAdapter, PluginMetadata};
use anyhow::Result;
use async_trait::async_trait;
use std::path::{Path, PathBuf};

mod build;
mod metadata;
mod scaffold;

pub struct TsAdapter;

impl TsAdapter {
    pub fn new() -> Self {
        Self
    }

    pub fn matches(dir: &Path) -> bool {
        dir.join("package.json").exists()
    }
}

#[async_trait]
impl PluginAdapter for TsAdapter {
    fn name(&self) -> &str {
        "ts"
    }

    fn matches(&self, dir: &Path) -> bool {
        Self::matches(dir)
    }

    async fn build(&self, dir: &Path, options: BuildOptions) -> Result<PathBuf> {
        build::build_ts_project(dir, options).await
    }

    async fn watch(&self, _dir: &Path) -> Result<()> {
        // Simple watch implementation (similar to Rust adapter)

        // Node tools usually have their own watch mode (npm run watch).
        // But for consistency, we trigger rebuilds.
        cliclack::log::info("Watching TypeScript project (Not full implemented)")?;
        Ok(())
    }

    fn metadata(&self, dir: &Path) -> Result<PluginMetadata> {
        metadata::extract_metadata(dir)
    }

    async fn generate_sbom(&self, _dir: &Path) -> Result<PathBuf> {
        Err(anyhow::anyhow!("TS SBOM logic not implemented"))
    }

    async fn check_health(&self) -> Result<()> {
        let spinner = cliclack::spinner();
        spinner.start("Checking Node.js toolchain...");

        match std::process::Command::new("node").arg("--version").output() {
            Ok(output) => {
                let version = String::from_utf8_lossy(&output.stdout);
                if !version.starts_with('v') {
                    spinner.stop("Failed");
                    anyhow::bail!("Unexpected Node.js version format: {}", version);
                }
            }
            Err(_) => {
                spinner.stop("Failed");
                anyhow::bail!("Node.js not found. Please install Node.js.");
            }
        }

        if std::process::Command::new("npm")
            .arg("--version")
            .output()
            .is_err()
        {
            spinner.stop("Failed");
            anyhow::bail!("npm not found. Please install npm.");
        }

        spinner.stop("Node.js toolchain OK");
        Ok(())
    }

    async fn scaffold(&self, dir: &Path, name: &str) -> Result<()> {
        scaffold::scaffold_ts_project(dir, name).await
    }
}
