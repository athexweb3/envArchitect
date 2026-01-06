use env_architect_sdk::prelude::*;
use serde_json::Value;

/// A basic example of an EnvArchitect plugin.
/// This plugin simulates installing a tool called "hello-world".
// Register the plugin entry point
#[plugin]
#[derive(Default)]
struct BasicPlugin;

#[async_trait]
impl PluginHandler for BasicPlugin {
    type Config = Value;
    const CONFIG_KEY: &'static str = "hello-world";

    /// 1. Validation Logic
    /// Checks if the configuration is valid.
    async fn validate(&self, manifest: &Value) -> Result<Vec<String>> {
        let mut errors = Vec::new();

        // Example: Check if "version" is a string if present
        if let Some(config) = manifest.get("hello-world") {
            if !config.is_string() {
                errors.push("hello-world config must be a version string".to_string());
            }
        }

        Ok(errors)
    }

    /// 2. Resolution Logic
    /// Determines what needs to be done.
    async fn resolve(
        &self,
        _ctx: &env_architect_sdk::ResolutionContext,
        _config: Self::Config,
    ) -> Result<(InstallPlan, Option<String>)> {
        // Use the Builder to construct the manifest
        // Example logic:
        // let manifest = EnvBuilder::from_context(ctx)?
        //    .project("hello-world-project", "1.0.0")
        //    .add_dependency("hello-cli", "1.2.3")
        //    .build();

        // Create an installation plan
        // In a real plugin, you would generate this based on the context
        let plan = InstallPlan::default();

        // Pass some state to the install phase (e.g., the resolved version)
        let state = Some("v1.2.3".to_string());

        Ok((plan, state))
    }

    /// 3. Installation Logic
    /// Performs the actual side effects (downloading, installing).
    async fn install(&self, _plan: &InstallPlan, state: Option<String>) -> Result<()> {
        let version = state.unwrap_or_default();

        // Use the Host capabilities to interact with the system
        // Note: In a real plugin, you would download files here.
        host::info(&format!("Installing hello-world version: {}", version));

        Ok(())
    }
}

fn main() {
    // This example is meant to be compiled as a reactor, but if run as bin it does nothing
    println!("This example is a plugin reactor. Build with: cargo build --target wasm32-wasip1 --example basic_plugin");
}
