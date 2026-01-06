use anyhow::Result;
use async_trait::async_trait;
use std::path::{Path, PathBuf};

pub mod rust;
pub mod ts;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub authors: Option<Vec<String>>,
    pub license: Option<String>,
    pub repository: Option<String>,
    // Capabilities and Dependencies are usually strictly defined in env.toml/env.json
    // But we can allow extracting them from package.json if possible.
    pub capabilities: Option<Vec<String>>,
    // We use a generic Map or Value for dependencies to avoid strict typing here?
}

#[derive(Debug, Clone, Default)]
pub struct BuildOptions {
    pub release: bool,
}

#[async_trait]
#[allow(dead_code)]
pub trait PluginAdapter: Send + Sync {
    /// Returns the unique name of the language adapter (e.g., "rust", "ts")
    fn name(&self) -> &str;

    /// Returns true if this adapter can handle the project at `dir`
    fn matches(&self, dir: &Path) -> bool;

    /// Builds the project into a Wasm component.
    /// Returns the path to the final .wasm file.
    async fn build(&self, dir: &Path, options: BuildOptions) -> Result<PathBuf>;

    /// Optional: Custom logic for watching file changes.
    /// Default implementation can rely on the caller or a standard watcher.
    async fn watch(&self, dir: &Path) -> Result<()>;

    /// Extracts plugin metadata (name, version, etc.)
    fn metadata(&self, dir: &Path) -> Result<PluginMetadata>;

    /// Generates an SBOM for the project.
    async fn generate_sbom(&self, dir: &Path) -> Result<PathBuf>;

    /// Checks if the language toolchain is installed and healthy.
    async fn check_health(&self) -> Result<()>;

    /// Scaffolds a new project for this language.
    async fn scaffold(&self, dir: &Path, name: &str) -> Result<()>;
}

pub fn get_adapter(dir: &Path) -> Result<Box<dyn PluginAdapter>> {
    // Priority order matters if a project could match multiple (unlikely but possible)
    if rust::RustAdapter::matches(dir) {
        return Ok(Box::new(rust::RustAdapter::new()));
    }

    if ts::TsAdapter::matches(dir) {
        return Ok(Box::new(ts::TsAdapter::new()));
    }

    Err(anyhow::anyhow!(
        "No supported language detected in {:?}",
        dir
    ))
}
