use env_architect_sdk::{host, plugin, prelude::*};
use serde_json::Value;

#[plugin]
#[derive(Default)]
struct NodePlugin;

#[async_trait]

impl PluginHandler for NodePlugin {
    type Config = serde_json::Value; // Parse simply as JSON for now
    const CONFIG_KEY: &'static str = "node";

    async fn resolve(
        &self,
        _context: &env_architect_sdk::ResolutionContext,
        config: Self::Config,
    ) -> Result<(InstallPlan, Option<String>)> {
        let mut plan = InstallPlan::default();

        // 1. Try to read package.json engines / Manifest config
        let mut custom_state = "node".to_string();

        // Extract state from the passed config if available
        if let Some(state_val) = config.get("state").and_then(|v| v.as_str()) {
            custom_state = state_val.to_string();
        }

        // Logic moved to config extraction above

        // 0. Detect Current Version
        let nvm_dir = host::get_env("NVM_DIR").unwrap_or_else(|| "$HOME/.nvm".to_string());

        // Try to detect managed NVM version
        let check_cmd = format!(
            "export NVM_DIR=\"{}\"; [ -s \"$NVM_DIR/nvm.sh\" ] && . \"$NVM_DIR/nvm.sh\" && nvm current",
            nvm_dir
        );

        let current_version = match host::exec("bash", &["-c", &check_cmd]) {
            Ok(out) => {
                let trimmed = out.trim();
                if !trimmed.is_empty() && trimmed != "none" {
                    Some(trimmed.to_string())
                } else {
                    None
                }
            }
            Err(_e) => None,
        };

        let should_prompt = if let Some(ver) = current_version {
            // Found existing version
            host::confirm(
                &format!("Node.js {} is currently active. Change version?", ver),
                false,
            )
        } else {
            // No info, definitely prompt
            true
        };

        if !should_prompt {
            return Ok((plan, Some(custom_state)));
        }

        // 1. Interactive Version Selection
        let options = vec![
            "Stable (LTS) - Recommended",
            "Latest Features",
            "Specific Version...",
        ];

        // host::select returns the string of the selected option
        let selection = host::select(
            "Which Node.js version do you need?",
            &options,
            Some("Stable (LTS) - Recommended".to_string()),
        );

        let version_req = if selection.contains("Stable") {
            "lts/*".to_string()
        } else if selection.contains("Latest") {
            "node".to_string()
        } else {
            // "Specific Version..."
            let input_ver = host::input("Enter version (e.g. 18.16.0):", None);
            if input_ver.trim().is_empty() {
                "lts/*".to_string()
            } else {
                input_ver.trim().to_string()
            }
        };

        // 2. Conflict / Defaulting Strategy
        // host::confirm returns bool directly
        let make_default = host::confirm("Set as system default?", true);

        // 3. Construct NVM Execution Plan
        let nvm_dir = host::get_env("NVM_DIR").unwrap_or_else(|| "$HOME/.nvm".to_string());

        // Robust NVM Loading
        let source_nvm = format!(
            "export NVM_DIR=\"{}\"; [ -s \"$NVM_DIR/nvm.sh\" ] && \\. \"$NVM_DIR/nvm.sh\"",
            nvm_dir
        );

        let mut cmd_chain = format!("{}; nvm install {}", source_nvm, version_req);

        if make_default {
            cmd_chain.push_str(&format!("; nvm alias default {}", version_req));
            cmd_chain.push_str(&format!("; nvm use {}", version_req));
        }

        // Add instructions
        plan.instructions.push(cmd_chain);

        Ok((plan, Some(custom_state)))
    }

    async fn validate(&self, _manifest: &Value) -> Result<Vec<String>> {
        // Minimal validation for now
        Ok(vec![])
    }
}
