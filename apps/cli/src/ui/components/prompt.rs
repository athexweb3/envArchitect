use inquire::{Confirm, Select, Text};

pub fn confirm(prompt: &str) -> bool {
    crate::ui::multi_progress().suspend(|| {
        Confirm::new(prompt)
            .with_default(true)
            .with_help_message("Press Enter to confirm")
            .prompt()
            .unwrap_or(false)
    })
}

pub fn input(prompt: &str) -> String {
    crate::ui::multi_progress().suspend(|| Text::new(prompt).prompt().unwrap_or_default())
}

pub fn select(prompt: &str, options: Vec<&str>) -> String {
    crate::ui::multi_progress().suspend(|| {
        Select::new(prompt, options)
            .with_page_size(10)
            .prompt()
            .unwrap_or_default()
            .to_string()
    })
}

pub fn secret(prompt: &str) -> String {
    crate::ui::multi_progress().suspend(|| {
        inquire::Password::new(prompt)
            .with_display_mode(inquire::PasswordDisplayMode::Masked)
            .prompt()
            .unwrap_or_default()
    })
}
