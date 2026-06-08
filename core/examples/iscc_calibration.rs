//! Calibration spike for Decision 4's threshold: how cleanly does the ISCC
//! Text-Code separate "same writing / lightly edited / different", and does a
//! larger code (256-bit) give finer resolution than 64-bit? Reports normalized
//! Hamming distance (bits-different / total-bits) so 64 and 256 are comparable.
//!
//! NOTE: this is synthetic (a handful of hand-written variants), so the numbers
//! are indicative, not a calibrated threshold — real calibration needs a corpus.
//!
//! Run: cargo run --example iscc_calibration

fn main() {
    let base = "Renewable energy is steadily reshaping how the world's economies allocate capital, because falling costs make solar and wind cheaper than fossil fuels in most regions, and countries that invest early gain a durable competitive edge in the low-carbon industries of the coming decades.";
    let reformatted = "Renewable energy is steadily reshaping how the world's economies\nallocate capital,  because falling costs make solar and wind cheaper than fossil fuels in most regions , and countries that invest early gain a durable competitive edge in the low-carbon industries of the coming decades.";
    let edited_1word = base.replace("steadily", "quietly");
    let edited_few = base
        .replace("steadily", "quietly")
        .replace("falling", "plunging")
        .replace("durable", "lasting")
        .replace("coming decades", "decades ahead");
    let different1 = "Coral reefs are among the most biodiverse ecosystems on the planet, sheltering a quarter of all marine species despite covering a tiny fraction of the ocean floor, and their slow growth makes recovery from bleaching events painfully gradual.";
    let different2 = "Medieval guilds tightly controlled who could practice a craft, setting prices, training apprentices over many years, and guarding trade secrets so jealously that an artisan who left town often could not legally work elsewhere.";

    for bits in [64u32, 256u32] {
        println!("=== {bits}-bit ISCC Text-Code (normalized Hamming vs base) ===");
        let base_d = digest(&code(base, bits), bits);
        for (label, t) in [
            ("reformatted", reformatted),
            ("edited 1 word", edited_1word.as_str()),
            ("edited ~5 words", edited_few.as_str()),
            ("different (reefs)", different1),
            ("different (guilds)", different2),
        ] {
            let d = hamming(&base_d, &digest(&code(t, bits), bits));
            println!("  {label:20} {d:3}/{bits}   = {:.3}", d as f64 / bits as f64);
        }
    }
}

fn code(t: &str, bits: u32) -> String {
    iscc_lib::gen_text_code_v0(t, bits).expect("iscc").iscc
}
fn digest(iscc: &str, bits: u32) -> Vec<u8> {
    let body = iscc.strip_prefix("ISCC:").unwrap_or(iscc);
    let bytes = b32_decode(body);
    let n = (bits / 8) as usize;
    bytes[bytes.len().saturating_sub(n)..].to_vec()
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
