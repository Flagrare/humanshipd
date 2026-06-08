//! Headless credential verifier (the CLI sibling of the WASM verify page).
//!
//! Usage: `cargo run --example verify_credential -- <credential.c2pa> <document-file>`

use humanshipd_core::credential::{read_sidecar_with_text, Verdict};
use humanshipd_core::formats::extract_named;
use std::{env, fs, process};

fn verdict_line(v: &Verdict) -> String {
    let pct = |d: f64| format!("{:.0}%", d * 100.0);
    match v {
        Verdict::Invalid => "invalid (broken or forged signature)".to_string(),
        Verdict::ExactFile => "exact file (byte-exact hard binding)".to_string(),
        Verdict::SameContent { distance } => format!("same content — distance {}", pct(*distance)),
        Verdict::SameWriting { distance } => format!("same writing — distance {}", pct(*distance)),
        Verdict::Borderline { distance } => format!("borderline — distance {}", pct(*distance)),
        Verdict::NoMatch { distance } => match distance {
            Some(d) => format!("no match — distance {}", pct(*d)),
            None => "no match (no comparable content fingerprint)".to_string(),
        },
    }
}

fn main() {
    let mut args = env::args().skip(1);
    let (Some(manifest_path), Some(doc_path)) = (args.next(), args.next()) else {
        eprintln!("usage: verify_credential <credential.c2pa> <document-file>");
        process::exit(2);
    };

    let manifest = fs::read(&manifest_path).unwrap_or_else(|e| {
        eprintln!("cannot read {manifest_path}: {e}");
        process::exit(2);
    });
    let document = fs::read(&doc_path).unwrap_or_else(|e| {
        eprintln!("cannot read {doc_path}: {e}");
        process::exit(2);
    });

    // Extract the document's text per its format (.txt/.docx/.pdf) so a non-.txt
    // export of the same writing reaches the content-fingerprint engine.
    let text = extract_named(&doc_path, &document).unwrap_or_else(|e| {
        eprintln!("cannot extract text from {doc_path}: {e}");
        process::exit(2);
    });
    let readout = read_sidecar_with_text(&manifest, &document, &text).unwrap_or_else(|e| {
        eprintln!("verification error: {e}");
        process::exit(1);
    });

    println!("verdict       : {}", verdict_line(&readout.verdict));
    let t = &readout.trust;
    let trust = if t.trusted { "trusted" } else { "self-signed (untrusted)" };
    let identity = if t.identity_verified { "verified" } else { "unverified" };
    let stamp = t.timestamp.as_deref().unwrap_or("none");
    println!("trust         : signed={} {trust}; identity {identity}; timestamp {stamp}", t.signed);
    println!("valid         : {}", readout.valid);
    println!(
        "doc sha256    : {}",
        readout.record.document_binding.final_text_sha256
    );
    println!(
        "char_count    : {}",
        readout.record.document_binding.char_count
    );
    println!(
        "ai_dump_flags : {}",
        readout.record.evidence_flags.large_unkeyed_insertions
    );
    println!("claim         : {}", readout.claim);

    if !readout.valid {
        process::exit(1);
    }
}
