//! Validation spike for Decision 4: does the ISCC Text-Code match the *same
//! writing* across the kinds of differences that format conversion + text
//! extraction introduce (line wraps, double spaces, punctuation spacing), while
//! still differing for genuinely different writing?
//!
//! Run: cargo run --example iscc_crossformat

use humanshipd_core::fingerprint::text_iscc;

fn main() {
    let base = "The impact of renewable energy on global economies is significant. It reshapes how capital is allocated worldwide.";
    // PDF-style: hard line wraps and a blank line between paragraphs.
    let reflowed = "The impact of renewable energy\non global economies is significant.\n\nIt reshapes how capital is allocated worldwide.";
    // .docx/HTML-extraction-style: double spaces, space-before-punctuation, trailing space.
    let respaced = "The  impact of renewable energy on global economies is significant .  It reshapes how capital is allocated worldwide. ";
    // Same writing, one word changed (the "lightly edited" / borderline case).
    let edited = "The impact of renewable energy on global economies is enormous. It reshapes how capital is allocated worldwide.";
    // Genuinely different writing.
    let different = "A completely unrelated paragraph about marine biology, coral reefs, and the warm shallow oceans they thrive in.";

    let code = |t: &str| text_iscc(t).expect("iscc");
    let base_d = digest(&code(base));
    println!("base       {}", code(base));
    for (label, text) in [
        ("reflowed", reflowed),
        ("respaced", respaced),
        ("edited", edited),
        ("different", different),
    ] {
        let c = code(text);
        let d = hamming(&base_d, &digest(&c));
        println!("{label:10} {c}   hamming-to-base = {d}/64");
    }
}

/// Decode an `ISCC:...` Text-Code to its 64-bit (8-byte) digest, skipping the header.
fn digest(iscc: &str) -> Vec<u8> {
    let body = iscc.strip_prefix("ISCC:").unwrap_or(iscc);
    let bytes = b32_decode(body);
    bytes[bytes.len().saturating_sub(8)..].to_vec()
}

/// RFC 4648 base32 decode (no padding).
fn b32_decode(s: &str) -> Vec<u8> {
    const ALPHA: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    let (mut bits, mut nbits, mut out) = (0u32, 0u32, Vec::new());
    for c in s.bytes() {
        let Some(v) = ALPHA.iter().position(|&x| x == c) else { continue };
        bits = (bits << 5) | v as u32;
        nbits += 5;
        if nbits >= 8 {
            nbits -= 8;
            out.push((bits >> nbits) as u8);
        }
    }
    out
}

fn hamming(a: &[u8], b: &[u8]) -> u32 {
    a.iter().zip(b).map(|(x, y)| (x ^ y).count_ones()).sum()
}
