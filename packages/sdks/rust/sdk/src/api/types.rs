use crate::contract::reexports::EnhancedManifest;
use serde::{Deserialize, Serialize};

/// The result of an environment resolution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallPlan {
    /// The fully resolved manifest that the core engine should execute
    pub manifest: EnhancedManifest,

    /// Optional custom instructions
    pub instructions: Vec<String>,
}

impl InstallPlan {
    pub fn new(manifest: EnhancedManifest) -> Self {
        Self {
            manifest,
            instructions: Vec::new(),
        }
    }

    pub fn add_instruction(&mut self, instruction: &str) {
        self.instructions.push(instruction.to_string());
    }
}

/// Metadata about the environment package itself
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: Option<String>,
}
