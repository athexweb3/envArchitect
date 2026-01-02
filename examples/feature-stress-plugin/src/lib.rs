use anyhow::{Context, Result};
use env_architect_sdk::prelude::*;

#[env_architect_sdk::plugin]
#[derive(Default)]
struct StressPlugin;

#[env_architect_sdk::async_trait]
impl PluginHandler for StressPlugin {
    async fn validate(&self, manifest: &serde_json::Value) -> Result<Vec<String>> {
        host_ui::info("Stress Test: Validating all sections...");
        let mut errors = Vec::new();

        if manifest.get("project").is_none() {
            errors.push("Project section missing".to_string());
        }

        Ok(errors)
    }

    async fn resolve(
        &self,
        context: &env_architect_sdk::ResolutionContext,
    ) -> Result<(InstallPlan, Option<String>)> {
        host_ui::info(format!(
            "Stress Test: Resolving for {}/{}...",
            context.target_os, context.target_arch
        ));

        // 1. System Tool Discovery (V2 Feature)
        if let Some(node_versions) = context.system_tools.get("node") {
            host_ui::info(format!(
                "Found {} node versions on host: {:?}",
                node_versions.len(),
                node_versions
            ));
        } else {
            host_ui::warn("Node.js not detected on host system.");
        }

        // 2. Text Input & Spinner
        let spinner = host_ui::spinner("Starting deep resolution...");
        let custom_name = host_ui::input(
            "Enter a custom environment name",
            Some("stress-env".to_string()),
        );
        spinner.set_message("Initializing builder...");

        let mut builder = EnvBuilder::from_context(context)
            .context("Failed to load from context")?
            .project(&custom_name, "1.0.0");

        // 3. Select & Multi-step Logic
        let db_type = host_ui::select(
            "Select database provider",
            &["postgres", "mysql", "sqlite"],
            Some("postgres".to_string()),
        );
        builder = builder.add_dependency(&format!("{}-client", db_type), ">=14.0.0");

        // 4. Confirm & Conditional Builder Calls
        if host_ui::confirm("Enable advanced monitoring?", true) {
            builder = builder.add_dev_dependency("monitoring-agent", "^2.1.0");
            builder = builder.service("monitor", ServiceDef::new("monitoring-agent --port 9090"));
        }

        // 5. Intelligence & Resolution Actions (New V2 Feature)
        if context.system_tools.get("node").is_none() {
            builder = builder.resolution_action(ResolutionAction::ManagedInstall {
                manager: "nvm".to_string(),
                command: "nvm install 20".to_string(),
            });
            builder = builder.resolution_action(ResolutionAction::ManualPrompt {
                message: "Node.js is missing.".to_string(),
                instructions: "Please install Node.js v20+ manually or via nvm.".to_string(),
            });
        }

        // 6. Secret & Capability Check
        let api_key = host_ui::secret("Enter sensitive API token");
        if !api_key.is_empty() {
            host_ui::success("API Token received and validated.");
        }

        // 7. Conflict Management & Platform Constraints
        builder = builder
            .conflict("legacy-tool", "Incompatible with Architect 2.0")
            .support_platform("macos", "aarch64")
            .support_platform("linux", "x86_64")
            .capability(Capability::Network(vec!["connect".to_string()]));

        spinner.finish();
        host_ui::success("Stress Test: Resolution complete!");

        Ok((
            InstallPlan::new(builder.build()),
            Some(format!("db={}", db_type)),
        ))
    }

    async fn install(&self, plan: &InstallPlan, state: Option<String>) -> Result<()> {
        host_ui::info("Stress Test: Running installation side-effects...");

        if let Some(s) = state {
            host_ui::info(format!("Restoring state: {}", s));
        }

        // 1. Env Var Host Function
        if let Some(architect_home) = host_ui::get_env("ARCHITECT_HOME") {
            host_ui::info(format!("ARCHITECT_HOME is set: {}", architect_home));
        }

        // 2. File System Host Function
        match host_ui::read_file("env.json") {
            Ok(_) => host_ui::success("Successfully read env.json via host."),
            Err(e) => host_ui::error(format!("Failed to read env.json: {}", e)),
        }

        host_ui::success(format!(
            "Installation of {} dependencies complete.",
            plan.manifest.dependencies.len()
        ));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "current_thread")]
    async fn test_v2_features_in_stress_plugin() -> Result<()> {
        let plugin = StressPlugin::default();
        let runner = TestRunner::new(plugin);

        // 1. Mock System Tools (V2 Feature)
        let context = ResolutionContext::new("macos", "aarch64", "/tmp");
        let mut context_with_node = context.clone();
        context_with_node
            .system_tools
            .insert("node".to_string(), vec!["20.11.0".to_string()]);

        // 2. Mock UI
        runner
            .host
            .mock_input("Enter a custom environment name", "test-env");
        runner
            .host
            .mock_select("Select database provider", "postgres");
        runner
            .host
            .mock_confirm("Enable advanced monitoring?", false);
        runner
            .host
            .mock_secret("Enter sensitive API token", "masked-token");

        let (plan, _state) = runner.resolve(&context_with_node).await?;

        // 3. Verify System Tools Logic
        assert!(runner
            .host
            .get_logs()
            .iter()
            .any(|(_, m)| m.contains("Found 1 node versions")));

        // 4. Verify Manifest Integrity
        assert_eq!(plan.manifest.project.name, "test-env");
        assert!(plan.manifest.dependencies.contains_key("postgres-client"));

        // 5. Test "Node Missing" Logic (Simulate no node)
        let (plan_no_node, _) = runner.resolve(&context).await?;

        let intel = plan_no_node.manifest.intelligence.as_ref().unwrap();
        assert!(intel
            .proposed_actions
            .iter()
            .any(|a| matches!(a, ResolutionAction::ManagedInstall { .. })));

        Ok(())
    }
}
