pub use crate::api::builder::EnvBuilder;
pub use crate::api::context::ResolutionContext;
pub use crate::api::host; // Export 'host' directly so plugins can use host::exec
pub use crate::api::test::{MockHost, TestRunner};
pub use crate::api::traits::{HostUI, PluginHandler, Spinner};
// Alias for user friendliness
pub use crate::api::traits::PluginHandler as Plugin;
pub use crate::api::types::{InstallPlan, PackageMetadata};
pub use crate::contract::reexports::*;

// Common external constants/types
pub use anyhow::{Context, Result};
pub use async_trait::async_trait;
pub use std::collections::HashMap;
