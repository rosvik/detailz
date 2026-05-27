use std::fs::Metadata;

use crate::format::{fmt_time, print_label_value_pairs};

pub fn print_dates(metadata: &Metadata, terminal_width: usize) {
    let mut pairs: Vec<(&str, String)> = Vec::new();
    if let Ok(t) = metadata.created() {
        pairs.push(("Created:", fmt_time(t)));
    }
    if let Ok(t) = metadata.modified() {
        pairs.push(("Modified:", fmt_time(t)));
    }
    if let Ok(t) = metadata.accessed() {
        pairs.push(("Accessed:", fmt_time(t)));
    }
    print_label_value_pairs(&pairs, terminal_width);
}
