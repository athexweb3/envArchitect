use anyhow::Result;
use env_architect_sdk::prelude::*;

#[env_architect_sdk::plugin]
#[derive(Default)]
struct MyPlugin;

#[env_architect_sdk::async_trait]
impl PluginHandler for MyPlugin {
    async fn validate(&self, manifest: &serde_json::Value) -> Result<Vec<String>> {
        host_ui::info("Validating manifest...");
        let mut errors = Vec::new();
        if manifest.get("project").is_none() {
            errors.push("Missing [project] section!".to_string());
        }
        Ok(errors)
    }

    async fn resolve(&self, context: &ResolutionContext) -> Result<(InstallPlan, Option<String>)> {
        // Automatically load project metadata from env.json
        let mut builder = EnvBuilder::from_context(context)?;

        host_ui::info("Simple Plugin: Resolving environment...");

        // Define the plugin struct UI for interactive resolution
        if host_ui::confirm("Include 'env-utils' package?", true) {
            let _token = host_ui::secret("Enter platform API token (Secure)");
            builder = builder.add_dependency("env-utils", "^0.5.0");
            host_ui::success("Dependency 'env-utils' added (Hot Reloaded!).");
        }

        Ok((
            InstallPlan::new(builder.build()),
            Some("my-cool-state".to_string()),
        ))
    }

    async fn install(&self, _plan: &InstallPlan, _state: Option<String>) -> Result<()> {
        host_ui::info("Installing assets...");
        if let Some(s) = _state {
            host_ui::info(format!("Received state: {}", s));
        }
        // Side effects would go here
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_plugin_resolution() -> Result<()> {
        let plugin = MyPlugin::default();
        let runner = TestRunner::new(plugin);

        let project_root = "/tmp/project";
        let env_path = PathBuf::from(project_root).join("env.json");

        // Mock host behavior
        runner
            .host
            .mock_confirm("Include 'env-utils' package?", true);

        runner.host.set_file(
            &env_path.to_string_lossy(),
            r#"{
            "project": {
                "name": "mock-project",
                "version": "1.2.3"
            }
        }"#,
        );

        let context = ResolutionContext::new("macos", "aarch64", project_root);
        let (plan, state) = runner.resolve(&context).await?;

        assert!(plan.manifest.dependencies.contains_key("env-utils"));
        assert_eq!(state, Some("my-cool-state".to_string()));

        Ok(())
    }
}
