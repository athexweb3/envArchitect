use crate::check::{Check, CheckResult};
use crate::diagnosis::{Diagnostic, Severity};
use env_architect_sdk::host;
// use env_architect_sdk::prelude::*;

pub struct NodeCheck;

impl Check for NodeCheck {
    fn id(&self) -> &'static str {
        "toolchain.node"
    }
    fn deps(&self) -> Vec<&'static str> {
        vec!["core.env"]
    }

    fn is_relevant(&self) -> bool {
        host::read_file("package.json").is_ok()
    }

    fn run(&self) -> CheckResult {
        let mut issues = Vec::new();

        // 1. Basic Existence
        match host::exec("node", &["--version"]) {
            Ok(v) => {
                // 2. Advisor: Check for NVM
                let is_nvm = host::get_env("NVM_DIR").is_some();
                if !is_nvm {
                    issues.push(Diagnostic::new(
                        Severity::Optimization,
                        "NODE_SYS_INSTALL",
                        "System Node.js detected",
                        "We recommend using 'nvm' or 'volta' to manage Node versions easily.",
                    ));
                }

                // 3. Version Check
                if v.starts_with("v12") || v.starts_with("v14") {
                    issues.push(Diagnostic::new(
                        Severity::Warning,
                        "NODE_DEPRECATED",
                        "Old Node.js version detected",
                        &format!("Found {}. Recommend upgrading to LTS (v18+).", v.trim()),
                    ));
                }
            }
            Err(_) => {
                issues.push(
                    Diagnostic::new(
                        Severity::Error,
                        "NODE_MISSING",
                        "Node.js not found",
                        "Install Node.js to use JavaScript environments.",
                    )
                    .with_advice("Visit https://nodejs.org/ or use nvm."),
                );
            }
        }

        Ok(issues)
    }
}
