//! Headless credential verifier (the CLI sibling of the WASM verify page).
//!
//! Usage: `cargo run --example verify_credential -- <credential.c2pa> <document-file>`

use humanshipd_core::credential::read_sidecar;
use std::{env, fs, process};

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

    let readout = read_sidecar(&manifest, &document).unwrap_or_else(|e| {
        eprintln!("verification error: {e}");
        process::exit(1);
    });

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
