use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Plugin error: {0}")]
    Plugin(String),

    #[error("Runtime error: {0}")]
    Runtime(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Operation cancelled by user")]
    Cancelled,
}

impl CliError {
    /// Returns a themed, actionable suggestion for the error.
    pub fn suggestion(&self) -> Option<String> {
        match self {
            CliError::Config(_) => {
                Some("Check your env.json for syntax errors or missing fields.".to_string())
            }
            CliError::Plugin(_) => {
                Some("Try updating the plugin or checking its capabilities.".to_string())
            }
            _ => None,
        }
    }

    pub fn render(&self) {
        eprintln!("\n{} {}", console::style("Error:").red().bold(), self);
        if let Some(s) = self.suggestion() {
            eprintln!("{} {}", console::style("  help:").dim(), s);
        }
    }
}
