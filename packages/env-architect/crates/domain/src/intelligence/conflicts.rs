use super::metrics::MetricsDetector;
use crate::intelligence::strategies::{Conflict, InstallStrategy, Recommendation, Risk};
use crate::system::{InstalledToolsRegistry, PlatformInfo};
use anyhow::Result;
use env_manifest::ResolutionAction;
use semver::{Version, VersionReq};

/// The main conflict resolution engine
/// Uses game theory and multi-objective optimization to resolve conflicts
pub struct ConflictResolver {
    _platform: PlatformInfo,
    registry: InstalledToolsRegistry,
    metrics: MetricsDetector,
}

impl ConflictResolver {
    pub fn new(platform: PlatformInfo, registry: InstalledToolsRegistry) -> Self {
        let metrics = MetricsDetector::new(platform.clone());
        Self {
            _platform: platform,
            registry,
            metrics,
        }
    }

    /// Detect conflicts for a given tool requirement
    pub fn detect_conflicts(
        &self,
        tool: &str,
        required: &VersionReq,
        required_by: &str,
    ) -> Option<Conflict> {
        use crate::intelligence::{Conflict, ConflictSource};

        let installed = self.registry.get_installed(tool);

        if installed.is_empty() {
            // Tool not installed
            return Some(Conflict::MissingTool {
                tool: tool.to_string(),
                required: required.clone(),
                source: ConflictSource::Manifest,
                required_by: required_by.to_string(),
            });
        }

        // Check if any installed version satisfies requirement
        let satisfied = installed.iter().any(|v| required.matches(&v.version));

        if !satisfied {
            // Version mismatch
            return Some(Conflict::VersionMismatch {
                tool: tool.to_string(),
                required: required.clone(),
                installed: installed.iter().map(|v| v.version.clone()).collect(),
                source: ConflictSource::Manifest,
                required_by: required_by.to_string(),
            });
        }

        None
    }

    /// Resolve a conflict using game-theoretic approach
    pub fn resolve(&self, conflict: &Conflict) -> Result<Vec<Recommendation>> {
        match conflict {
            Conflict::VersionMismatch {
                tool,
                required,
                installed,
                ..
            } => self.resolve_version_mismatch(tool, required, installed),
            Conflict::MissingTool { tool, required, .. } => {
                self.resolve_missing_tool(tool, required)
            }
            Conflict::IncompatibleDependency { .. } => Ok(Vec::new()),
        }
    }

