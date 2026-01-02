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
            LogLevel::Info => { let _ = cliclack::log::info(message); },
            LogLevel::Warn => { let _ = cliclack::log::warning(message); },
            LogLevel::Error => { let _ = cliclack::log::error(message); },
        }
    }

    async fn confirm(&mut self, prompt_msg: String, _default: bool) -> bool {
        spawn_blocking(move || {
            cliclack::confirm(prompt_msg).interact().unwrap_or(false)
        })
        .await
        .unwrap_or(false)
    }

    async fn input(&mut self, prompt_msg: String, _default: Option<String>) -> String {
        spawn_blocking(move || {
            cliclack::input(prompt_msg).interact().unwrap_or_default()
        })
        .await
        .unwrap_or_default()
    }

    async fn secret(&mut self, prompt_msg: String) -> String {
        // Enforce capability
        if !self.allowed_capabilities.contains(&"ui-secret".to_string()) {
            let _ = cliclack::log::error("Capability Denied: ui-secret");
            return String::new();
        }

        spawn_blocking(move || {
            cliclack::password(prompt_msg).interact().unwrap_or_default()
        })
        .await
        .unwrap_or_default()
    }

    async fn select(
        &mut self,
        prompt_msg: String,
        options: Vec<String>,
        _default: Option<String>,
    ) -> String {
        spawn_blocking(move || {
            let mut selection = cliclack::select(prompt_msg);
            for opt in &options {
                selection = selection.item(opt, opt, "");
            }
            selection.interact().unwrap_or_default().to_string()
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
