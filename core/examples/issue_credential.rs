//! Headless credential issuer (the sibling of `verify_credential`).
//!
//! Builds a record from a synthetic mixed session (typing + one paste) so the
//! resulting credential exercises the banded provenance report, then writes a
//! `.c2pa` sidecar and its document next to each other.
//!
//! Usage: `cargo run --example issue_credential -- <out-dir>`
//! Writes `<out-dir>/credential.c2pa` and `<out-dir>/document.txt`.

use humanshipd_core::credential::issue_sidecar;
use humanshipd_core::session::{build_record, EditEvent, SessionInput};
use std::{env, fs, process};

fn main() {
    let out_dir = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage: issue_credential <out-dir>");
        process::exit(2);
    });

    // A mostly-typed document with one pasted block in the middle, so the report
    // shows distinct Typed and Pasted bands.
    let typed_a = "Renewable energy is reshaping how economies allocate capital. ";
    let pasted = "Countries investing in renewable technologies stand to gain a competitive edge. ";
    let typed_b = "That shift is already visible in grid-scale procurement decisions.";
    let document = format!("{typed_a}{pasted}{typed_b}");

    // Incremental typing (≈5-char bursts) → one paste → more typing, with a small
    // deletion, so the timeline has enough points to show a real shape.
    let mut events = Vec::new();
    let mut at = 0u64;
    let typed_bursts = |events: &mut Vec<EditEvent>, at: &mut u64, text: &str| {
        let chars: Vec<char> = text.chars().collect();
        for chunk in chars.chunks(5) {
            *at += 350;
            events.push(EditEvent {
                at_ms: *at,
                inserted_chars: chunk.len() as u64,
                deleted_chars: 0,
                keystrokes: chunk.len() as u64,
            });
        }
    };

    typed_bursts(&mut events, &mut at, typed_a);
    at += 1500;
    events.push(EditEvent {
        at_ms: at,
        inserted_chars: pasted.chars().count() as u64,
        deleted_chars: 0,
        keystrokes: 0,
    });
    at += 800;
    typed_bursts(&mut events, &mut at, typed_b);

    let record = build_record(&SessionInput {
        session_id: "demo-mixed-0001".into(),
        surface_kind: "native-ax".into(),
        surface_app: "TextEdit".into(),
        final_text: document.clone(),
        events,
    });

    let manifest = issue_sidecar(&record, document.as_bytes()).unwrap_or_else(|e| {
        eprintln!("issue error: {e}");
        process::exit(1);
    });

    fs::create_dir_all(&out_dir).unwrap_or_else(|e| {
        eprintln!("cannot create {out_dir}: {e}");
        process::exit(2);
    });
    let cred_path = format!("{out_dir}/credential.c2pa");
    let doc_path = format!("{out_dir}/document.txt");
    fs::write(&cred_path, &manifest).expect("write credential");
    fs::write(&doc_path, document.as_bytes()).expect("write document");

    println!("wrote {cred_path}");
    println!("wrote {doc_path}");
}
