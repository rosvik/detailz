> [!NOTE]
> This project is very much vibe coded.

# detailz

A small Rust CLI that prints detailed information about a file.

## Features

- File name, absolute path, size (human-readable and exact bytes)
- SHA256 hash
- File type detection: magic-byte MIME via [`infer`](https://crates.io/crates/infer), or text-vs-binary with character encoding via [`chardetng`](https://crates.io/crates/chardetng) — supports UTF-8/16/32 BOMs and legacy encodings like `windows-1252`
- Permissions (ls-style + octal), owner, group, inode, hard-link count
- Created / modified / accessed timestamps
- Symlink target, with a warning when the target is missing
- Extended attributes, with decoders for macOS Finder tags and `com.apple.quarantine`
- macOS BSD file flags (`hidden`, `uchg`, `schg`, …)
- Colorized output that auto-strips when stdout isn't a TTY or `NO_COLOR` is set

## Build

```sh
cargo build --release
```

Binary is at `target/release/detailz`.

## Usage

```sh
detailz <FILE>
```

Example:

```
$ detailz Cargo.toml
File name:   Cargo.toml
Path:        /Users/you/detailz/Cargo.toml
Type:        text
Encoding:    UTF-8
Size:        321 B (321 bytes)
SHA256:      560ae6841faf205f30c3eb79aded4c49d07c8f5e131a2360e705236c288b9f0a
Permissions: rw-r--r-- (0644)
Owner:       you (501)
Group:       staff (20)
Inode:       102174220
Hard links:  1
Created:     2026-05-22 23:20:31 +0200
Modified:    2026-05-22 23:20:31 +0200
Accessed:    2026-05-22 23:20:31 +0200
```

`Symlink:`, `Extended attributes:`, and macOS `Flags:` sections only appear when relevant.

## Tests

```sh
cargo test
```

Committed fixtures live in `tests/fixtures/`. Tests for state that git can't track (xattrs, BSD flags) construct their own files in a tempdir and are macOS-gated.
