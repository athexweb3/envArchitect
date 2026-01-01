use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A command to run.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum ScriptCommand {
    /// A single string command to run in a shell.
    Single(String),
    /// A sequence of commands to run (chain execution).
    Multiple(Vec<String>),
}

/// Hooks that run at specific points in the lifecycle.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default, JsonSchema)]
pub struct LifecycleHooks {
    pub pre_install: Option<String>,
    pub post_install: Option<String>,
    pub pre_activate: Option<String>,
    pub post_activate: Option<String>,
    pub pre_deactivate: Option<String>,
    pub post_deactivate: Option<String>,
}
