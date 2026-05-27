use std::time::SystemTime;

use chrono::{DateTime, Local};
use colored::{ColoredString, Colorize};

pub const LABEL_WIDTH: usize = 13;

pub fn label(s: &str) -> ColoredString {
    format!("{:<width$}", s, width = LABEL_WIDTH).bold().cyan()
}

/// Print a sequence of label/value pairs, joining them on a single line when
/// they fit in `terminal_width`, otherwise falling back to one pair per line.
///
/// Values are assumed to be plain (non-ANSI) strings so that `.len()` reflects
/// the rendered width.
pub fn print_label_value_pairs(pairs: &[(&str, String)], terminal_width: usize) {
    if pairs.is_empty() {
        return;
    }

    // Inline form: "<padded label>VALUE / Label: VALUE / Label: VALUE"
    //   - first pair: LABEL_WIDTH chars (padded) + value
    //   - each subsequent pair: " / " (3) + label + " " (1) + value
    let inline_len: usize = pairs
        .iter()
        .enumerate()
        .map(|(i, (lbl, val))| {
            if i == 0 {
                LABEL_WIDTH + val.len()
            } else {
                3 + lbl.len() + 1 + val.len()
            }
        })
        .sum();

    if inline_len <= terminal_width {
        let mut line = String::new();
        for (i, (lbl, val)) in pairs.iter().enumerate() {
            if i == 0 {
                line.push_str(&format!("{}{}", label(lbl), val));
            } else {
                line.push_str(&format!(
                    " {} {} {}",
                    "/".dimmed(),
                    lbl.cyan().bold(),
                    val
                ));
            }
        }
        println!("{}", line);
    } else {
        for (lbl, val) in pairs {
            println!("{}{}", label(lbl), val);
        }
    }
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

/// Render the same layout used by `print_label_value_pairs`, but to a String
/// instead of stdout. Exposed for tests.
#[cfg(test)]
fn render_label_value_pairs(pairs: &[(&str, String)], terminal_width: usize) -> String {
    use std::fmt::Write;
    if pairs.is_empty() {
        return String::new();
    }
    let inline_len: usize = pairs
        .iter()
        .enumerate()
        .map(|(i, (lbl, val))| {
            if i == 0 {
                LABEL_WIDTH + val.len()
            } else {
                3 + lbl.len() + 1 + val.len()
            }
        })
        .sum();

    let mut out = String::new();
    if inline_len <= terminal_width {
        for (i, (lbl, val)) in pairs.iter().enumerate() {
            if i == 0 {
                write!(out, "{:<width$}{}", lbl, val, width = LABEL_WIDTH).unwrap();
            } else {
                write!(out, " / {} {}", lbl, val).unwrap();
            }
        }
    } else {
        for (i, (lbl, val)) in pairs.iter().enumerate() {
            if i > 0 {
                out.push('\n');
            }
            write!(out, "{:<width$}{}", lbl, val, width = LABEL_WIDTH).unwrap();
        }
    }
    out
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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_pairs() -> Vec<(&'static str, String)> {
        vec![
            ("Created:", "2026-05-27 14:16:29 +0200".to_string()),
            ("Modified:", "2026-05-27 14:16:29 +0200".to_string()),
            ("Accessed:", "2026-05-27 14:16:48 +0200".to_string()),
        ]
    }

    #[test]
    fn inline_when_terminal_is_wide() {
        let pairs = sample_pairs();
        let out = render_label_value_pairs(&pairs, 200);
        assert_eq!(
            out,
            "Created:     2026-05-27 14:16:29 +0200 / Modified: 2026-05-27 14:16:29 +0200 / Accessed: 2026-05-27 14:16:48 +0200"
        );
    }

    #[test]
    fn inline_when_terminal_exactly_fits() {
        let pairs = sample_pairs();
        let out = render_label_value_pairs(&pairs, 114);
        assert!(!out.contains('\n'));
    }

    #[test]
    fn multiline_when_terminal_is_narrow() {
        let pairs = sample_pairs();
        let out = render_label_value_pairs(&pairs, 80);
        assert_eq!(
            out,
            "Created:     2026-05-27 14:16:29 +0200\n\
             Modified:    2026-05-27 14:16:29 +0200\n\
             Accessed:    2026-05-27 14:16:48 +0200"
        );
    }

    #[test]
    fn multiline_when_one_short_of_fitting() {
        let pairs = sample_pairs();
        let out = render_label_value_pairs(&pairs, 113);
        assert!(out.contains('\n'));
    }
}
