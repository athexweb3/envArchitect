//! # EnvArchitect SDK (Rust Core)
//!
//! This SDK provides the fundamental building blocks for creating Wasm-based plugins
//! for the EnvArchitect ecosystem. It abstracts the underlying `wit-bindgen` guest bindings
//! and exposes a type-safe, ergonomic API.
//!
//! ## Core Components
//!
//! *   **`PluginHandler` Trait**: The main entry point for your plugin logic. Implement this
//!     to define validation, resolution, and installation behavior.
//! *   **`EnvBuilder`**: A fluent builder pattern for constructing environment plans.
//! *   **`host` Module**: Access to host capabilities such as file system, network, and UI.
//!
//!
//! ## Plugin Lifecycle
//!
//! 1.  **Validate**: The host calls `validate` with the raw JSON configuration. The plugin checks for schema errors.
//! 2.  **Resolve**: The host calls `resolve` with the execution context (OS, Arch, etc.). The plugin returns an `InstallPlan`.
//! 3.  **Install**: The host calls `install` with the plan and any state passed from the resolve phase. This is where side effects occur.
//!
//! ## Example
//!
//! See `examples/basic_plugin.rs` for a complete runnable example.
//!
//! ```rust,no_run
//! use env_architect_sdk::prelude::*;
//!
//! struct MyPlugin;
//!
//! #[async_trait]
//! impl PluginHandler for MyPlugin {
//!     async fn resolve(&self, ctx: &ResolutionContext) -> Result<(InstallPlan, Option<String>)> {
//!         // 1. Build the manifest using the Fluent Builder
//!         let manifest = EnvBuilder::from_context(ctx)?
//!             .add_dependency("node", "18.x")
//!             .build();
//!
//!         // 2. Return the plan
//!         Ok((InstallPlan::default(), None))
//!     }
//! }
//!
//! // Register the plugin
//! plugin!(MyPlugin);
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
