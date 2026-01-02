use crate::check::{Check, CheckResult};
use crate::diagnosis::{Diagnostic, Severity};
use env_architect_sdk::host;

pub struct PolyglotSecurityCheck;

impl Check for PolyglotSecurityCheck {
    fn id(&self) -> &'static str {
        "doctor.security.polyglot"
    }

    fn run(&self) -> CheckResult {
        let mut diagnostics = Vec::new();

        // 1. Rust Audit
        if host::read_file("Cargo.toml").is_ok() {
            diagnostics.extend(self.check_cargo_audit());
        }

        // 2. Node Audit
        if host::read_file("package.json").is_ok() {
            diagnostics.extend(self.check_npm_audit());
        }

        Ok(diagnostics)
    }
}

impl PolyglotSecurityCheck {
    fn check_cargo_audit(&self) -> Vec<Diagnostic> {
        // 1. Check if tool exists explicitly
        if host::exec("cargo", &["audit", "--version"]).is_err() {
            return vec![Diagnostic {
                severity: Severity::Warning,
                code: "SEC_RUST_TOOL_MISSING".to_string(),
                title: "Security Scan Skipped".to_string(),
                message: "Could not run 'cargo audit'. Security checks skipped.".to_string(),
                advice: Some("Install 'cargo-audit' via 'cargo install cargo-audit'.".to_string()),
                data: Default::default(),
                treatment: Some(Box::new(crate::treatments::CargoInstallTreatment {
                    package: "cargo-audit".to_string(),
                })),
            }];
        }

        // 2. Run cargo audit
        // Note: cargo audit exits with 1 if vulnerabilities are found, causing host::exec to return Err
        let result = host::exec("cargo", &["audit", "--json", "--no-fetch"]);

        match result {
            Ok(output_str) => {
                if output_str.contains("\"kind\":\"vulnerability\"")
                    || output_str.contains("\"vulnerabilities\": { \"list\": [")
                {
                    Self::vuln_found_diagnostic()
                } else {
                    vec![] // Safe
                }
            }
            Err(_) => {
                // If it failed but is installed, it likely found vulnerabilities (exit code 1)
                // Since we can't easily parse stdout on error with current host::exec, we assume vulns.
                Self::vuln_found_diagnostic()
            }
        }
    }

    fn vuln_found_diagnostic() -> Vec<Diagnostic> {
        vec![Diagnostic {
            severity: Severity::Error,
            code: "SEC_RUST_VULN".to_string(),
            title: "Vulnerabilities Detected".to_string(),
            message: "Cargo audit found vulnerabilities in dependencies.".to_string(),
            advice: Some(
                "Run 'cargo audit' to view details and 'cargo update' to fix.".to_string(),
            ),
            data: Default::default(),
            treatment: None,
        }]
    }

    fn check_npm_audit(&self) -> Vec<Diagnostic> {
        let result = host::exec("npm", &["audit", "--json"]);
        match result {
            Ok(output_str) => {
                if output_str.contains("\"vulnerabilities\":")
                    && !output_str.contains("\"vulnerabilities\":{}")
                {
                    vec![Diagnostic {
                        severity: Severity::Error, // Gatekeeper
                        code: "SEC_NODE_VULN".to_string(),
                        title: "Vulnerabilities Detected".to_string(),
                        message: "NPM audit found vulnerabilities.".to_string(),
                        advice: Some("Run 'npm audit fix' to resolve.".to_string()),
                        data: Default::default(),
                        treatment: None,
                    }]
                } else {
                    vec![]
                }
            }
            Err(_) => vec![],
        }
    }
}
