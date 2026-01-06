use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

#[derive(Parser)]
pub struct BundleCommand {
    /// Path to the package manifest (e.g. Cargo.toml or directory)
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Skip optimization step
    #[arg(long)]
    pub no_optimize: bool,
}

impl BundleCommand {
    pub async fn execute(&self) -> Result<PathBuf> {
        crate::services::bundle::BundleService::execute(&self.path, self.no_optimize).await
    }
}
