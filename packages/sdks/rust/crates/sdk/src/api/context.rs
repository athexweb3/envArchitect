use crate::contract::reexports::EnhancedManifest;
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    /// Stores the capabilities allowed for the currently executing plugin lifecycle method.
    /// This is injected by the macro adapter before calling the user's code.
    pub static ACTIVE_CAPABILITIES: RefCell<Vec<String>> = RefCell::new(Vec::new());
}

pub fn set_active_capabilities(caps: Vec<String>) {
    ACTIVE_CAPABILITIES.with(|c| *c.borrow_mut() = caps);
}

pub fn check_capability(cap: &str) -> bool {
    ACTIVE_CAPABILITIES.with(|c| c.borrow().contains(&cap.to_string()))
}

/// Context provided to the environment package during resolution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolutionContext {
    /// The target operating system (e.g., "linux", "macos", "windows").
    pub target_os: String,

    /// The target CPU architecture (e.g., "x86_64", "aarch64").
    pub target_arch: String,

    /// The absolute path to the project root directory.
    pub project_root: String,

    /// Environment variables available to the plugin.
    pub env_vars: HashMap<String, String>,

    /// Capabilities explicitly granted to this plugin execution by the host.
    pub allowed_capabilities: Vec<crate::contract::reexports::Capability>,

    /// The manifest of the consumer project (if this plugin is being resolved as a dependency).
    /// *Coming soon.*
    pub parent_manifest: Option<Box<EnhancedManifest>>,

    /// A map of tools already installed in the environment (e.g., `{"node": ["18.16.0"]}`).
    #[serde(default)]
    pub system_tools: HashMap<String, Vec<String>>,
}

impl ResolutionContext {
    /// Creates a new resolution context.
    pub fn new(
        target_os: impl Into<String>,
        target_arch: impl Into<String>,
        project_root: impl Into<String>,
    ) -> Self {
        Self {
            target_os: target_os.into(),
            target_arch: target_arch.into(),
            project_root: project_root.into(),
            env_vars: HashMap::new(),
            allowed_capabilities: Vec::new(),
            parent_manifest: None,
            system_tools: HashMap::new(),
        }
    }
}
