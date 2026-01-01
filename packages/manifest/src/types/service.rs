use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Definition of a background service.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub struct ServiceDef {
    /// The command to start the service.
    pub command: String,

    /// Restart policy for the service.
    #[serde(default)]
    pub restart: RestartPolicy,

    /// User to run the service as (default: current user).
    #[serde(default)]
    pub user: Option<String>,

    /// Environment variables specific to this service.
    #[serde(default)]
    pub env: HashMap<String, String>,
}

impl ServiceDef {
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            restart: RestartPolicy::default(),
            user: None,
            env: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum RestartPolicy {
    No,
    Always,
    OnFailure,
}

impl Default for RestartPolicy {
    fn default() -> Self {
        Self::OnFailure
    }
}
