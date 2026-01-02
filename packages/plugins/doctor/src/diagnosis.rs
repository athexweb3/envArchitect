use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

// 1. Severity Levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Error,        // Blocking issue
    Warning,      // Non-blocking but risky
    Optimization, // "Advisor": Recommended best practice
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

// 2. The Treatment (Remediation)
#[async_trait]
pub trait Treatment: fmt::Debug + Send + Sync {
    fn description(&self) -> String;
    // Returns a risk level/confirmation prompt
    fn risk(&self) -> RiskLevel;
    fn apply(&self) -> anyhow::Result<()>;
}

// 3. The Diagnostic (SARIF-lite)
#[derive(Debug)]
#[allow(dead_code)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: String,
    pub title: String,
    pub message: String,
    pub advice: Option<String>, // "Why you should do this"
    pub data: HashMap<String, String>,
    pub treatment: Option<Box<dyn Treatment>>,
}

impl Diagnostic {
    pub fn new(severity: Severity, code: &str, title: &str, message: &str) -> Self {
        Self {
            severity,
            code: code.to_string(),
            title: title.to_string(),
            message: message.to_string(),
            advice: None,
            data: HashMap::new(),
            treatment: None,
        }
    }

    pub fn with_advice(mut self, advice: &str) -> Self {
        self.advice = Some(advice.to_string());
        self
    }

    #[allow(dead_code)]
    pub fn with_treatment(mut self, treatment: Box<dyn Treatment>) -> Self {
        self.treatment = Some(treatment);
        self
    }
}
