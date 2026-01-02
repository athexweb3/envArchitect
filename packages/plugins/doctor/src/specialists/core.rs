use crate::check::{Check, CheckResult};
use crate::diagnosis::{Diagnostic, Severity};
use env_architect_sdk::host;
// use env_architect_sdk::prelude::*;

pub struct CoreCheck;
pub struct EnvCheck;

impl Check for CoreCheck {
    fn id(&self) -> &'static str {
        "core.system"
    }

    fn run(&self) -> CheckResult {
        let mut issues = Vec::new();

        if let Some(shell) = host::get_env("SHELL") {
            if shell.contains("bash") || shell.contains("zsh") || shell.contains("fish") {
                // OK
            } else {
                issues.push(Diagnostic::new(
                    Severity::Optimization,
                    "SHELL_RECOMMEND",
                    "Non-standard shell detected",
                    "We recommend zsh, bash, or fish for best compatibility.",
                ));
            }
        } else {
            issues.push(Diagnostic::new(
                Severity::Error,
                "SHELL_MISSING",
                "SHELL environment variable not set",
                "Critical system variable missing.",
            ));
        }

        Ok(issues)
    }
}

impl Check for EnvCheck {
    fn id(&self) -> &'static str {
        "core.env"
    }
    fn deps(&self) -> Vec<&'static str> {
        vec!["core.system"]
    }

    fn run(&self) -> CheckResult {
        let mut issues = Vec::new();

        if host::get_env("PATH").is_none() {
            issues.push(Diagnostic::new(
                Severity::Error,
                "PATH_MISSING",
                "PATH not set",
                "Binaries cannot be found.",
            ));
        }

        // 2. Intelligent Scannning for Missing Env Vars
        // Read .env file content first
        let mut defined_vars = std::collections::HashSet::new();
        if let Ok(content) = host::read_file(".env") {
            for line in content.lines() {
                if let Some((key, _)) = line.split_once('=') {
                    defined_vars.insert(key.trim().to_string());
                }
            }
        }

        // Scan Rust files for `std::env::var("VAR")`
        // We use grep for efficiency if available, or just skip if no grep
        if let Ok(output) = host::exec("grep", &["-r", "std::env::var", "."]) {
            for line in output.lines() {
                // Very naive parsing: looks for "VAR_NAME"
                // Example: std::env::var("DATABASE_URL")
                if let Some(start) = line.find("env::var(\"") {
                    if let Some(end) = line[start + 10..].find('"') {
                        let var_name = &line[start + 10..start + 10 + end];
                        if !defined_vars.contains(var_name) && !var_name.contains('$') {
                            issues.push(Diagnostic {
                                severity: Severity::Optimization,
                                code: "ENV_MISSING_IN_DOTENV".to_string(),
                                title: "Undefined Environment Variable".to_string(),
                                message: format!("Code uses '{}' but it is not in .env", var_name),
                                advice: Some(format!(
                                    "Add '{}=\"...\"' to your .env file.",
                                    var_name
                                )),
                                data: Default::default(),
                                treatment: None, // Could add an "Append to .env" treatment later
                            });
                            // Avoid duplicates
                            defined_vars.insert(var_name.to_string());
                        }
                    }
                }
            }
        }

        Ok(issues)
    }
}
