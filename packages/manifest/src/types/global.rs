use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The global manifest file (~/.env-architect/global.env.toml).
/// This tracks all tools installed in "System Mode" (Global Context).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema, Default)]
pub struct GlobalManifest {
    /// List of globally installed tools.
    /// Key: Tool Name (e.g., "node", "python")
    #[serde(default)]
    pub tools: HashMap<String, GlobalTool>,

    /// Future: Track managed projects?
    #[serde(default)]
    pub projects: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct GlobalTool {
    /// The resolution string (e.g., "registry:python").
    pub source: String,

    /// The installed version.
    pub version: Option<String>,

    /// The verification signature.
    pub signature: Option<String>,

    /// When it was installed (ISO 8601 string).
    pub installed_at: Option<String>,
}
