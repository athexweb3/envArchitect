use anyhow::{Context, Result};
use semver::Version;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Command;

/// Which tool manager installed this tool
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToolManager {
    EnvArchitect,
    Homebrew,
    Apt,
    Yum,
    Dnf,
    Pacman,
    Cargo,
    Npm,
    System,
    Unknown,
}

/// An installed version of a tool
#[derive(Debug, Clone)]
pub struct InstalledVersion {
    pub tool: String,
    pub version: Version,
    pub location: PathBuf,
    pub managed_by: ToolManager,
}

/// Tool family - equivalent tools that can be used interchangeably
#[derive(Debug, Clone)]
pub struct ToolFamily {
    pub name: String,
    pub members: Vec<String>,
    pub preference_order: Vec<String>, // Ordered by preference
}

impl ToolFamily {
    /// Create a new tool family
    pub fn new(name: impl Into<String>, members: Vec<String>) -> Self {
        let preference_order = members.clone(); // By default, order is preference
        Self {
            name: name.into(),
            members,
            preference_order,
        }
    }

    /// Set custom preference order
    pub fn with_preference(mut self, order: Vec<String>) -> Self {
        self.preference_order = order;
        self
    }

    /// Check if a tool belongs to this family
    pub fn contains(&self, tool: &str) -> bool {
        self.members.iter().any(|m| m == tool)
    }

    /// Get the most preferred available tool from installed versions
    pub fn get_preferred<'a>(
        &self,
        installed: &'a [InstalledVersion],
    ) -> Option<&'a InstalledVersion> {
        for preferred in &self.preference_order {
            if let Some(version) = installed.iter().find(|v| &v.tool == preferred) {
                return Some(version);
            }
        }
        None
    }
}

/// Registry of all installed tools on the system
pub struct InstalledToolsRegistry {
    cache: HashMap<String, Vec<InstalledVersion>>,
    families: Vec<ToolFamily>,
}

