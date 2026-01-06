mod detector;
// mod doctor; // Removed

use env_architect_sdk::plugin;
use env_architect_sdk::prelude::*;

#[plugin]
#[derive(Default)]
struct StandardPlugin;

#[derive(serde::Deserialize, Default)]
struct StandardConfig {
    auto_detect: Option<bool>,
}

#[async_trait]
impl Plugin for StandardPlugin {
    type Config = StandardConfig;
    const CONFIG_KEY: &'static str = "standard";

    async fn validate(&self, _manifest: &serde_json::Value) -> Result<Vec<String>> {
        // Delegate health checks to the Physician (Doctor Plugin)
        // Delegate health checks to the Physician (Doctor Plugin) - Removed for now
        Ok(vec![])
    }

    async fn resolve(
        &self,
        _context: &env_architect_sdk::ResolutionContext,
        config: Self::Config,
    ) -> Result<(InstallPlan, Option<String>)> {
        let auto_detect = config.auto_detect.unwrap_or(true);

        // 1. Run Auto-Detection
        let plan = if auto_detect {
            detector::detect_all()?
        } else {
            InstallPlan::default()
        };
        Ok((plan, None))
    }

    async fn install(&self, _plan: &InstallPlan, _state: Option<String>) -> Result<()> {
        // Standard plugin doesn't install anything itself (yet)
        Ok(())
    }
}
