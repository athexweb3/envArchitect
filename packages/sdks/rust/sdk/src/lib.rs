//! # EnvArchitect SDK (Rust Core)
//!
//! This SDK provides the fundamental traits and types for creating environment packages
//! for EnvArchitect. It is designed to be a thin, type-safe wrapper around the core
//! domain logic, ensuring consistency between plugin behavior and the core engine.
//!
//! ## Architecture
//!
//! - **Contract Layer** (`contract`): Re-exports types from `domain` crate.
//! - **API Layer** (`api`): user-facing traits (`EnvPackage`) and builders (`EnvBuilder`).
//! - **Internal Layer** (`internal`): Hidden glue code (WIT bindings).
//!
//! ## Usage
//!
//! ```rust
//! use env_architect_sdk::prelude::*;
//! ```

pub mod api;
pub mod contract;
#[doc(hidden)]
pub mod internal;
pub mod prelude;

// Facade re-exports
pub use api::builder::EnvBuilder;
pub use api::context::ResolutionContext;
pub use api::host;
pub use api::traits::{HostUI, NoOpUI, PluginHandler, Spinner};
pub use api::types::{InstallPlan, PackageMetadata};
pub use async_trait::async_trait;
pub use env_architect_macros::plugin;
pub use futures;
pub use semver;
pub use serde_json;

// Re-export contract types for convenience
pub use api::test::{MockHost, TestRunner};
pub use contract::reexports::*;
pub use internal::bindings::env_architect::plugin::host::LogLevel;
