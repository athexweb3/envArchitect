use crate::api::context::ResolutionContext;
use crate::api::types::InstallPlan;
use crate::contract::reexports::ValidationResult;
use anyhow::Result;
use async_trait::async_trait;

/// A specialized UI interface for plugins.
/// This is "Host-Mediated", meaning the plugin requests a UI element,
/// and the Host (CLI) creates it. This ensures security (no raw terminal access)
/// and CI/CD compatibility (auto-answers).
#[async_trait]
pub trait HostUI: Send + Sync {
    /// Display an informational message
    fn info(&self, msg: &str);

    /// Display a success message
    fn success(&self, msg: &str);

    /// Display an error message
    fn error(&self, msg: &str);

    /// Ask the user a yes/no question
    async fn confirm(&self, prompt: &str, default: bool) -> Result<bool>;

    /// Ask the user to select from a list of options
    async fn select(&self, prompt: &str, options: &[&str], default: Option<&str>)
        -> Result<String>;

    /// Ask the user for text input
    async fn input(&self, prompt: &str, default: Option<&str>) -> Result<String>;

    /// Ask the user for a secret (masked input)
    async fn secret(&self, prompt: &str) -> Result<String>;

    /// Start a spinner that stops when the returned guard is dropped
    fn spinner(&self, msg: &str) -> Box<dyn Spinner>;
}

pub trait Spinner: Send + Sync {
    fn set_message(&self, msg: &str);
    fn finish(&self);
}

// Default no-op spinner for testing
struct NoOpSpinner;
impl Spinner for NoOpSpinner {
    fn set_message(&self, _msg: &str) {}
    fn finish(&self) {}
}

/// A no-op UI implementation for testing or fallback
pub struct NoOpUI;
#[async_trait]
impl HostUI for NoOpUI {
    fn info(&self, _msg: &str) {}
    fn success(&self, _msg: &str) {}
    fn error(&self, _msg: &str) {}
    async fn confirm(&self, _prompt: &str, default: bool) -> Result<bool> {
        Ok(default)
    }
    async fn select(
        &self,
        _prompt: &str,
        options: &[&str],
        default: Option<&str>,
    ) -> Result<String> {
        Ok(default
            .unwrap_or(options.first().unwrap_or(&""))
            .to_string())
    }
    async fn input(&self, _prompt: &str, default: Option<&str>) -> Result<String> {
        Ok(default.unwrap_or("").to_string())
    }
    async fn secret(&self, _prompt: &str) -> Result<String> {
        Ok("".to_string())
    }
    fn spinner(&self, _msg: &str) -> Box<dyn Spinner> {
        Box::new(NoOpSpinner)
    }
}

/// The core logic trait for an environment plugin.
///
/// Implementors of this trait define how a plugin validates configuration,
/// resolves execution plans, and performs installation steps.
#[async_trait]
#[async_trait]
pub trait PluginHandler: Send + Sync {
    /// The configuration structure for this plugin.
    /// It must be deserializable from JSON/TOML and implement Default.
    /// If no configuration is needed, use `()`.
    type Config: serde::de::DeserializeOwned + Default + Send + Sync;

    /// The configuration key to look for (e.g., "node", "python").
    /// If empty or not found, the default configuration will be used.
    const CONFIG_KEY: &'static str;

    /// Validates the manifest configuration.
    async fn validate(&self, _manifest: &serde_json::Value) -> Result<Vec<String>> {
        Ok(vec![])
    }

    /// Resolves the current context into an execution plan.
    /// The context and the parsed configuration are provided.
    async fn resolve(
        &self,
        ctx: &ResolutionContext,
        config: Self::Config,
    ) -> Result<(InstallPlan, Option<String>)>;

    /// Executes the installation plan.
    async fn install(&self, _plan: &InstallPlan, _state: Option<String>) -> Result<()> {
        Ok(())
    }

    /// Verifies that the environment is healthy after installation.
    async fn verify(&self, _ctx: &ResolutionContext) -> Result<ValidationResult> {
        Ok(ValidationResult::new())
    }
}
