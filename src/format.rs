use std::time::SystemTime;

use chrono::{DateTime, Local};
use colored::{ColoredString, Colorize};

pub fn label(s: &str) -> ColoredString {
    format!("{:<13}", s).bold().cyan()
}

pub fn human_size(bytes: u64) -> String {
    const UNITS: [&str; 7] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB"];
    if bytes < 1024 {
        return format!("{} B", bytes);
    }
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    format!("{:.2} {}", value, UNITS[unit])
}

pub fn fmt_time(t: SystemTime) -> String {
    let dt: DateTime<Local> = t.into();
    dt.format("%Y-%m-%d %H:%M:%S %z").to_string()
}

pub fn format_mode(mode: u32) -> String {
    let mut s = String::with_capacity(9 * 12);
    for (bit, ch) in [
        (0o400, 'r'),
        (0o200, 'w'),
        (0o100, 'x'),
        (0o040, 'r'),
        (0o020, 'w'),
        (0o010, 'x'),
        (0o004, 'r'),
        (0o002, 'w'),
        (0o001, 'x'),
    ] {
        let part = if mode & bit != 0 {
            match ch {
                'r' => ch.to_string().yellow(),
                'w' => ch.to_string().red(),
                'x' => ch.to_string().green(),
                _ => ch.to_string().normal(),
            }
        } else {
            "-".dimmed()
        };
        s.push_str(&part.to_string());
    }
    s
}
