use crate::system::{OsType, PlatformInfo};
use anyhow::Result;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

/// Metrics detector for real system data
pub struct MetricsDetector {
    platform: PlatformInfo,
}

#[derive(Debug, Deserialize)]
struct HomebrewBottle {
    _stable: Option<HomebrewBottleStable>,
}

#[derive(Debug, Deserialize)]
struct HomebrewBottleStable {
    _files: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct HomebrewInfo {
    _bottle: Option<HomebrewBottle>,
}

impl MetricsDetector {
    pub fn new(platform: PlatformInfo) -> Self {
        Self { platform }
    }

    /// Get actual tool size by querying package managers and registries
    pub fn get_tool_size(&self, tool: &str) -> Result<u64> {
        // First try language-specific registries (more accurate)
        if let Ok(size) = self.try_registry_apis(tool) {
            return Ok(size);
        }

        // Fall back to OS package managers
        match self.platform.os_type {
            OsType::MacOS => self.query_homebrew_size(tool),
            OsType::Linux => self.query_apt_size(tool),
            _ => Ok(self.fallback_estimate(tool)),
        }
    }

    /// Try language-specific package registry APIs
    fn try_registry_apis(&self, tool: &str) -> Result<u64> {
        match tool {
            // JavaScript ecosystem
            t if t.starts_with("@") || self.is_npm_package(t) => self.query_npm_registry(t),
            // Python ecosystem
            t if self.is_python_package(t) => self.query_pypi_registry(t),
            // Rust ecosystem
            t if self.is_rust_crate(t) => self.query_crates_io(t),
            _ => anyhow::bail!("No registry API available"),
        }
    }

    /// Check if this looks like an npm package
    fn is_npm_package(&self, name: &str) -> bool {
        // Common npm packages we might install
        matches!(
            name,
            "typescript" | "webpack" | "vite" | "eslint" | "prettier"
        )
    }

    /// Check if this is a Python package
    fn is_python_package(&self, name: &str) -> bool {
        matches!(name, "django" | "flask" | "requests" | "numpy" | "pandas")
    }

    /// Check if this is a Rust crate
    fn is_rust_crate(&self, name: &str) -> bool {
        matches!(name, "serde" | "tokio" | "actix-web" | "rocket")
    }

    /// Query npm registry for package size
    fn query_npm_registry(&self, package: &str) -> Result<u64> {
        // npm registry API: https://registry.npmjs.org/<package>
        // We'd use reqwest here in production
        // For now, return known sizes for common packages

        let size_kb = match package {
            "typescript" => 65_000, // ~65MB
            "webpack" => 5_000,     // ~5MB
            "vite" => 15_000,       // ~15MB
            "eslint" => 8_000,      // ~8MB
            "prettier" => 3_000,    // ~3MB
            "react" => 500,         // ~500KB
            "vue" => 3_000,         // ~3MB
            "express" => 200,       // ~200KB
            _ => anyhow::bail!("Unknown npm package"),
        };

        Ok(size_kb / 1024) // Convert to MB
    }

    /// Query PyPI for package size
    fn query_pypi_registry(&self, package: &str) -> Result<u64> {
        // PyPI API: https://pypi.org/pypi/<package>/json
        // Would parse urls[0].size from the response

        let size_kb = match package {
            "django" => 10_000,      // ~10MB
            "flask" => 700,          // ~700KB
            "requests" => 500,       // ~500KB
            "numpy" => 25_000,       // ~25MB
            "pandas" => 40_000,      // ~40MB
            "tensorflow" => 450_000, // ~450MB
            "pytorch" => 800_000,    // ~800MB
            _ => anyhow::bail!("Unknown PyPI package"),
        };

        Ok(size_kb / 1024) // Convert to MB
    }

    /// Query crates.io for crate size
    fn query_crates_io(&self, crate_name: &str) -> Result<u64> {
        // crates.io API: https://crates.io/api/v1/crates/<name>
        // Would parse crate.downloads and recent_downloads

        let size_kb = match crate_name {
            "serde" => 200,       // ~200KB
            "tokio" => 3_000,     // ~3MB
            "actix-web" => 2_000, // ~2MB
            "rocket" => 1_500,    // ~1.5MB
            "diesel" => 5_000,    // ~5MB
            _ => anyhow::bail!("Unknown crate"),
        };

        Ok(size_kb / 1024) // Convert to MB
    }

    /// Query Homebrew for package size (macOS)
    fn query_homebrew_size(&self, tool: &str) -> Result<u64> {
        // Try to get info from Homebrew
        let output = Command::new("brew")
            .args(&["info", "--json=v1", tool])
            .output();

        if let Ok(out) = output {
            if out.status.success() {
                let json_str = String::from_utf8_lossy(&out.stdout);
                if let Ok(infos) = serde_json::from_str::<Vec<HomebrewInfo>>(&json_str) {
                    if let Some(_info) = infos.first() {
                        // For now, use formula-based estimation
                        // In production, would parse bottle URL and query size
                        return Ok(self.fallback_estimate(tool));
                    }
                }
            }
        }

        // Fallback if Homebrew query fails
        Ok(self.fallback_estimate(tool))
    }

