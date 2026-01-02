use crate::check::{Check, CheckResult};
use crate::diagnosis::{Diagnostic, Severity};
use env_architect_sdk::host;

pub struct RustCheck;

impl Check for RustCheck {
    fn id(&self) -> &'static str {
        "toolchain.rust"
    }

    fn deps(&self) -> Vec<&'static str> {
        vec!["core.env"]
    }

    fn is_relevant(&self) -> bool {
        host::read_file("Cargo.toml").is_ok()
    }

    fn run(&self) -> CheckResult {
        let mut diagnostics = Vec::new();

        // 1. Check rustc
        match host::exec("rustc", &["--version"]) {
            Ok(version) => {
                diagnostics.push(Diagnostic::new(
                    Severity::Optimization,
                    "RUST_VERSION",
                    "Rust detected",
                    &format!("Version: {}", version.trim()),
                ));
            }
            Err(_) => {
                diagnostics.push(Diagnostic::new(
                    Severity::Error,
                    "RUST_MISSING",
                    "Rust Toolchain not found",
                    "rustc is not in PATH."
                ).with_advice("Install via 'rustup': curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"));
                return Ok(diagnostics); // Critical failure, stop here
            }
        }

        // 2. Check cargo
        match host::exec("cargo", &["--version"]) {
            Ok(_) => {}
            Err(_) => {
                diagnostics.push(Diagnostic::new(
                    Severity::Warning,
                    "CARGO_MISSING",
                    "Cargo not found",
                    "rustc exists but cargo is missing.",
                ));
            }
        }

        Ok(diagnostics)
    }
}
