use std::fs;
use std::path::{Path, PathBuf};

use assert_cmd::Command;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn run(path: &Path) -> String {
    let output = Command::cargo_bin("detailz")
        .unwrap()
        .arg(path)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    String::from_utf8(output).expect("stdout was not utf-8")
}

#[test]
fn hello_txt_size_and_sha256() {
    let out = run(&fixture("hello.txt"));
    assert!(out.contains("Size:        6 B (6 bytes)"), "{out}");
    assert!(
        out.contains("5891b5b522d5df086d0ff0b110fbd9d21bb4fc7163af34d08286a2e846f6be03"),
        "{out}"
    );
}

#[test]
fn file_name_and_absolute_path() {
    let path = fixture("hello.txt");
    let out = run(&path);
    assert!(out.contains("File name:   hello.txt"), "{out}");
    assert!(
        out.contains(&format!("Path:        {}", path.canonicalize().unwrap().display())),
        "{out}"
    );
}

#[test]
fn utf8_text_detection() {
    let out = run(&fixture("utf8.txt"));
    assert!(out.contains("Type:        text"), "{out}");
    assert!(out.contains("Encoding:    UTF-8"), "{out}");
}

#[test]
fn latin1_detected_as_windows_1252() {
    let out = run(&fixture("latin1.txt"));
    assert!(out.contains("Type:        text"), "{out}");
    assert!(out.contains("Encoding:    windows-1252"), "{out}");
}

#[test]
fn utf16_without_bom_classified_as_binary() {
    let out = run(&fixture("utf16.txt"));
    assert!(out.contains("Type:        binary"), "{out}");
    assert!(!out.contains("Encoding:"), "{out}");
}

#[test]
fn utf16_with_bom_detected_as_text() {
    let out = run(&fixture("utf16le-bom.txt"));
    assert!(out.contains("Type:        text"), "{out}");
    assert!(out.contains("Encoding:    UTF-16LE"), "{out}");
}

#[test]
fn utf32_be_with_bom_detected_as_text() {
    // Without a BOM check, UTF-32BE's leading 00 00 FE FF would flag NUL → binary.
    let out = run(&fixture("utf32be-bom.txt"));
    assert!(out.contains("Type:        text"), "{out}");
    assert!(out.contains("Encoding:    UTF-32BE"), "{out}");
}

#[test]
fn binary_without_magic_is_binary() {
    let out = run(&fixture("binary.bin"));
    assert!(out.contains("Type:        binary"), "{out}");
    assert!(!out.contains("Encoding:"), "{out}");
}

#[test]
fn png_recognised_by_magic() {
    let out = run(&fixture("tiny.png"));
    assert!(out.contains("image/png"), "{out}");
    assert!(!out.contains("Type:        binary"), "{out}");
    assert!(!out.contains("Encoding:"), "{out}");
}

#[test]
fn regular_file_omits_symlink_and_xattr_sections() {
    let out = run(&fixture("hello.txt"));
    assert!(!out.contains("Symlink:"), "{out}");
    assert!(!out.contains("Extended attributes:"), "{out}");
}

#[test]
fn permissions_and_owner_lines_present() {
    let out = run(&fixture("hello.txt"));
    assert!(out.contains("Permissions: "), "{out}");
    assert!(out.contains("Owner:       "), "{out}");
    assert!(out.contains("Group:       "), "{out}");
    assert!(out.contains("Inode:       "), "{out}");
    assert!(out.contains("Hard links:  1"), "{out}");
}

#[test]
fn human_size_scales_to_kib() {
    let out = run(&fixture("two-kib.bin"));
    assert!(out.contains("Size:        2.00 KiB (2048 bytes)"), "{out}");
}

#[test]
fn symlink_shows_target() {
    let out = run(&fixture("working-link.txt"));
    assert!(out.contains("Symlink:     -> link-target.txt"), "{out}");
    assert!(!out.contains("(target does not exist)"), "{out}");
}

#[test]
fn broken_symlink_marks_target_missing() {
    let out = run(&fixture("broken-link.txt"));
    assert!(out.contains("Symlink:     -> nonexistent-target"), "{out}");
    assert!(out.contains("(target does not exist)"), "{out}");
    assert!(out.contains("(symlink target unreachable)"), "{out}");
}

#[cfg(target_os = "macos")]
#[test]
fn plain_xattr_is_listed() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("with-xattr.txt");
    fs::write(&path, "hi").unwrap();
    xattr::set(&path, "com.example.test", b"hello world").unwrap();

    let out = run(&path);
    assert!(out.contains("Extended attributes:"), "{out}");
    assert!(out.contains("com.example.test"), "{out}");
    assert!(out.contains("hello world"), "{out}");
}

#[cfg(target_os = "macos")]
#[test]
fn quarantine_xattr_is_decoded() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("quarantined.txt");
    fs::write(&path, "hi").unwrap();
    xattr::set(
        &path,
        "com.apple.quarantine",
        b"0083;5e8c5b22;Safari;F8E9B7C8-1234-5678-9ABC-DEF012345678",
    )
    .unwrap();

    let out = run(&path);
    assert!(out.contains("quarantine:"), "{out}");
    assert!(out.contains("Safari"), "{out}");
    assert!(out.contains("flags=0083"), "{out}");
    assert!(out.contains("F8E9B7C8-1234-5678-9ABC-DEF012345678"), "{out}");
}

#[cfg(target_os = "macos")]
#[test]
fn finder_tags_xattr_is_decoded() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("tagged.txt");
    fs::write(&path, "hi").unwrap();
    let tags = vec!["Important\n6".to_string(), "Work".to_string()];
    let mut plist_bytes = Vec::new();
    plist::to_writer_binary(&mut plist_bytes, &tags).unwrap();
    xattr::set(&path, "com.apple.metadata:_kMDItemUserTags", &plist_bytes).unwrap();

    let out = run(&path);
    assert!(out.contains("Finder tags:"), "{out}");
    assert!(out.contains("Important (red)"), "{out}");
    assert!(out.contains("Work"), "{out}");
}

#[cfg(target_os = "macos")]
#[test]
fn hidden_flag_is_reported() {
    let tmp = tempfile::tempdir().unwrap();
    let path = tmp.path().join("hidden.txt");
    fs::write(&path, "hi").unwrap();
    let status = std::process::Command::new("chflags")
        .arg("hidden")
        .arg(&path)
        .status()
        .expect("chflags");
    assert!(status.success(), "chflags failed");

    let out = run(&path);
    assert!(out.contains("Flags:       hidden"), "{out}");
}
