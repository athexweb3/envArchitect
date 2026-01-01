use crate::api::test::{MockHost, ACTIVE_MOCK};
#[allow(unused_imports)]
use crate::internal::bindings::env_architect::plugin::host;
use host::LogLevel;

/// Log a message to the host terminal.
pub fn log(level: LogLevel, message: impl Into<String>) {
    let msg = message.into();
    ACTIVE_MOCK.with(|m| {
        if let Some(mock) = m.borrow().as_ref() {
            mock.log(&format!("{:?}", level), &msg);
        } else {
            host::log(level, &msg);
        }
    });
}

pub fn debug(message: impl Into<String>) {
    log(LogLevel::Debug, message);
}

pub fn info(message: impl Into<String>) {
    let has_cap = crate::api::context::check_capability("ui-interact");

    if !has_cap {
        // Since 'info' is just logging, we might want to skip it silently or log a warning to stdout
        // println!("DEBUG: [info] Skipped due to missing capability.");
        return;
    }

    match crate::api::context::check_capability("ui-interact") {
        true => log(LogLevel::Info, message),
        false => {}
    }
}

pub fn warn(message: impl Into<String>) {
    log(LogLevel::Warn, message);
}

pub fn error(message: impl Into<String>) {
    log(LogLevel::Error, message);
}

/// Get an environment variable from the host context.
pub fn get_env(key: impl Into<String>) -> Option<String> {
    let k = key.into();
    ACTIVE_MOCK.with(|m| {
        if let Some(mock) = m.borrow().as_ref() {
            mock.get_env(&k)
        } else {
            host::get_env(&k)
        }
    })
}

/// Read a file from the virtualized filesystem.
pub fn read_file(path: impl Into<String>) -> Result<String, String> {
    let p = path.into();
    ACTIVE_MOCK.with(|m| {
        if let Some(mock) = m.borrow().as_ref() {
            mock.read_file(&p)
        } else {
            host::read_file(&p)
        }
    })
}

// UI Re-exports or wrappers
pub fn success(message: impl Into<String>) {
    if !crate::api::context::check_capability("ui-interact") {
        return;
    }
    let msg = message.into();
    info(format!("✔ {}", msg));
}

pub fn confirm(prompt: impl Into<String>, default: bool) -> bool {
    let has_cap = crate::api::context::check_capability("ui-interact");

    // Debug print that bypasses CLI suppression
    if !has_cap {
        return default;
    }

    let p = prompt.into();
    ACTIVE_MOCK.with(|m| {
        if let Some(mock) = m.borrow().as_ref() {
            mock.confirm(&p, default)
        } else {
            host::confirm(&p, default)
        }
    })
}

pub fn input(prompt: impl Into<String>, default: Option<String>) -> String {
    if !crate::api::context::check_capability("ui-interact") {
        return default.unwrap_or_default();
    }
    let p = prompt.into();
    ACTIVE_MOCK.with(|m| {
        if let Some(mock) = m.borrow().as_ref() {
            mock.input(&p, default.as_deref())
        } else {
            host::input(&p, default.as_deref())
        }
    })
}

pub fn select(prompt: impl Into<String>, options: &[&str], default: Option<String>) -> String {
    if !crate::api::context::check_capability("ui-interact") {
        return default
            .or_else(|| options.first().map(|s| s.to_string()))
            .unwrap_or_default();
    }
    let p = prompt.into();
    let opts: Vec<String> = options.iter().map(|s| s.to_string()).collect();
    ACTIVE_MOCK.with(|m| {
        if let Some(mock) = m.borrow().as_ref() {
            mock.select(&p, options, default.as_deref())
        } else {
            host::select(&p, &opts, default.as_deref())
        }
    })
}

pub fn secret(prompt: impl Into<String>) -> String {
    if !crate::api::context::check_capability("ui-secret") {
        // Log an error if capability is missing - the host should ideally handle this more strictly
        error("Missing required capability: ui-secret");
        return String::new();
    }
    let p = prompt.into();
    ACTIVE_MOCK.with(|m| {
        if let Some(mock) = m.borrow().as_ref() {
            mock.secret(&p)
        } else {
            host::secret(&p)
        }
    })
}

pub fn spinner(message: impl Into<String>) -> Box<dyn crate::api::traits::Spinner> {
    let msg = message.into();
    ACTIVE_MOCK.with(|m| {
        if let Some(mock) = m.borrow().as_ref() {
            mock.spinner(&msg)
        } else {
            // WIT doesn't have spinner yet, use a logging fallback
            info(format!("[Spinner] {}", msg));
            Box::new(RealHostSpinner { msg })
        }
    })
}

struct RealHostSpinner {
    msg: String,
}

impl crate::api::traits::Spinner for RealHostSpinner {
    fn set_message(&self, msg: &str) {
        info(format!("[Spinner Update] {}", msg));
    }
    fn finish(&self) {
        info(format!("[Spinner Finish] {}", self.msg));
    }
}
