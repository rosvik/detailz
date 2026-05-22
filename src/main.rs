mod format;
mod hash;
#[cfg(target_os = "macos")]
mod macos;
mod text;
mod xattrs;

use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::Path;

use anyhow::Result;
use clap::Parser;
use clio::ClioPath;
use colored::Colorize;

use crate::format::{fmt_time, format_mode, human_size, label};
use crate::hash::sha256_file;
use crate::text::{TextKind, detect_text_kind};
use crate::xattrs::decode_xattr;

#[derive(Parser, Debug)]
#[command(name = "detailz", about = "Print detailed information about a file")]
struct Args {
    /// File to inspect
    file: ClioPath,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let path: &Path = args.file.path();

    let symlink_meta = fs::symlink_metadata(path)?;
    let is_symlink = symlink_meta.file_type().is_symlink();
    let target_meta = fs::metadata(path).ok();

    let name = path
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string());
    let abs_path = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());
    println!(
        "{} {}{}{}",
        name.bold().green(),
        "(".dimmed(),
        abs_path.display().to_string().dimmed(),
        ")".dimmed()
    );
    println!(
        "{}",
        "─".repeat(name.len() + abs_path.display().to_string().len() + 3)
    );

    if let Some(m) = &target_meta {
        if m.is_file() {
            if let Ok(Some(t)) = infer::get_from_path(path) {
                println!(
                    "{}{} ({})",
                    label("Type:"),
                    t.mime_type(),
                    t.extension().dimmed()
                );
            } else if let Ok(kind) = detect_text_kind(path) {
                match kind {
                    TextKind::Binary => println!("{}{}", label("Type:"), "binary".yellow()),
                    TextKind::Text(enc) => {
                        println!("{}{}", label("Type:"), "text".green());
                        println!("{}{}", label("Encoding:"), enc);
                    }
                }
            }
        } else if m.is_dir() {
            println!("{}directory", label("Type:"));
        }
    }

    match &target_meta {
        Some(m) => println!(
            "{}{} ({})",
            label("Size:"),
            human_size(m.len()),
            format!("{} bytes", m.len()).dimmed()
        ),
        None => println!(
            "{}{}",
            label("Size:"),
            "(symlink target unreachable)".dimmed()
        ),
    }

    if let Some(m) = &target_meta {
        let mode = m.permissions().mode();
        println!(
            "{}{} ({})",
            label("Permissions:"),
            format_mode(mode),
            format!("{:04o}", mode & 0o7777).dimmed()
        );

        let uid = m.uid();
        let user = uzers::get_user_by_uid(uid)
            .map(|u| u.name().to_string_lossy().into_owned())
            .unwrap_or_else(|| "?".to_string());
        let gid = m.gid();
        let group = uzers::get_group_by_gid(gid)
            .map(|g| g.name().to_string_lossy().into_owned())
            .unwrap_or_else(|| "?".to_string());
        println!(
            "{}{}:{} ({})",
            label("Owner:"),
            user,
            group,
            format!("{}:{}", uid, gid).dimmed()
        );

        println!("{}{}", label("Inode:"), m.ino());
        if m.nlink() > 1 {
            println!("{}{}", label("Hard links:"), m.nlink());
        }
    }

    #[cfg(target_os = "macos")]
    if let Some(m) = &target_meta {
        use std::os::macos::fs::MetadataExt as MacMetadataExt;
        let flags = m.st_flags();
        if flags != 0 {
            println!(
                "{}{}",
                label("Flags:"),
                crate::macos::format_bsd_flags(flags).yellow()
            );
        }
    }

    if let Some(m) = &target_meta {
        if let Ok(t) = m.created() {
            println!("{}{}", label("Created:"), fmt_time(t));
        }
        if let Ok(t) = m.modified() {
            println!("{}{}", label("Modified:"), fmt_time(t));
        }
        if let Ok(t) = m.accessed() {
            println!("{}{}", label("Accessed:"), fmt_time(t));
        }
    }

    if is_symlink {
        let target = fs::read_link(path)?;
        println!(
            "{}-> {}",
            label("Symlink:"),
            target.display().to_string().cyan()
        );
        if target_meta.is_none() {
            println!("             {}", "(target does not exist)".red());
        }
    }

    if target_meta.as_ref().filter(|m| m.is_file()).is_some() {
        println!("{}{}", label("SHA256:"), sha256_file(path)?.dimmed());
    }

    let xattrs: Vec<_> = xattr::list(path)?.collect();
    if !xattrs.is_empty() {
        println!("{}", "Extended attributes:".bold().cyan());
        for attr in xattrs {
            let attr_name = attr.to_string_lossy();
            match xattr::get(path, &attr)? {
                Some(value) => println!(
                    "  {} = {}",
                    attr_name.magenta(),
                    decode_xattr(&attr_name, &value)
                ),
                None => println!("  {} = {}", attr_name.magenta(), "(empty)".dimmed()),
            }
        }
    }

    Ok(())
}
