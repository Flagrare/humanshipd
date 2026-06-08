//! Validation spike for the PDF leg of Decision 4: extract text from a real
//! Google-exported PDF and check its ISCC matches the `.txt` export of the same
//! document. PDF text extraction is the hard format — this measures how close it
//! lands.
//!
//! Run: cargo run --example pdf_extract -- <doc.pdf> <doc.txt>

use humanshipd_core::fingerprint::text_iscc;

fn main() {
    let mut args = std::env::args().skip(1);
    let (pdf_path, txt_path) = (args.next().expect("pdf path"), args.next().expect("txt path"));

    let pdf_text = pdf_extract::extract_text(&pdf_path).expect("extract pdf text");
    let txt = std::fs::read_to_string(&txt_path).expect("read txt");

    println!("pdf-extracted ({} chars): {:?}", pdf_text.chars().count(), prev(&pdf_text));
    println!("txt-export    ({} chars): {:?}", txt.chars().count(), prev(&txt));

    let iscc_pdf = text_iscc(&pdf_text).expect("iscc pdf");
    let iscc_txt = text_iscc(&txt).expect("iscc txt");
    println!("\nISCC(pdf) = {iscc_pdf}");
    println!("ISCC(txt) = {iscc_txt}");
    println!("hamming(pdf, txt) = {}/64", hamming(&digest(&iscc_pdf), &digest(&iscc_txt)));
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
