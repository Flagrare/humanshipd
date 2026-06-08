//! Real-corpus calibration for Decision 4's threshold, sourced online (no user
//! files needed): genuine human edits = successive Wikipedia revisions of one
//! article; "different" = unrelated articles. Reports 256-bit ISCC normalized
//! Hamming of the current revision vs each, so the edit→different gradient is
//! visible on real data.
//!
//! Run: cargo run --example corpus_calibration -- <corpus-dir>

fn main() {
    let dir = std::env::args().nth(1).unwrap_or_else(|| ".playwright-mcp/corpus".into());
    let read = |name: &str| std::fs::read_to_string(format!("{dir}/{name}")).expect(name);

    let base = read("rev_cur.txt");
    let base_d = digest(&code(&base));

    println!("base = current revision ({} chars)\n", base.chars().count());
    let rows = [
        ("edit: 1 revision back", "rev_back1.txt"),
        ("edit: 3 revisions back", "rev_back3.txt"),
        ("edit: 10 revisions back", "rev_back10.txt"),
        ("edit: 40 revisions back", "rev_back40.txt"),
        ("edit: 150 revisions back", "rev_back150.txt"),
        ("different: coral reef", "different_reef.txt"),
        ("different: chess history", "different_chess.txt"),
    ];
    for (label, f) in rows {
        let t = read(f);
        let d = hamming(&base_d, &digest(&code(&t)));
        println!("{label:28} {d:3}/256 = {:.3}", d as f64 / 256.0);
    }
}

fn code(t: &str) -> String {
    iscc_lib::gen_text_code_v0(t, 256).expect("iscc").iscc
}
fn digest(iscc: &str) -> Vec<u8> {
    let body = iscc.strip_prefix("ISCC:").unwrap_or(iscc);
    let bytes = b32_decode(body);
    bytes[bytes.len().saturating_sub(32)..].to_vec()
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
