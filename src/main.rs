use std::fs;
use std::io::Read;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::Path;
use std::time::SystemTime;

use anyhow::Result;
use chrono::{DateTime, Local};
use clap::Parser;
use clio::ClioPath;
use colored::{ColoredString, Colorize};
use sha2::{Digest, Sha256};

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
    println!("{}{}", label("File name:"), name.bold());
    let abs_path = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());
    println!("{}{}", label("Path:"), abs_path.display());

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

    match target_meta.as_ref().filter(|m| m.is_file()) {
        Some(_) => println!("{}{}", label("SHA256:"), sha256_file(path)?.dimmed()),
        None if target_meta.as_ref().is_some_and(|m| m.is_dir()) => {
            println!("{}{}", label("SHA256:"), "(directory)".dimmed())
        }
        None => println!("{}{}", label("SHA256:"), "(not a regular file)".dimmed()),
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
        println!("{}{} ({})", label("Owner:"), user, uid.to_string().dimmed());

        let gid = m.gid();
        let group = uzers::get_group_by_gid(gid)
            .map(|g| g.name().to_string_lossy().into_owned())
            .unwrap_or_else(|| "?".to_string());
        println!(
            "{}{} ({})",
            label("Group:"),
            group,
            gid.to_string().dimmed()
        );

        println!("{}{}", label("Inode:"), m.ino());
        println!("{}{}", label("Hard links:"), m.nlink());
    }

    #[cfg(target_os = "macos")]
    if let Some(m) = &target_meta {
        use std::os::macos::fs::MetadataExt as MacMetadataExt;
        let flags = m.st_flags();
        if flags != 0 {
            println!("{}{}", label("Flags:"), format_bsd_flags(flags).yellow());
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

fn label(s: &str) -> ColoredString {
    format!("{:<13}", s).bold().cyan()
}

enum TextKind {
    Binary,
    Text(String),
}

fn detect_bom(bytes: &[u8]) -> Option<&'static str> {
    // UTF-32 BOMs must be checked before UTF-16: UTF-32LE starts with FF FE 00 00
    // which shares its first two bytes with UTF-16LE.
    if bytes.starts_with(b"\x00\x00\xFE\xFF") {
        Some("UTF-32BE")
    } else if bytes.starts_with(b"\xFF\xFE\x00\x00") {
        Some("UTF-32LE")
    } else if bytes.starts_with(b"\xFE\xFF") {
        Some("UTF-16BE")
    } else if bytes.starts_with(b"\xFF\xFE") {
        Some("UTF-16LE")
    } else if bytes.starts_with(b"\xEF\xBB\xBF") {
        Some("UTF-8")
    } else {
        None
    }
}

fn detect_text_kind(path: &Path) -> std::io::Result<TextKind> {
    let mut file = fs::File::open(path)?;
    let mut sample = vec![0u8; 8192];
    let n = file.read(&mut sample)?;
    sample.truncate(n);

    if let Some(enc) = detect_bom(&sample) {
        return Ok(TextKind::Text(enc.to_string()));
    }

    if sample.contains(&0) {
        return Ok(TextKind::Binary);
    }

    let utf8_ok = match std::str::from_utf8(&sample) {
        Ok(_) => true,
        Err(e) => e.error_len().is_none() && sample.len() - e.valid_up_to() <= 3,
    };
    if utf8_ok {
        return Ok(TextKind::Text("UTF-8".to_string()));
    }

    let mut detector = chardetng::EncodingDetector::new();
    detector.feed(&sample, true);
    let enc = detector.guess(None, true);
    Ok(TextKind::Text(enc.name().to_string()))
}

fn sha256_file(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

fn human_size(bytes: u64) -> String {
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

fn fmt_time(t: SystemTime) -> String {
    let dt: DateTime<Local> = t.into();
    dt.format("%Y-%m-%d %H:%M:%S %z").to_string()
}

fn format_mode(mode: u32) -> String {
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

#[cfg(target_os = "macos")]
fn format_bsd_flags(flags: u32) -> String {
    let mapping: &[(u32, &str)] = &[
        (libc::UF_NODUMP, "nodump"),
        (libc::UF_IMMUTABLE, "uchg"),
        (libc::UF_APPEND, "uappnd"),
        (libc::UF_OPAQUE, "opaque"),
        (libc::UF_COMPRESSED, "compressed"),
        (libc::UF_TRACKED, "tracked"),
        (0x0000_0080, "datavault"),
        (libc::UF_HIDDEN, "hidden"),
        (libc::SF_ARCHIVED, "arch"),
        (libc::SF_IMMUTABLE, "schg"),
        (libc::SF_APPEND, "sappnd"),
        (0x0008_0000, "restricted"),
        (0x0010_0000, "sunlnk"),
    ];
    let known: u32 = mapping.iter().map(|(b, _)| b).fold(0, |a, b| a | b);
    let mut parts: Vec<&str> = mapping
        .iter()
        .filter(|(bit, _)| flags & bit != 0)
        .map(|(_, name)| *name)
        .collect();
    let unknown = flags & !known;
    let mut s = parts.join(", ");
    if unknown != 0 {
        if !parts.is_empty() {
            s.push_str(", ");
        }
        s.push_str(&format!("unknown(0x{:x})", unknown));
        parts.push("");
    }
    if s.is_empty() {
        format!("0x{:x}", flags)
    } else {
        s
    }
}

fn decode_xattr(name: &str, value: &[u8]) -> String {
    match name {
        "com.apple.metadata:_kMDItemUserTags" => {
            decode_finder_tags(value).unwrap_or_else(|| display_xattr_value(value))
        }
        "com.apple.quarantine" => {
            decode_quarantine(value).unwrap_or_else(|| display_xattr_value(value))
        }
        _ => display_xattr_value(value),
    }
}

fn decode_finder_tags(value: &[u8]) -> Option<String> {
    let parsed: plist::Value = plist::from_bytes(value).ok()?;
    let arr = parsed.as_array()?;
    let tags: Vec<String> = arr
        .iter()
        .filter_map(|v| v.as_string())
        .map(|s| {
            let (name, color) = s.split_once('\n').unwrap_or((s, ""));
            let color_name = match color {
                "1" => Some("gray"),
                "2" => Some("green"),
                "3" => Some("purple"),
                "4" => Some("blue"),
                "5" => Some("yellow"),
                "6" => Some("red"),
                "7" => Some("orange"),
                _ => None,
            };
            match color_name {
                Some(c) => format!("{} ({})", name, c),
                None => name.to_string(),
            }
        })
        .collect();
    if tags.is_empty() {
        return None;
    }
    Some(format!("{} [{}]", "Finder tags:".bold(), tags.join(", ")))
}

fn decode_quarantine(value: &[u8]) -> Option<String> {
    let s = std::str::from_utf8(value).ok()?;
    let parts: Vec<&str> = s.split(';').collect();
    if parts.len() < 3 {
        return None;
    }
    let flags = parts[0];
    let timestamp = u64::from_str_radix(parts[1], 16).ok()?;
    let agent = parts[2];
    let event = parts.get(3).copied().unwrap_or("");
    let dt: DateTime<Local> = DateTime::from_timestamp(timestamp as i64, 0)?.with_timezone(&Local);
    let mut out = format!(
        "{} flags={} at={} by={}",
        "quarantine:".yellow().bold(),
        flags,
        dt.format("%Y-%m-%d %H:%M:%S %z"),
        agent.yellow()
    );
    if !event.is_empty() {
        out.push_str(&format!(" event={}", event.dimmed()));
    }
    Some(out)
}

fn display_xattr_value(v: &[u8]) -> String {
    let printable = std::str::from_utf8(v).ok().filter(|s| {
        s.chars()
            .all(|c| !c.is_control() || c == '\n' || c == '\t' || c == '\r')
    });
    match printable {
        Some(s) => format!("{} ({} bytes)", s, v.len()),
        None => {
            let preview: String = v.iter().take(32).map(|b| format!("{:02x}", b)).collect();
            if v.len() > 32 {
                format!("0x{}... ({} bytes)", preview, v.len())
            } else {
                format!("0x{} ({} bytes)", preview, v.len())
            }
        }
    }
}
