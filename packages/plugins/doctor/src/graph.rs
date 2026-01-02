use crate::check::Check;
use crate::diagnosis::{Diagnostic, Severity};
use std::collections::{HashMap, HashSet};

pub struct GraphScheduler {
    checks: HashMap<&'static str, Box<dyn Check>>,
}

impl GraphScheduler {
    pub fn new() -> Self {
        Self {
            checks: HashMap::new(),
        }
    }

    pub fn register(&mut self, check: Box<dyn Check>) {
        self.checks.insert(check.id(), check);
    }

    pub fn run(&self) -> Vec<Diagnostic> {
        let mut results = Vec::new();
        let mut visited = HashSet::new();
        let mut failed_ids = HashSet::new();

        // Simple logic for now
        let mut queue: Vec<&str> = self.checks.keys().cloned().collect();
        queue.sort_by_key(|id| self.checks.get(id).unwrap().deps().len());

        for id in &queue {
            self.solve(*id, &mut visited, &mut failed_ids, &mut results);
        }

        results
    }

    fn solve(
        &self,
        id: &'static str,
        visited: &mut HashSet<&'static str>,
        failed_ids: &mut HashSet<&'static str>,
        results: &mut Vec<Diagnostic>,
    ) {
        if visited.contains(id) {
            return;
        }

        if let Some(check) = self.checks.get(id) {
            // 1. Check Deps
            for dep in check.deps() {
                self.solve(dep, visited, failed_ids, results);
                if failed_ids.contains(dep) {
                    failed_ids.insert(id);
                    visited.insert(id);
                    return;
                }
            }

            // 2. Check Relevance
            if !check.is_relevant() {
                visited.insert(id);
                return;
            }

            // 3. Run Check
            match check.run() {
                Ok(diagnostics) => {
                    let has_error = diagnostics.iter().any(|d| d.severity == Severity::Error);
                    if has_error {
                        failed_ids.insert(id);
                    }
                    results.extend(diagnostics);
                }
                Err(e) => {
                    failed_ids.insert(id);
                    results.push(Diagnostic::new(
                        Severity::Error,
                        "CHECK_PANIC",
                        &format!("Check {} failed to execute", id),
                        &e,
                    ));
                }
            }
            visited.insert(id);
        }
    }
}
