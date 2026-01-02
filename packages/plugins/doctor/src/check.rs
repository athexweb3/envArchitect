use crate::diagnosis::Diagnostic;

pub type CheckResult = Result<Vec<Diagnostic>, String>;

pub trait Check {
    /// Unique ID for the check (e.g. "toolchain.node")
    fn id(&self) -> &'static str;

    /// Dependencies that must pass before this check runs
    fn deps(&self) -> Vec<&'static str> {
        vec![]
    }

    /// The main check logic
    fn run(&self) -> CheckResult;

    /// Whether this check is relevant for the current context (e.g. file existence)
    fn is_relevant(&self) -> bool {
        true
    }
}
