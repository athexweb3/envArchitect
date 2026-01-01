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
#[async_trait]
pub trait PluginHandler: Send + Sync {
    /// 1. Validation Logic
    /// Validate the manifest configuration. Return a list of error messages.
    async fn validate(&self, _manifest: &serde_json::Value) -> Result<Vec<String>> {
        Ok(vec![])
    }

    /// 2. Resolution Logic
    /// Transforms current context into an execution plan.
    /// Returns the plan and an optional opaque state to pass to the install phase.
    async fn resolve(&self, ctx: &ResolutionContext) -> Result<(InstallPlan, Option<String>)>;

    /// 3. Installation Logic (Side Effects)
    /// Performs system changes like downloading binaries or symlinking.
    /// Receives the plan and the opaque state returned from the resolution phase.
    async fn install(&self, _plan: &InstallPlan, _state: Option<String>) -> Result<()> {
        Ok(())
    }

    /// Optional: Verify that the environment is healthy
    async fn verify(&self, _ctx: &ResolutionContext) -> Result<ValidationResult> {
        Ok(ValidationResult::new())
    }
}
