use env_manifest::{ResolutionAction, ServiceDef};
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Source of a conflict
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConflictSource {
    Manifest,    // From env.toml
    Plugin,      // From plugin dependency
    System,      // From OS requirements
    UserRequest, // Direct user command
}

impl fmt::Display for ConflictSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConflictSource::Manifest => write!(f, "env.toml"),
            ConflictSource::Plugin => write!(f, "Plugin Dependency"),
            ConflictSource::System => write!(f, "System Requirement"),
            ConflictSource::UserRequest => write!(f, "User Command"),
        }
    }
}

/// Type of conflict detected
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Conflict {
    /// Version mismatch: required version doesn't match installed
    VersionMismatch {
        tool: String,
        required: VersionReq,
        installed: Vec<Version>,
        source: ConflictSource,
        required_by: String, // What package requires this
    },

    /// Tool is missing entirely
    MissingTool {
        tool: String,
        required: VersionReq,
        source: ConflictSource,
        required_by: String,
    },

    /// Incompatible dependency (e.g., Rust package needs newer rustc)
    IncompatibleDependency {
        parent: String,
        child: String,
        reason: String,
        source: ConflictSource,
    },
}

impl fmt::Display for Conflict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Conflict::VersionMismatch {
                tool,
                required,
                installed,
                source,
                required_by,
            } => {
                write!(
                    f,
                    "Version Mismatch: {} requires {} @ {}, but ",
                    required_by, tool, required
                )?;
                if installed.is_empty() {
                    write!(f, "not installed")?;
                } else {
                    write!(
                        f,
                        "found {}",
                        installed
                            .iter()
                            .map(|v| v.to_string())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )?;
                }
                write!(f, " (from {})", source)
            }
            Conflict::MissingTool {
                tool,
                required,
                source,
                required_by,
            } => {
                write!(
                    f,
                    "Missing Tool: {} requires {} @ {} (from {})",
                    required_by, tool, required, source
                )
            }
            Conflict::IncompatibleDependency {
                parent,
                child,
                reason,
                source,
            } => {
                write!(
                    f,
                    "Incompatible: {} → {}: {} (from {})",
                    parent, child, reason, source
                )
            }
        }
    }
}

/// Installation strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstallStrategy {
    Alongside,   // Install without removing existing (Nix-style)
    Replace,     // Uninstall old, install new
    Link,        // Use existing from another manager
    UseExisting, // Keep current installation
}

impl fmt::Display for InstallStrategy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InstallStrategy::Alongside => write!(f, "Install alongside existing"),
            InstallStrategy::Replace => write!(f, "Replace existing version"),
            InstallStrategy::Link => write!(f, "Link to existing installation"),
            InstallStrategy::UseExisting => write!(f, "Use existing installation"),
        }
    }
}

/// Risk level of a resolution strategy
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Risk {
    Low,
    Medium,
    High,
}

impl fmt::Display for Risk {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Risk::Low => write!(f, "🟢 Low"),
            Risk::Medium => write!(f, "🟡 Medium"),
            Risk::High => write!(f, "🔴 High"),
        }
    }
}

/// A recommended resolution strategy with pros/cons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub action: String,
    pub strategy: InstallStrategy,
    pub pros: Vec<String>,
    pub cons: Vec<String>,
    pub risk: Risk,
    pub estimated_disk_mb: u64,
    pub estimated_time_sec: u64,
    /// Automated actions that can be taken to fulfill this recommendation
    #[serde(default)]
    pub resolution_actions: Vec<ResolutionAction>,
}

/// Resolution decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Resolution {
    Install {
        tool: String,
        version: Version,
        strategy: InstallStrategy,
    },
    Upgrade {
        tool: String,
        from: Version,
        to: Version,
        strategy: InstallStrategy,
    },
    Downgrade {
        tool: String,
        from: Version,
        to: Version,
        strategy: InstallStrategy,
    },
    UseExisting {
        tool: String,
        version: Version,
    },
    Skip {
        tool: String,
        reason: String,
    },
    Abort {
        reason: String,
    },
}

impl fmt::Display for Resolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Resolution::Install {
                tool,
                version,
                strategy,
            } => {
                write!(f, "Install {} @ {} ({})", tool, version, strategy)
            }
            Resolution::Upgrade {
                tool,
                from,
                to,
                strategy,
            } => {
                write!(f, "Upgrade {} from {} → {} ({})", tool, from, to, strategy)
            }
            Resolution::Downgrade {
                tool,
                from,
                to,
                strategy,
            } => {
                write!(
                    f,
                    "Downgrade {} from {} → {} ({})",
                    tool, from, to, strategy
                )
            }
            Resolution::UseExisting { tool, version } => {
                write!(f, "Use existing {} @ {}", tool, version)
            }
            Resolution::Skip { tool, reason } => {
                write!(f, "Skip {}: {}", tool, reason)
            }
            Resolution::Abort { reason } => {
                write!(f, "Abort: {}", reason)
            }
        }
    }
}

/// Strategy for resolving conflicts
pub trait ResolutionStrategy {
    fn name(&self) -> &str;
    fn can_resolve(&self, conflict: &Conflict) -> bool;
    fn recommend(&self, conflict: &Conflict) -> Vec<Recommendation>;
}
