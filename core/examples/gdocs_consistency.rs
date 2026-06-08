//! Validation spike for Decision 2's reconcile + consistency gates: run our real
//! `gdocs` parser on a real full `/revisions/load` committed changelog, and check
//! the reconstructed document text matches the document's actual text (a fresh
//! `.txt` export). If they match, the committed history reconciles with reality
//! and our parser reconstructs real Google output correctly.
//!
//! Run: cargo run --example gdocs_consistency -- <changelog.json> <doc.txt>

use humanshipd_core::fingerprint::text_iscc;
use humanshipd_core::session_from_changelog;

fn main() {
    let mut args = std::env::args().skip(1);
    let (changelog_path, txt_path) =
        (args.next().expect("changelog path"), args.next().expect("txt path"));

    let body = std::fs::read_to_string(&changelog_path).expect("read changelog");
    let input = session_from_changelog(&body, "gdoc", "docs.google.com").expect("parse changelog");
    let actual = std::fs::read_to_string(&txt_path).expect("read txt");

    println!("reconstructed ({} chars): {:?}", input.final_text.chars().count(), prev(&input.final_text));
    println!("actual .txt   ({} chars): {:?}", actual.chars().count(), prev(&actual));
    println!("edit events: {}", input.events.len());

    let iscc_recon = text_iscc(&input.final_text).expect("iscc recon");
    let iscc_actual = text_iscc(&actual).expect("iscc actual");
    println!("\nISCC(reconstructed) = {iscc_recon}");
    println!("ISCC(actual .txt)   = {iscc_actual}");
    println!("hamming = {}/64   <- 0 means reconstruction == real document", hamming(&digest(&iscc_recon), &digest(&iscc_actual)));
}

fn prev(s: &str) -> String {
    s.trim_start_matches('\u{feff}').chars().take(75).collect()
}
fn digest(iscc: &str) -> Vec<u8> {
    let body = iscc.strip_prefix("ISCC:").unwrap_or(iscc);
    let bytes = b32_decode(body);
    bytes[bytes.len().saturating_sub(8)..].to_vec()
}
fn b32_decode(s: &str) -> Vec<u8> {
    const ALPHA: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    let (mut bits, mut nbits, mut out) = (0u32, 0u32, Vec::new());
    for c in s.bytes() {
        let Some(v) = ALPHA.iter().position(|&x| x == c) else { continue };
        bits = (bits << 5) | v as u32;
        nbits += 5;
        if nbits >= 8 { nbits -= 8; out.push((bits >> nbits) as u8); }
    }
    out
}
fn hamming(a: &[u8], b: &[u8]) -> u32 {
    a.iter().zip(b).map(|(x, y)| (x ^ y).count_ones()).sum()
}
