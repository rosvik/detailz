> [!NOTE]
> This project is very much vibe coded.

# detailz

A small Rust CLI that prints detailed information about a file.

## Features

- File name, absolute path, size
- File type detection: magic-byte MIME via [`infer`](https://crates.io/crates/infer), or text-vs-binary with character encoding via [`chardetng`](https://crates.io/crates/chardetng)
- Permissions (ls-style + octal), owner and group, inode, hard-link count
- Created / modified / accessed timestamps
- Symlink target
- Extended attributes
- macOS BSD file flags (`hidden`, `uchg`, `schg`, …)
- SHA256 hash

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
Cargo.toml (/Users/you/detailz/Cargo.toml)
─────────────────────────────────────────
Type:        text
Encoding:    UTF-8
Size:        373 B (373 bytes)
Permissions: rw-r--r-- (0644)
Owner:       you:staff (501:20)
Inode:       102176003
Created:     2026-05-22 23:30:24 +0200
Modified:    2026-05-22 23:30:24 +0200
Accessed:    2026-05-22 23:30:25 +0200
SHA256:      659011f7cc1a10a40c9064d29144d956a4c9ceda4220443297d997af2e3ca532
```

## Tests

```sh
cargo test
```

Committed fixtures live in `tests/fixtures/`. Tests for state that git can't track (xattrs, BSD flags) construct their own files in a tempdir and are macOS-gated.
