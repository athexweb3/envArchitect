mod detector;
// mod doctor; // Removed

use env_architect_sdk::plugin;
use env_architect_sdk::prelude::*;

#[plugin]
#[derive(Default)]
struct StandardPlugin;

#[async_trait]
impl Plugin for StandardPlugin {
    async fn validate(&self, _manifest: &serde_json::Value) -> Result<Vec<String>> {
        // Delegate health checks to the Physician (Doctor Plugin)
        Ok(env_plugin_doctor::check_system())
    }

    async fn resolve(&self, _context: &ResolutionContext) -> Result<(InstallPlan, Option<String>)> {
        // 1. Run Auto-Detection
        let plan = detector::detect_all()?;
        Ok((plan, None))
    }

    async fn install(&self, _plan: &InstallPlan, _state: Option<String>) -> Result<()> {
        // Standard plugin doesn't install anything itself (yet)
        Ok(())
    }
}
