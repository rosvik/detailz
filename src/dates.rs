use std::fs::Metadata;

use colored::Colorize;

use crate::format::{fmt_time, label};

pub fn print_dates(metadata: &Metadata) {
    let inline_label = |s: &str| format!("{} {}", "/".dimmed(), s.cyan().bold());

    let mut parts = Vec::new();
    match metadata.created() {
        Ok(t) => parts.push(format!("{}{}", label("Created:"), fmt_time(t))),
        Err(_) => return,
    }
    if let Ok(t) = metadata.modified() {
        parts.push(format!("{} {}", inline_label("Modified:"), fmt_time(t)));
    }
    if let Ok(t) = metadata.accessed() {
        parts.push(format!("{} {}", inline_label("Accessed:"), fmt_time(t)));
    }

    if !parts.is_empty() {
        println!("{}", parts.join(" "));
    }
}
