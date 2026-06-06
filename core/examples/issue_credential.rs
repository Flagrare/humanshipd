//! Headless credential issuer (the sibling of `verify_credential`).
//!
//! Simulates a realistic editing session against a text buffer — incremental
//! typing, one paste, then a mid-document revisit — so the resulting credential
//! exercises the banded report AND the position-aware fingerprint (the revisit
//! shows as a dip back to an earlier offset). Writes a `.c2pa` sidecar + document.
//!
//! Usage: `cargo run --example issue_credential -- <out-dir>`
//! Writes `<out-dir>/credential.c2pa` and `<out-dir>/document.txt`.

use humanshipd_core::credential::issue_sidecar;
use humanshipd_core::session::{build_record, EditEvent, SessionInput};
use std::{env, fs, process};

/// A tiny editing simulator: applies inserts at character offsets to a buffer and
/// records a real `EditEvent` (with `at_offset`) for each, so the document and the
/// event stream are mutually consistent.
struct Editor {
    buf: Vec<char>,
    at_ms: u64,
    events: Vec<EditEvent>,
}

impl Editor {
    fn new() -> Self {
        Self { buf: Vec::new(), at_ms: 0, events: Vec::new() }
    }

    fn insert(&mut self, offset: usize, text: &str, keyed: bool, gap_ms: u64) {
        self.at_ms += gap_ms;
        let chars: Vec<char> = text.chars().collect();
        let n = chars.len() as u64;
        self.buf.splice(offset..offset, chars);
        self.events.push(EditEvent {
            at_ms: self.at_ms,
            inserted_chars: n,
            deleted_chars: 0,
            keystrokes: if keyed { n } else { 0 },
            at_offset: Some(offset as u64),
        });
    }

    /// Type `text` at the end in ≈5-char bursts.
    fn type_bursts(&mut self, text: &str) {
        for chunk in text.chars().collect::<Vec<_>>().chunks(5) {
            let chunk: String = chunk.iter().collect();
            let end = self.buf.len();
            self.insert(end, &chunk, true, 350);
        }
    }

    fn document(&self) -> String {
        self.buf.iter().collect()
    }
}

fn main() {
    let out_dir = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage: issue_credential <out-dir>");
        process::exit(2);
    });

    let mut ed = Editor::new();
    ed.type_bursts("Renewable energy is reshaping how economies allocate capital. ");
    // A pasted block appended without keystrokes (the AI-dump signal).
    let paste_at = ed.buf.len();
    ed.insert(
        paste_at,
        "Countries investing in renewable technologies stand to gain a competitive edge. ",
        false,
        1500,
    );
    ed.type_bursts("That shift is already visible in grid-scale procurement decisions.");
    // A mid-document revisit: insert a word back near the start (offset 10) — this
    // is the case a position-aware fingerprint reveals that a length-only one cannot.
    ed.insert(10, "clean ", true, 1200);

    let document = ed.document();
    let record = build_record(&SessionInput {
        session_id: "demo-mixed-0001".into(),
        surface_kind: "native-ax".into(),
        surface_app: "TextEdit".into(),
        final_text: document.clone(),
        events: ed.events,
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
