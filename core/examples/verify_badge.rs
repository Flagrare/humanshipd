//! Headless badge verifier (the CLI sibling of the WASM verify page).
//!
//! Usage: `cargo run --example verify_badge -- <badge.json>`

use humanshipd_core::{verify_badge, Badge};
use std::{env, fs, process};

fn main() {
    let Some(path) = env::args().nth(1) else {
        eprintln!("usage: verify_badge <badge.json>");
        process::exit(2);
    };

    let data = fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("cannot read {path}: {e}");
        process::exit(2);
    });
    let badge: Badge = serde_json::from_str(&data).unwrap_or_else(|e| {
        eprintln!("not a valid badge json: {e}");
        process::exit(2);
    });

    let result = verify_badge(&badge).unwrap_or_else(|e| {
        eprintln!("verification error: {e}");
        process::exit(1);
    });

    println!("signature_valid : {}", result.signature_valid);
    println!(
        "timestamp_valid : {} (present: {})",
        result.timestamp_valid, result.timestamp_present
    );
    if let (Some(authority), Some(when)) = (&result.authority, &result.gen_time) {
        println!("anchored_by     : {authority} @ {when}");
    }
    println!("document_sha256 : {}", result.document_sha256);
    println!("char_count      : {}", result.char_count);
    println!("ai_dump_flags   : {}", result.large_unkeyed_insertions);
    println!("claim           : {}", result.claim);

    if !result.signature_valid {
        process::exit(1);
    }
}