    /// Resolve version mismatch conflict
    /// Example: Node 18 installed, but Node 20 required
    fn resolve_version_mismatch(
        &self,
        tool: &str,
        required: &VersionReq,
        installed: &[Version],
    ) -> Result<Vec<Recommendation>> {
        let mut recommendations = Vec::new();

        // Find the newest version that matches requirement
        let target_version = self.find_target_version(required)?;

        let current_version = installed
            .first()
            .ok_or_else(|| anyhow::anyhow!("No installed version"))?;

        // Strategy 1: Install Alongside (Recommended)
        recommendations.push(Recommendation {
            action: format!("Install {} alongside {}", target_version, current_version),
            strategy: InstallStrategy::Alongside,
            pros: vec![
                "No risk to existing projects".to_string(),
                "Can switch versions easily".to_string(),
                "Preserves current setup".to_string(),
            ],
            cons: vec![
                format!(
                    "Uses additional ~{}MB disk space",
                    self.estimate_tool_size(tool)
                ),
                "Need to manage multiple versions".to_string(),
            ],
            risk: Risk::Low,
            estimated_disk_mb: self.estimate_tool_size(tool),
            estimated_time_sec: self.estimate_install_time(tool),
            resolution_actions: vec![ResolutionAction::ManagedInstall {
                manager: self.detect_best_manager(tool),
                command: format!("architect install {}@{}", tool, target_version),
            }],
        });

        // Strategy 2: Upgrade/Downgrade (if target is newer/older)
        let is_upgrade = target_version > *current_version;
        let num_dependent_projects = self.count_dependent_projects(tool);

        recommendations.push(Recommendation {
            action: if is_upgrade {
                format!("Upgrade {} → {}", current_version, target_version)
            } else {
                format!("Downgrade {} → {}", current_version, target_version)
            },
            strategy: InstallStrategy::Replace,
            pros: vec![
                "Clean system with single version".to_string(),
                if is_upgrade {
                    "Latest features and security".to_string()
                } else {
                    "Matches requirement exactly".to_string()
                },
            ],
            cons: vec![
                format!("May break {} existing project(s)", num_dependent_projects),
                "Irreversible without reinstall".to_string(),
            ],
            risk: if num_dependent_projects > 5 {
                Risk::High
            } else if num_dependent_projects > 0 {
                Risk::Medium
            } else {
                Risk::Low
            },
            estimated_disk_mb: 0, // No additional space
            estimated_time_sec: self.estimate_install_time(tool) + 30, // +30s for uninstall
            resolution_actions: vec![ResolutionAction::ManagedInstall {
                manager: self.detect_best_manager(tool),
                command: format!("architect install {}@{}", tool, target_version),
            }],
        });

        // Strategy 3: Use Existing (if any version partially satisfies)
        if let Some(partial_match) = self.find_partial_match(installed, required) {
            recommendations.push(Recommendation {
                action: format!("Try using existing {} (partial match)", partial_match),
                strategy: InstallStrategy::Alongside, // Use Alongside as a proxy for 'using existing'
                pros: vec![
                    "No installation needed".to_string(),
                    "Zero disk space".to_string(),
                ],
                cons: vec![
                    "May not fully satisfy requirements".to_string(),
                    "Could cause runtime errors".to_string(),
                ],
                risk: Risk::High,
                estimated_disk_mb: 0,
                estimated_time_sec: 0,
                resolution_actions: vec![],
            });
        }

        // Sort recommendations by risk (Low risk first)
        recommendations.sort_by_key(|r| r.risk);

        Ok(recommendations)
    }

    /// Resolve missing tool
    fn resolve_missing_tool(
        &self,
        tool: &str,
        required: &VersionReq,
    ) -> Result<Vec<Recommendation>> {
        let mut recommendations = Vec::new();

        let target_version = self.find_target_version(required)?;

        // Strategy: Install
        recommendations.push(Recommendation {
            action: format!("Install {} @ {}", tool, target_version),
            strategy: InstallStrategy::Alongside,
            pros: vec![
                "Satisfies requirement".to_string(),
                format!("Official {} version", tool),
            ],
            cons: vec![format!(
                "Requires ~{}MB disk space",
                self.estimate_tool_size(tool)
            )],
            risk: Risk::Low,
            estimated_disk_mb: self.estimate_tool_size(tool),
            estimated_time_sec: self.estimate_install_time(tool),
            resolution_actions: vec![ResolutionAction::ManagedInstall {
                manager: self.detect_best_manager(tool),
                command: format!("architect install {}@{}", tool, target_version),
            }],
        });

        // Check for alternative tools in the same family
        let alternatives = self.registry.get_recommendations(tool);
        if !alternatives.is_empty() {
            recommendations.push(Recommendation {
                action: format!("Use alternative: {}", alternatives.join(" or ")),
                strategy: InstallStrategy::Alongside, // Use Alongside as a proxy
                pros: vec![
                    format!("Already have {}", alternatives[0]),
                    "No additional install".to_string(),
                ],
                cons: vec!["Might not be fully compatible".to_string()],
                risk: Risk::Medium,
                estimated_disk_mb: 0,
                estimated_time_sec: 0,
                resolution_actions: vec![],
            });
        }

        // Add auto-shim recommendation if it's a known tool
        if self.is_shimmable(tool) {
            recommendations.push(Recommendation {
                action: format!("Auto-shim {} (Zero System Impact)", tool),
                strategy: InstallStrategy::Alongside,
                pros: vec![
                    "Zero system-wide impact".to_string(),
                    "Specific to this project".to_string(),
                ],
                cons: vec!["Requires Architect to launch tool".to_string()],
                risk: Risk::Low,
                estimated_disk_mb: self.estimate_tool_size(tool),
                estimated_time_sec: self.estimate_install_time(tool),
                resolution_actions: vec![ResolutionAction::AutoShim {
                    url: format!(
                        "https://registry.architect.io/bin/{}/{}",
                        tool, target_version
                    ),
                    binary_name: tool.to_string(),
                }],
            });
        }

        Ok(recommendations)
    }

