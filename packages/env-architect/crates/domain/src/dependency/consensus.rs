use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Represents a pinned environment state for collaborative consensus
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Lockfile {
    pub project_name: String,
    pub versions: HashMap<String, PinnedVersion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinnedVersion {
    pub version: String,
    pub content_hash: String,
    pub verified_by: String, // Sigstore OIDC identity
}

pub struct ConsensusEngine;

impl ConsensusEngine {
    /// Detect drift between the project's lockfile and the local store
    pub fn detect_drift(lockfile: &Lockfile, local_store_tools: &[String]) -> Vec<Drift> {
        let mut drifts = Vec::new();

        for (tool, pinned) in &lockfile.versions {
            if !local_store_tools.contains(tool) {
                drifts.push(Drift::MissingTool(tool.clone()));
            } else {
                // In production, we'd check if the store has the EXACT hash
                // For prototype, we simulate a version mismatch
                if pinned.version == "20.10.0" {
                    // Mock mismatch
                    drifts.push(Drift::VersionMismatch {
                        tool: tool.clone(),
                        expected: pinned.version.clone(),
                        actual: "20.11.0".to_string(),
                    });
                }
            }
        }

        drifts
    }

    /// Load a lockfile from a project root
    pub fn load_lockfile(project_root: &Path) -> Result<Lockfile> {
        let path = project_root.join("env.lock");
        if !path.exists() {
            return Ok(Lockfile::default());
        }
        let content = std::fs::read_to_string(&path)?;
        serde_json::from_str(&content).context("Failed to parse env.lock")
    }

    /// Save a lockfile to a project root
    pub fn save_lockfile(project_root: &Path, lockfile: &Lockfile) -> Result<()> {
        let path = project_root.join("env.lock");
        let content = serde_json::to_string_pretty(lockfile)?;
        std::fs::write(path, content).context("Failed to write env.lock")
    }
}

pub enum Drift {
    MissingTool(String),
    VersionMismatch {
        tool: String,
        expected: String,
        actual: String,
    },
}

impl Drift {
    pub fn description(&self) -> String {
        match self {
            Drift::MissingTool(t) => format!("Tool '{}' is missing from local store", t),
            Drift::VersionMismatch {
                tool,
                expected,
                actual,
            } => format!(
                "Tool '{}' version drift: Team expects {}, but you have {}",
                tool, expected, actual
            ),
        }
    }
}
