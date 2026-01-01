use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Action taken to resolve a conflict
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ResolutionAction {
    /// Execute a command via a known manager (e.g., nvm, brew)
    ManagedInstall { manager: String, command: String },
    /// Download and shim a binary automatically
    AutoShim { url: String, binary_name: String },
    /// Modify project configuration
    ConfigUpdate { path: String, patch: String },
    /// Ask user to manually resolve
    ManualPrompt {
        message: String,
        instructions: String,
    },
}