impl InstalledToolsRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            cache: HashMap::new(),
            families: Vec::new(),
        };

        // Register known tool families
        registry.register_default_families();
        registry
    }

    /// Register default tool families
    fn register_default_families(&mut self) {
        // JavaScript package managers (prefer bun > pnpm > yarn > npm)
        self.families.push(
            ToolFamily::new(
                "js-package-manager",
                vec![
                    "npm".to_string(),
                    "yarn".to_string(),
                    "pnpm".to_string(),
                    "bun".to_string(),
                ],
            )
            .with_preference(vec![
                "bun".to_string(),
                "pnpm".to_string(),
                "yarn".to_string(),
                "npm".to_string(),
            ]),
        );

        // Python (prefer python3 > python)
        self.families.push(
            ToolFamily::new("python", vec!["python".to_string(), "python3".to_string()])
                .with_preference(vec!["python3".to_string(), "python".to_string()]),
        );

        // Node.js
        self.families.push(
            ToolFamily::new("nodejs", vec!["node".to_string(), "nodejs".to_string()])
                .with_preference(vec!["node".to_string(), "nodejs".to_string()]),
        );

        // Rust
        self.families.push(
            ToolFamily::new(
                "rust-compiler",
                vec!["rustc".to_string(), "cargo".to_string()],
            )
            .with_preference(vec!["cargo".to_string(), "rustc".to_string()]),
        );

        // Git
        self.families
            .push(ToolFamily::new("git", vec!["git".to_string()]));

        // Make
        self.families.push(
            ToolFamily::new("make", vec!["make".to_string(), "gmake".to_string()])
                .with_preference(vec!["make".to_string(), "gmake".to_string()]),
        );
    }

    /// Scan the system for installed tools (Strategy 1: PATH scanning)
    pub fn scan(&mut self) -> Result<()> {
        self.scan_path()?;
        Ok(())
    }

    /// Strategy 1: Scan PATH for executables
    fn scan_path(&mut self) -> Result<()> {
        // Get list of common tools to check
        let tools_to_check = self.get_common_tools();

        for tool in tools_to_check {
            if let Ok(version) = self.detect_via_path(&tool) {
                self.add_version(version);
            }
        }

        Ok(())
    }

    /// Get list of common tools from all families
    fn get_common_tools(&self) -> Vec<String> {
        let mut tools = Vec::new();
        for family in &self.families {
            tools.extend(family.members.clone());
        }
        tools.dedup();
        tools
    }

    /// Detect a tool via PATH
    fn detect_via_path(&self, tool: &str) -> Result<InstalledVersion> {
        // Try to find tool in PATH
        let which_output = Command::new("which")
            .arg(tool)
            .output()
            .context("Failed to run 'which' command")?;

        if !which_output.status.success() {
            anyhow::bail!("Tool '{}' not found in PATH", tool);
        }

        let location = String::from_utf8(which_output.stdout)?.trim().to_string();

        // Try to get version
        let version = self.get_tool_version(tool)?;

        Ok(InstalledVersion {
            tool: tool.to_string(),
            version,
            location: PathBuf::from(location),
            managed_by: ToolManager::Unknown, // We don't know yet
        })
    }

    /// Get version of a tool by running it with --version
    fn get_tool_version(&self, tool: &str) -> Result<Version> {
        let version_output = Command::new(tool)
            .arg("--version")
            .output()
            .context(format!("Failed to get version for '{}'", tool))?;

        if !version_output.status.success() {
            anyhow::bail!("Failed to get version for '{}'", tool);
        }

        let output = String::from_utf8(version_output.stdout)?;
        self.parse_version(&output)
    }

    /// Parse version string from tool output
    fn parse_version(&self, output: &str) -> Result<Version> {
        // Try to extract version number from output
        // Common patterns: "v1.2.3", "1.2.3", "tool 1.2.3"

        for line in output.lines() {
            // Try to find a version-looking string
            for word in line.split_whitespace() {
                let clean = word.trim_start_matches('v').trim_matches(',');
                if let Ok(version) = Version::parse(clean) {
                    return Ok(version);
                }

                // Try partial version (e.g., "1.2" -> "1.2.0")
                if let Some((_major, rest)) = clean.split_once('.') {
                    if let Some((_minor, _)) = rest.split_once('.') {
                        // Already has major.minor.patch
                        if let Ok(version) = Version::parse(clean) {
                            return Ok(version);
                        }
                    } else {
                        // Only major.minor, add .0
                        let full_version = format!("{}.0", clean);
                        if let Ok(version) = Version::parse(&full_version) {
                            return Ok(version);
                        }
                    }
                }
            }
        }

        anyhow::bail!("Could not parse version from output: {}", output)
    }

    /// Get all installed versions of a tool
    pub fn get_installed(&self, tool: &str) -> Vec<InstalledVersion> {
        self.cache.get(tool).cloned().unwrap_or_default()
    }

    /// Get tool family by name
    pub fn get_family(&self, family_name: &str) -> Option<&ToolFamily> {
        self.families.iter().find(|f| f.name == family_name)
    }

    /// Get all installed tools from a family
    pub fn get_family_installed(&self, family_name: &str) -> Vec<InstalledVersion> {
        if let Some(family) = self.get_family(family_name) {
            let mut all_versions = Vec::new();
            for member in &family.members {
                all_versions.extend(self.get_installed(member));
            }
            all_versions
        } else {
            Vec::new()
        }
    }

    /// Get the preferred tool from a family
    pub fn get_family_preferred(&self, family_name: &str) -> Option<InstalledVersion> {
        if let Some(family) = self.get_family(family_name) {
            let installed = self.get_family_installed(family_name);
            family.get_preferred(&installed).cloned()
        } else {
            None
        }
    }

    /// Get recommendations for alternatives if a tool is not installed
    pub fn get_recommendations(&self, tool: &str) -> Vec<String> {
        // Find which family this tool belongs to
        for family in &self.families {
            if family.contains(tool) {
                // Get all installed alternatives
                let installed = self.get_family_installed(&family.name);
                if !installed.is_empty() {
                    return installed.iter().map(|v| v.tool.clone()).collect();
                }

                // If nothing installed, suggest other family members
                return family
                    .members
                    .iter()
                    .filter(|m| *m != tool)
                    .cloned()
                    .collect();
            }
        }

        Vec::new()
    }

    /// Add a manually detected version
    pub fn add_version(&mut self, version: InstalledVersion) {
        self.cache
            .entry(version.tool.clone())
            .or_default()
            .push(version);
    }
}

impl Default for InstalledToolsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_families() {
        let registry = InstalledToolsRegistry::new();

        // Check JavaScript package manager family
        let js_family = registry.get_family("js-package-manager").unwrap();
        assert!(js_family.contains("npm"));
        assert!(js_family.contains("bun"));
        assert!(js_family.contains("yarn"));
        assert!(js_family.contains("pnpm"));

        // Check preference order (bun > pnpm > yarn > npm)
        assert_eq!(js_family.preference_order[0], "bun");
        assert_eq!(js_family.preference_order[1], "pnpm");
    }

    #[test]
    fn test_version_parsing() {
        let registry = InstalledToolsRegistry::new();

        // Test various version formats
        assert!(registry.parse_version("v1.2.3").is_ok());
        assert!(registry.parse_version("1.2.3").is_ok());
        assert!(registry.parse_version("Node.js v20.11.0").is_ok());
        assert!(registry.parse_version("rustc 1.75.0").is_ok());
    }
}
