//! Layer 2: embed a credential *inside* plain text using non-rendering Unicode
//! variation selectors, per C2PA 2.4 Appendix A.8 ("Embedding Manifests into
//! Unstructured Text"). The bytes are invisible and survive copy-paste, so the
//! credential can travel with the text itself.
//!
//! Each manifest byte maps to exactly one variation selector:
//! - 0x00..=0x0F → U+FE00..=U+FE0F (16 selectors)
//! - 0x10..=0xFF → U+E0100..=U+E01EF (240 selectors)
//!
//! A leading ZWNBSP (U+FEFF) marks the start of the embedded payload.

/// Marks the start of the embedded payload within the text.
const MARKER: char = '\u{FEFF}';

fn byte_to_selector(byte: u8) -> char {
    let code = if byte < 16 {
        0xFE00 + byte as u32
    } else {
        0xE0100 + (byte as u32 - 16)
    };
    char::from_u32(code).expect("valid variation selector code point")
}

fn selector_to_byte(c: char) -> Option<u8> {
    let code = c as u32;
    if (0xFE00..=0xFE0F).contains(&code) {
        Some((code - 0xFE00) as u8)
    } else if (0xE0100..=0xE01EF).contains(&code) {
        Some((code - 0xE0100 + 16) as u8)
    } else {
        None
    }
}

/// Append `manifest` to `text` as invisible variation selectors (after a marker).
pub fn embed(text: &str, manifest: &[u8]) -> String {
    let mut out = String::with_capacity(text.len() + 1 + manifest.len() * 3);
    out.push_str(text);
    out.push(MARKER);
    for &byte in manifest {
        out.push(byte_to_selector(byte));
    }
    out
}

/// Extract an embedded manifest, if present (bytes after the marker).
pub fn extract(s: &str) -> Option<Vec<u8>> {
    let marker_idx = s.rfind(MARKER)?;
    let after = &s[marker_idx + MARKER.len_utf8()..];
    let bytes: Option<Vec<u8>> = after.chars().map(selector_to_byte).collect();
    match bytes {
        Some(b) if !b.is_empty() => Some(b),
        _ => None,
    }
}

/// Return the visible text with any embedded payload (marker + selectors) removed.
pub fn strip(s: &str) -> String {
    match s.rfind(MARKER) {
        Some(idx) => s[..idx].to_string(),
        None => s.to_string(),
    }
}
