use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

/// Manages the immutable Architect Store
pub struct StoreManager {
    root: PathBuf,
}

impl StoreManager {
    /// Initialize the store manager at the given root path
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    /// Initialize the default store manager at ~/.architect/store
    pub fn default() -> Result<Self> {
        let home = dirs::home_dir().context("Could not find home directory")?;
        let root = home.join(".architect").join("store");

        if !root.exists() {
            fs::create_dir_all(&root).context("Failed to create architect store directory")?;
        }

        Ok(Self::new(root))
    }

    /// Calculate the store path for a tool version
    pub fn calculate_path(&self, tool: &str, version: &str, content_hash: &str) -> PathBuf {
        // Path format: <root>/<hash>-<tool>-<version>
        // Using a short hash for readability in the filename (first 12 chars)
        let short_hash = &content_hash[..12.min(content_hash.len())];
        let dirname = format!("{}-{}-{}", short_hash, tool, version);
        self.root.join(dirname)
    }

    /// Check if a tool version already exists in the store
    pub fn exists(&self, tool: &str, version: &str, content_hash: &str) -> bool {
        self.calculate_path(tool, version, content_hash).exists()
    }

    /// Ensure a tool directory exists in the store
    pub fn ensure_dir(&self, tool: &str, version: &str, content_hash: &str) -> Result<PathBuf> {
        let path = self.calculate_path(tool, version, content_hash);
        if !path.exists() {
            fs::create_dir_all(&path)
                .context(format!("Failed to create store directory: {:?}", path))?;
        }
        Ok(path)
    }

    /// List all tools in the store
    pub fn list_tools(&self) -> Result<Vec<String>> {
        let mut tools = Vec::new();
        if !self.root.exists() {
            return Ok(tools);
        }

        for entry in fs::read_dir(&self.root)? {
            let entry = entry?;
            if let Some(name) = entry.file_name().to_str() {
                // Parse tool name from directory name: <hash>-<tool>-<version>
                let parts: Vec<&str> = name.split('-').collect();
                if parts.len() >= 2 {
                    tools.push(parts[1].to_string());
                }
            }
        }
        tools.sort();
        tools.dedup();
        Ok(tools)
    }

    /// Get the actual path to a tool's executable within its store directory
    pub fn get_executable_path(
        &self,
        tool: &str,
        version: &str,
        content_hash: &str,
        binary_name: &str,
    ) -> Option<PathBuf> {
        let root = self.calculate_path(tool, version, content_hash);
        let bin_path = root.join("bin").join(binary_name);
        if bin_path.exists() {
            return Some(bin_path);
        }

        // Fallback to searching root directly
        let direct_path = root.join(binary_name);
        if direct_path.exists() {
            return Some(direct_path);
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_store_path_calculation() {
        let root = PathBuf::from("/tmp/store");
        let manager = StoreManager::new(root.clone());
        let path = manager.calculate_path("node", "20.11.0", "abc1234567890def");

        assert_eq!(path, root.join("abc123456789-node-20.11.0"));
    }
}
