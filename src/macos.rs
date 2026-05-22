pub fn format_bsd_flags(flags: u32) -> String {
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
