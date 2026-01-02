mod check;
mod diagnosis;
mod graph;
mod specialists;
pub mod treatments;

use self::diagnosis::Severity;
use self::graph::GraphScheduler;
use self::specialists::core::{CoreCheck, EnvCheck};
use self::specialists::node::NodeCheck;
use self::specialists::python::PythonCheck;
use self::specialists::rust::RustCheck;
use self::specialists::security::PolyglotSecurityCheck;
use env_architect_sdk::plugin;
use env_architect_sdk::prelude::*;

#[plugin]
#[derive(Default)]
struct DoctorPlugin;

#[async_trait]
impl Plugin for DoctorPlugin {
    async fn resolve(&self, _context: &ResolutionContext) -> Result<(InstallPlan, Option<String>)> {
        // POC: Run checks during resolve to see output in CLI
        run_doctor();
        Ok((InstallPlan::default(), None))
    }

    async fn validate(&self, _manifest: &serde_json::Value) -> Result<Vec<String>> {
        // Validation doesn't print, just returns empty for now
        Ok(vec![])
    }
}

fn run_doctor() {
    // ANSI Colors
    let style = |s: &str, code: u8| format!("\x1b[{}m{}\x1b[0m", code, s);
    let bold = |s: &str| style(s, 1);
    let dim = |s: &str| style(s, 2);
    // let red = |s: &str| style(s, 31);
    // let green = |s: &str| style(s, 32);
    // let yellow = |s: &str| style(s, 33);

    // Header
    host::info(format!(
        "{} v{} \n{}",
        bold("🏥 Physician"),
        env!("CARGO_PKG_VERSION"),
        dim("Intelligent System Diagnostic")
    ));

    let mut scheduler = GraphScheduler::new();
    scheduler.register(Box::new(CoreCheck));
    scheduler.register(Box::new(EnvCheck));
    scheduler.register(Box::new(NodeCheck));
    scheduler.register(Box::new(RustCheck));
    scheduler.register(Box::new(PythonCheck));
    scheduler.register(Box::new(PolyglotSecurityCheck));

    let results = scheduler.run();

    let mut has_errors = false;
    let mut warnings = 0;

    for diag in results {
        let title = bold(&diag.title);
        let code = dim(&format!("[{}]", diag.code));

        // Format the message nicely
        let mut msg = format!("{} {}", title, code);
        msg.push_str(&format!("\n{}", diag.message)); // Message on new line for clarity

        if let Some(advice) = diag.advice {
            msg.push_str(&format!("\n{}→ {}", dim(""), advice));
        }

        match diag.severity {
            Severity::Error => {
                has_errors = true;
                host::error(msg);
            }
            Severity::Warning => {
                warnings += 1;
                host::warn(msg);
            }
            Severity::Optimization => {
                host::info(msg);
            }
        }

        // AUTO-FIX: Intelligent Treatment Application
        if let Some(treatment) = &diag.treatment {
            let prompt = format!(
                "Auto-Fix: {}? (Risk: {:?})",
                treatment.description(),
                treatment.risk()
            );

            // Uses the host's native confirmation dialog
            if host::confirm(prompt, true) {
                host::info("Applying fix... (this may take a moment)".to_string());
                match treatment.apply() {
                    Ok(_) => {
                        host::success("Treatment applied successfully.");
                        // If fixed, remove from error/warning count for final report
                        match diag.severity {
                            Severity::Error => has_errors = false, // optimistically assume fixed
                            Severity::Warning => warnings -= 1,
                            _ => {}
                        }
                    }
                    Err(e) => host::error(format!("Treatment failed: {}", e)),
                }
            }
        }
    }

    // Summary Footer
    if has_errors {
        host::error("System Health: CRITICAL (Fix errors above)");
    } else if warnings > 0 {
        host::warn(format!("System Health: DEGRADED ({} warnings)", warnings));
    } else {
        host::success("System Health: EXCELLENT");
    }
}
