use crate::check::{Check, CheckResult};
use crate::diagnosis::{Diagnostic, Severity};
use env_architect_sdk::host;

pub struct PythonCheck;

impl Check for PythonCheck {
    fn id(&self) -> &'static str {
        "toolchain.python"
    }

    fn deps(&self) -> Vec<&'static str> {
        vec!["core.env"]
    }

    fn is_relevant(&self) -> bool {
        host::read_file(".python-version").is_ok()
            || host::read_file("requirements.txt").is_ok()
            || host::read_file("Pipfile").is_ok()
            || host::read_file("pyproject.toml").is_ok()
            || host::read_file("runtime.txt").is_ok()
    }

    fn run(&self) -> CheckResult {
        let mut diagnostics = Vec::new();

        // 1. Check python3 or python
        let python_bin = if host::exec("python3", &["--version"]).is_ok() {
            "python3"
        } else if host::exec("python", &["--version"]).is_ok() {
            "python"
        } else {
            diagnostics.push(
                Diagnostic::new(
                    Severity::Error,
                    "PYTHON_MISSING",
                    "Python not found",
                    "Neither python3 nor python found in PATH.",
                )
                .with_advice("Install via your package manager or python.org."),
            );
            return Ok(diagnostics);
        };

        // Get Version
        if let Ok(version) = host::exec(python_bin, &["--version"]) {
            diagnostics.push(Diagnostic::new(
                Severity::Optimization,
                "PYTHON_VERSION",
                "Python detected",
                &format!("Using binary: {} ({})", python_bin, version.trim()),
            ));
        }

        // 2. Check Pip
        let pip_bin = if python_bin == "python3" {
            "pip3"
        } else {
            "pip"
        };
        if host::exec(pip_bin, &["--version"]).is_err() {
            diagnostics.push(
                Diagnostic::new(
                    Severity::Warning,
                    "PIP_MISSING",
                    "Pip not found",
                    "Python exists but pip is missing. You may not be able to install packages.",
                )
                .with_advice("Install via 'ensurepip': python -m ensurepip --upgrade"),
            );
        }

        Ok(diagnostics)
    }
}
