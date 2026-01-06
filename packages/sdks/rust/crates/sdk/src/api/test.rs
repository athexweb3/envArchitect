use crate::api::context::ResolutionContext;
use crate::api::traits::PluginHandler;
use crate::api::types::InstallPlan;
use anyhow::Result;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

thread_local! {
    pub static ACTIVE_MOCK: RefCell<Option<MockHost>> = RefCell::new(None);
}

/// A mock implementation of the EnvArchitect host.
/// Allows developers to record logs and mock UI/IO interactions during unit tests.
#[derive(Default, Clone)]
pub struct MockHost {
    logs: Arc<Mutex<Vec<(String, String)>>>,
    env: Arc<Mutex<HashMap<String, String>>>,
    files: Arc<Mutex<HashMap<String, String>>>,
    ui_responses: Arc<Mutex<HashMap<String, bool>>>,
    ui_string_responses: Arc<Mutex<HashMap<String, String>>>,
}

impl MockHost {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set this mock as the active one for the current thread.
    pub fn enter(&self) -> MockGuard {
        ACTIVE_MOCK.with(|m| *m.borrow_mut() = Some(self.clone()));
        MockGuard
    }

    pub fn exit() {
        ACTIVE_MOCK.with(|m| *m.borrow_mut() = None);
    }

    /// Record a log entry.
    pub fn log(&self, level: &str, message: &str) {
        self.logs
            .lock()
            .unwrap()
            .push((level.to_string(), message.to_string()));
    }

    /// Get all recorded logs.
    pub fn get_logs(&self) -> Vec<(String, String)> {
        self.logs.lock().unwrap().clone()
    }

    /// Mock an environment variable.
    pub fn set_env(&self, key: &str, value: &str) {
        self.env
            .lock()
            .unwrap()
            .insert(key.to_string(), value.to_string());
    }

    pub fn get_env(&self, key: &str) -> Option<String> {
        self.env.lock().unwrap().get(key).cloned()
    }

    /// Mock a file in the virtualized filesystem.
    pub fn set_file(&self, path: &str, content: &str) {
        self.files
            .lock()
            .unwrap()
            .insert(path.to_string(), content.to_string());
    }

    pub fn read_file(&self, path: &str) -> Result<String, String> {
        self.files
            .lock()
            .unwrap()
            .get(path)
            .cloned()
            .ok_or_else(|| "File not found".to_string())
    }

    /// Mock a UI confirmation response.
    pub fn mock_confirm(&self, prompt: &str, response: bool) {
        self.ui_responses
            .lock()
            .unwrap()
            .insert(prompt.to_string(), response);
    }

    pub fn confirm(&self, prompt: &str, default: bool) -> bool {
        self.ui_responses
            .lock()
            .unwrap()
            .get(prompt)
            .copied()
            .unwrap_or(default)
    }

    /// Mock a UI text input response.
    pub fn mock_input(&self, prompt: &str, response: &str) {
        self.ui_string_responses
            .lock()
            .unwrap()
            .insert(prompt.to_string(), response.to_string());
    }

    pub fn input(&self, prompt: &str, default: Option<&str>) -> String {
        self.ui_string_responses
            .lock()
            .unwrap()
            .get(prompt)
            .cloned()
            .unwrap_or_else(|| default.unwrap_or("").to_string())
    }

    /// Mock a UI select response.
    pub fn mock_select(&self, prompt: &str, response: &str) {
        self.ui_string_responses
            .lock()
            .unwrap()
            .insert(prompt.to_string(), response.to_string());
    }

    pub fn select(&self, prompt: &str, options: &[&str], default: Option<&str>) -> String {
        self.ui_string_responses
            .lock()
            .unwrap()
            .get(prompt)
            .cloned()
            .unwrap_or_else(|| {
                default
                    .unwrap_or(options.first().unwrap_or(&""))
                    .to_string()
            })
    }

    /// Mock a UI secret response.
    pub fn mock_secret(&self, prompt: &str, response: &str) {
        self.ui_string_responses
            .lock()
            .unwrap()
            .insert(prompt.to_string(), response.to_string());
    }

    pub fn secret(&self, prompt: &str) -> String {
        self.ui_string_responses
            .lock()
            .unwrap()
            .get(prompt)
            .cloned()
            .unwrap_or_default()
    }

    pub fn spinner(&self, msg: &str) -> Box<dyn crate::api::traits::Spinner> {
        self.log("Spinner", msg);
        Box::new(MockSpinner {
            msg: msg.to_string(),
            host: self.clone(),
        })
    }
}

struct MockSpinner {
    msg: String,
    host: MockHost,
}

impl crate::api::traits::Spinner for MockSpinner {
    fn set_message(&self, msg: &str) {
        self.host.log("Spinner Update", msg);
    }
    fn finish(&self) {
        self.host.log("Spinner Finish", &self.msg);
    }
}

/// A helper to run a plugin handler against a MockHost.
pub struct TestRunner<P: PluginHandler> {
    plugin: P,
    pub host: MockHost,
    pub capabilities: Vec<String>,
}

impl<P: PluginHandler> TestRunner<P> {
    pub fn new(plugin: P) -> Self {
        Self {
            plugin,
            host: MockHost::new(),
            // Default to granting all common permissions for tests
            capabilities: vec![
                "ui-interact".to_string(),
                "ui-secret".to_string(),
                "network".to_string(),
                "fs-read".to_string(),
                "fs-write".to_string(),
            ],
        }
    }

    /// Customize the capabilities for this test runner.
    pub fn with_capabilities(mut self, caps: Vec<String>) -> Self {
        self.capabilities = caps;
        self
    }

    fn set_context(&self) {
        crate::api::context::set_active_capabilities(self.capabilities.clone());
    }

    pub async fn run_test<F, Fut, R>(&self, f: F) -> R
    where
        F: FnOnce(MockHost) -> Fut,
        Fut: std::future::Future<Output = R>,
    {
        let _guard = self.host.enter();
        self.set_context();
        f(self.host.clone()).await
    }

    pub async fn resolve(&self, ctx: &ResolutionContext) -> Result<(InstallPlan, Option<String>)> {
        let _guard = self.host.enter();
        self.set_context();
        self.plugin
            .resolve(ctx, <P as PluginHandler>::Config::default())
            .await
    }

    pub async fn install(&self, plan: &InstallPlan, state: Option<String>) -> Result<()> {
        let _guard = self.host.enter();
        self.set_context();
        self.plugin.install(plan, state).await
    }

    pub async fn validate(&self, manifest: &serde_json::Value) -> Result<Vec<String>> {
        let _guard = self.host.enter();
        self.set_context();
        self.plugin.validate(manifest).await
    }
}

pub struct MockGuard;
impl Drop for MockGuard {
    fn drop(&mut self) {
        MockHost::exit();
    }
}
