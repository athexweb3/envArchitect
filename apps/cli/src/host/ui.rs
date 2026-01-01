use crate::ui;
use crate::ui::components::prompt;
use tokio::task::spawn_blocking;

// Define the bindings module structure to match wit-bindgen output
use super::bindings::env_architect::plugin::host::Host;
use super::bindings::env_architect::plugin::host::LogLevel;
use super::state::HostState;

#[async_trait::async_trait]
impl Host for HostState {
    async fn log(&mut self, level: LogLevel, message: String) -> () {
        match level {
            LogLevel::Debug => (), // Skip debug
            LogLevel::Info => ui::info(message),
            LogLevel::Warn => ui::warn(message),
            LogLevel::Error => ui::error(message),
        }
    }

    async fn confirm(&mut self, prompt_msg: String, _default: bool) -> bool {
        // Inquire handles defaults internally or via wrapper, our wrapper sets true as default
        spawn_blocking::<_, bool>(move || prompt::confirm(&prompt_msg))
            .await
            .unwrap_or(false)
    }

    async fn input(&mut self, prompt_msg: String, _default: Option<String>) -> String {
        spawn_blocking::<_, String>(move || prompt::input(&prompt_msg))
            .await
            .unwrap_or_default()
    }

    async fn secret(&mut self, prompt_msg: String) -> String {
        // Enforce capability
        if !self.allowed_capabilities.contains(&"ui-secret".to_string()) {
            // High-fidelity diagnostic
            if let (Some(path), Some(content)) = (&self.manifest_path, &self.manifest_content) {
                ui::diagnostic::report_denied_capability(path, content, "ui-secret");
            } else {
                ui::error("Capability Denied: ui-secret (No manifest context available)");
            }
            return String::new();
        }

        spawn_blocking::<_, String>(move || prompt::secret(&prompt_msg))
            .await
            .unwrap_or_default()
    }

    async fn select(
        &mut self,
        prompt_msg: String,
        options: Vec<String>,
        _default: Option<String>,
    ) -> String {
        spawn_blocking::<_, String>(move || {
            // Options need to be &str for our wrapper, converting
            let opts_ref: Vec<&str> = options.iter().map(|s| s.as_str()).collect();
            prompt::select(&prompt_msg, opts_ref)
        })
        .await
        .unwrap_or_default()
    }

    async fn get_env(&mut self, key: String) -> Option<String> {
        std::env::var(key).ok()
    }

    async fn read_file(&mut self, path: String) -> Result<String, String> {
        std::fs::read_to_string(path).map_err(|e| e.to_string())
    }
}