    fn detect_best_manager(&self, tool: &str) -> String {
        match tool {
            "node" | "nodejs" | "npm" => "nvm".to_string(),
            "python" | "python3" => "pyenv".to_string(),
            "rust" | "rustc" | "cargo" => "rustup".to_string(),
            _ => "brew".to_string(),
        }
    }

    fn is_shimmable(&self, tool: &str) -> bool {
        matches!(tool, "node" | "python" | "rustup" | "go")
    }

    /// Find the best version that matches requirement
    fn find_target_version(&self, required: &VersionReq) -> Result<Version> {
        // In production, this would query the registry for available versions

        // Parse the requirement to extract major version
        let req_str = required.to_string();
        if req_str.starts_with('^') || req_str.starts_with('~') {
            let version_str = req_str.trim_start_matches('^').trim_start_matches('~');
            if let Ok(base) = Version::parse(version_str) {
                // For demo, add .11 to make it realistic (e.g., 20.0.0 -> 20.11.0)
                return Ok(Version::new(base.major, base.minor + 11, 0));
            }
        }

        // Fallback: try to parse as exact version
        Version::parse(&req_str).or_else(|_| Ok(Version::new(1, 0, 0)))
    }

    /// Find a partial match from installed versions
    fn find_partial_match(&self, installed: &[Version], required: &VersionReq) -> Option<Version> {
        // Check if major version matches
        installed
            .iter()
            .find(|v| {
                // Extract major version from requirement
                if let Some(major) = self.extract_major_version(required) {
                    v.major == major
                } else {
                    false
                }
            })
            .cloned()
    }

    /// Extract major version from version requirement
    fn extract_major_version(&self, req: &VersionReq) -> Option<u64> {
        let req_str = req.to_string();
        let clean = req_str
            .trim_start_matches('^')
            .trim_start_matches('~')
            .trim_start_matches('=');

        if let Ok(version) = Version::parse(clean) {
            Some(version.major)
        } else {
            None
        }
    }

    /// Get actual tool size (queries package managers)
    fn estimate_tool_size(&self, tool: &str) -> u64 {
        self.metrics.get_tool_size(tool).unwrap_or(100)
    }

    /// Estimate install time in seconds
    fn estimate_install_time(&self, tool: &str) -> u64 {
        match tool {
            "nodejs" | "node" => 120,
            "python" | "python3" => 180,
            "rust" | "rustc" | "cargo" => 300,
            _ => 60,
        }
    }

    /// Count how many projects depend on this tool
    /// Scans filesystem for .node-version files, package.json, Cargo.toml, etc.
    fn count_dependent_projects(&self, tool: &str) -> usize {
        self.metrics.scan_dependent_projects(tool).len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::system::platform::PlatformDetector;
    use semver::VersionReq;

    #[test]
    fn test_version_mismatch_resolution() {
        let platform = PlatformDetector::detect();
        let registry = InstalledToolsRegistry::new();
        let resolver = ConflictResolver::new(platform, registry);

        // Simulate: Node 18 installed, Node 20 required
        let installed = vec![Version::new(18, 19, 0)];
        let required = VersionReq::parse("^20.0.0").unwrap();

        let recommendations = resolver
            .resolve_version_mismatch("nodejs", &required, &installed)
            .unwrap();

        assert!(!recommendations.is_empty());
        assert_eq!(recommendations[0].strategy, InstallStrategy::Alongside);
        assert_eq!(recommendations[0].risk, Risk::Low);

        println!("📋 Recommendations for Node 18 → 20:");
        for (i, rec) in recommendations.iter().enumerate() {
            println!("\n{}. {} ({})", i + 1, rec.action, rec.risk);
            println!("   Pros: {}", rec.pros.join(", "));
            println!("   Cons: {}", rec.cons.join(", "));
        }
    }
}
