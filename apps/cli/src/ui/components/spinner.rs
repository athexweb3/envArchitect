use crate::ui::Theme;
use indicatif::{ProgressBar, ProgressStyle};

pub struct Spinner {
    pb: ProgressBar,
}

impl Spinner {
    pub fn new(msg: impl Into<String>) -> Self {
        let pb = crate::ui::multi_progress().add(ProgressBar::new_spinner());
        pb.set_style(
            ProgressStyle::default_spinner()
                // Premium high-density dot sequence
                .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
                .template("{spinner:.cyan} {prefix:.bold} {msg:.dim}")
                .unwrap(),
        );
        pb.set_prefix(msg.into());
        Self { pb }
    }

    /// Sets the secondary (sub-task) message
    pub fn set_message(&self, msg: impl Into<String>) {
        self.pb.set_message(msg.into());
    }

    /// Updates the primary task name (prefix)
    pub fn set_task(&self, task: impl Into<String>) {
        self.pb.set_prefix(task.into());
    }

    pub fn success(&self, msg: impl Into<String>) {
        self.pb
            .finish_with_message(format!("{} {}", Theme::success("✔"), msg.into()));
    }

    pub fn fail(&self, msg: impl Into<String>) {
        self.pb
            .finish_with_message(format!("{} {}", Theme::error("✖"), msg.into()));
    }
}

impl Drop for Spinner {
    fn drop(&mut self) {
        if !self.pb.is_finished() {
            self.pb.finish_and_clear();
        }
    }
}
