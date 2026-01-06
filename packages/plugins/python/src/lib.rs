use env_architect_sdk::host;
use env_architect_sdk::plugin;
use env_architect_sdk::prelude::*;
use serde_json::Value;

#[plugin]
#[derive(Default)]
struct PythonPlugin;

#[derive(serde::Deserialize, Default)]
struct PythonConfig {
    version: Option<String>,
}

#[async_trait]
impl Plugin for PythonPlugin {
    type Config = PythonConfig;
    const CONFIG_KEY: &'static str = "python";

    async fn resolve(
        &self,
        _context: &env_architect_sdk::ResolutionContext,
        config: Self::Config,
    ) -> Result<(InstallPlan, Option<String>)> {
        let mut plan = InstallPlan::default();
        let mut version_req = "3.11".to_string(); // Default

        // 0. Explicit Configuration
        if let Some(v) = config.version {
            version_req = v;
        }

        // 1. Try to read .python-version
        if let Ok(content) = host::read_file(".python-version") {
            version_req = content.trim().to_string();
        } else if let Ok(content) = host::read_file("runtime.txt") {
            // Heroku style "python-3.11.0"
            if content.starts_with("python-") {
                version_req = content.trim().trim_start_matches("python-").to_string();
            }
        } else if let Ok(content) = host::read_file("Pipfile") {
            // Very naive check for now
            if content.contains("python_version =") {
                // Parsing TODO
            }
        }

        if let Ok(content) = host::read_file("pyproject.toml") {
            if let Ok(value) = content.parse::<toml::Value>() {
                if let Some(requires) = value.get("project").and_then(|p| p.get("requires-python"))
                {
                    if let Some(v) = requires.as_str() {
                        version_req = v.to_string();
                    }
                }
            }
        }

        // 2. Set environment requirements
        plan.manifest
            .env
            .insert("python".to_string(), version_req.clone());

        // 3. Determine Installation Strategy
        // Recommendation: use a manageable python
        plan.instructions
            .push(format!("echo 'Python {} required.'", version_req));

        if host::read_file("requirements.txt").is_ok() {
            plan.instructions
                .push("pip install -r requirements.txt".to_string());
        } else if host::read_file("Pipfile").is_ok() {
            plan.instructions.push("pipenv install".to_string());
        } else if host::read_file("pyproject.toml").is_ok() {
            plan.instructions.push("pip install .".to_string());
        }

        Ok((plan, Some("python".to_string())))
    }

    async fn validate(&self, _manifest: &Value) -> Result<Vec<String>> {
        Ok(vec![])
    }
}
