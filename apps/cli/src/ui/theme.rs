use owo_colors::{OwoColorize, Style};
use std::fmt;

/// The central theme definition for EnvArchitect CLI.
/// Defines the official color palette and iconography.
pub struct Theme;

impl Theme {
    /// Primary "Architecture" Color (Cyan) - Structure, Blueprints.
    pub fn primary(text: impl fmt::Display) -> String {
        format!("{}", text.cyan().bold())
    }

    pub fn bold(text: impl fmt::Display) -> String {
        format!("{}", text.bold())
    }

    /// Secondary "Plugin" Color (Magenta) - Extensions, Dynamic parts.
    pub fn secondary(text: impl fmt::Display) -> String {
        format!("{}", text.magenta().bold())
    }

    /// Success Color (Neon Green)
    pub fn success(text: impl fmt::Display) -> String {
        format!("{}", text.green().bold())
    }

    /// Warning Color (Orange/Yellow)
    pub fn warning(text: impl fmt::Display) -> String {
        format!("{}", text.yellow().bold())
    }

    /// Error Color (Red)
    pub fn error(text: impl fmt::Display) -> String {
        format!("{}", text.red().bold())
    }

    /// Muted/Dimmed Color (Blue-Gray) - Metadata, Timestamps.
    pub fn muted(text: impl fmt::Display) -> String {
        format!("{}", text.dimmed())
    }
}

/// Standardized Nerd Font Icons.
/// Usage: `println!("{} Resolving...", Icon::Asset)`
pub enum Icon {
    Architect,
    Plugin,
    Package,
    Rocket,
    Shield,
    Check,
    Cross,
    Globe,
    Gear,
    Wrench,
    Download,
    File,
    Info,
    Success,
}

impl fmt::Display for Icon {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let icon = match self {
            Icon::Architect => "🏛️ ",
            Icon::Plugin => "🧩",
            Icon::Package => "📦",
            Icon::Rocket => "🚀",
            Icon::Shield => "🛡️ ",
            Icon::Check => "✔",
            Icon::Cross => "✖",
            Icon::Globe => "🌐",
            Icon::Gear => "⚙️ ",
            Icon::Wrench => "🔧",
            Icon::Download => "📥",
            Icon::File => "📄",
            Icon::Info => "ℹ️ ",
            Icon::Success => "✅",
        };
        write!(f, "{}", icon)
    }
}