    /// Query APT for package size (Linux)
    fn query_apt_size(&self, tool: &str) -> Result<u64> {
        let output = Command::new("apt").args(&["show", tool]).output();

        if let Ok(out) = output {
            if out.status.success() {
                let text = String::from_utf8_lossy(&out.stdout);

                // Parse "Installed-Size: 12345" line
                for line in text.lines() {
                    if line.starts_with("Installed-Size:") {
                        if let Some(size_str) = line.split(':').nth(1) {
                            let size_kb: u64 = size_str.trim().parse().unwrap_or(0);
                            // Convert KB to MB
                            return Ok(size_kb / 1024);
                        }
                    }
                }
            }
        }

        Ok(self.fallback_estimate(tool))
    }

    /// Fallback size estimation
    fn fallback_estimate(&self, tool: &str) -> u64 {
        match tool {
            "nodejs" | "node" => 250,
            "python" | "python3" => 150,
            "rust" | "rustc" | "cargo" => 500,
            "go" => 200,
            "java" | "jdk" => 300,
            _ => 100,
        }
    }

    /// Scan filesystem for projects that depend on this tool
    pub fn scan_dependent_projects(&self, tool: &str) -> Vec<PathBuf> {
        let search_paths = self.get_common_project_dirs();
        let mut projects = Vec::new();

        for base_path in search_paths {
            if !base_path.exists() {
                continue;
            }

            // Walk directory tree with max depth 3
            for entry in WalkDir::new(&base_path)
                .max_depth(3)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.file_type().is_dir() {
                    if self.is_project_using_tool(entry.path(), tool) {
                        projects.push(entry.path().to_path_buf());
                    }
                }
            }
        }

        projects
    }

    /// Get common project directories to scan
    fn get_common_project_dirs(&self) -> Vec<PathBuf> {
        let mut dirs = Vec::new();

        if let Some(home) = dirs::home_dir() {
            dirs.push(home.join("Projects"));
            dirs.push(home.join("Developer"));
            dirs.push(home.join("Code"));
            dirs.push(home.join("dev"));
            dirs.push(home.join("workspace"));
        }

        dirs
    }

    /// Check if a project uses a specific tool
    fn is_project_using_tool(&self, path: &Path, tool: &str) -> bool {
        match tool {
            "node" | "nodejs" => self.is_nodejs_project(path),
            "python" | "python3" => self.is_python_project(path),
            "rust" | "rustc" | "cargo" => self.is_rust_project(path),
            "go" => path.join("go.mod").exists(),
            "ruby" => path.join("Gemfile").exists(),
            _ => false,
        }
    }

    /// Check if directory is a Node.js project
    fn is_nodejs_project(&self, path: &Path) -> bool {
        // Check for version files
        if path.join(".node-version").exists() || path.join(".nvmrc").exists() {
            return true;
        }

        // Check package.json with engines field
        if let Ok(content) = std::fs::read_to_string(path.join("package.json")) {
            if let Ok(pkg) = serde_json::from_str::<serde_json::Value>(&content) {
                if pkg.get("engines").and_then(|e| e.get("node")).is_some() {
                    return true;
                }
            }
        }

        false
    }

    /// Check if directory is a Python project
    fn is_python_project(&self, path: &Path) -> bool {
        path.join(".python-version").exists()
            || path.join("runtime.txt").exists()
            || path.join("Pipfile").exists()
            || path.join("pyproject.toml").exists()
    }

    /// Check if directory is a Rust project
    fn is_rust_project(&self, path: &Path) -> bool {
        path.join("Cargo.toml").exists() || path.join("rust-toolchain.toml").exists()
    }

    /// Get actual disk usage of a directory (like `du -sb`)
    #[cfg(unix)]
    pub fn get_disk_usage(&self, path: &Path) -> Result<u64> {
        use std::os::unix::fs::MetadataExt;

        let mut total_blocks = 0u64;

        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    // Get allocated blocks (512-byte blocks on Unix)
                    total_blocks += metadata.blocks();
                }
            }
        }

        // Convert 512-byte blocks to MB
        Ok((total_blocks * 512) / (1024 * 1024))
    }

    #[cfg(not(unix))]
    pub fn get_disk_usage(&self, path: &Path) -> Result<u64> {
        let mut total_size = 0u64;

        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_file() {
                    total_size += metadata.len();
                }
            }
        }

        // Convert bytes to MB
        Ok(total_size / (1024 * 1024))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::system::platform::PlatformDetector;

    #[test]
    fn test_detect_dependent_projects() {
        let platform = PlatformDetector::detect();
        let detector = MetricsDetector::new(platform);

        // Scan for Node.js projects
        let nodejs_projects = detector.scan_dependent_projects("nodejs");

        println!("Found {} Node.js projects:", nodejs_projects.len());
        for (i, project) in nodejs_projects.iter().take(5).enumerate() {
            println!("  {}. {}", i + 1, project.display());
        }

        // This test will find actual projects on the system
        // Don't assert specific counts as it depends on the machine
    }

    #[test]
    fn test_tool_size_detection() {
        let platform = PlatformDetector::detect();
        let detector = MetricsDetector::new(platform);

        let size = detector.get_tool_size("nodejs").unwrap();
        println!("Node.js estimated size: {}MB", size);

        assert!(size > 0);
        assert!(size < 1000); // Sanity check
    }
}
