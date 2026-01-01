use indicatif::MultiProgress;
use std::sync::OnceLock;

pub mod components;
pub mod diagnostic;
pub mod theme;

pub use theme::{Icon, Theme};

static MULTI_PROGRESS: OnceLock<MultiProgress> = OnceLock::new();

pub fn multi_progress() -> &'static MultiProgress {
    MULTI_PROGRESS.get_or_init(MultiProgress::new)
}

pub fn info(message: impl AsRef<str>) {
    let msg = format!("{} {}", Theme::primary("ℹ️"), message.as_ref());
    if let Err(_) = multi_progress().println(&msg) {
        println!("{}", msg);
    }
}

pub fn warn(message: impl AsRef<str>) {
    let msg = format!("{} {}", Theme::warning("⚠️"), message.as_ref());
    if let Err(_) = multi_progress().println(&msg) {
        println!("{}", msg);
    }
}

pub fn error(message: impl AsRef<str>) {
    let msg = format!("{} {}", Theme::error("❌"), message.as_ref());
    if let Err(_) = multi_progress().println(&msg) {
        eprintln!("{}", msg);
    }
}

pub fn println(message: impl AsRef<str>) {
    if let Err(_) = multi_progress().println(message.as_ref()) {
        println!("{}", message.as_ref());
    }
}
