use humanshipd_core::canonical::sha256_hex;
use humanshipd_core::credential::{issue_sidecar, read_sidecar};
use humanshipd_core::session::{build_record, EditEvent, SessionInput, LARGE_UNKEYED_THRESHOLD};

/// Simulate incremental typing: ~5 chars per event, each driven by keystrokes.
fn typed_session(text: &str) -> SessionInput {
    let chars: Vec<char> = text.chars().collect();
    let mut events = Vec::new();
    let mut at = 0u64;
    for chunk in chars.chunks(5) {
        at += 300;
        events.push(EditEvent {
            at_ms: at,
            inserted_chars: chunk.len() as u64,
            deleted_chars: 0,
            keystrokes: chunk.len() as u64,
            at_offset: None,
        });
    }
    SessionInput {
        session_id: "s1".into(),
        surface_kind: "native-ax".into(),
        surface_app: "TextEdit".into(),
        final_text: text.into(),
        events,
    }
}

#[test]
fn round_trip_typed_session_verifies_and_binds_document() {
    // AT-1: build → issue (C2PA sidecar) → read is valid and binds the document hash.
    let text = "The quick brown fox jumps over the lazy dog.";
    let record = build_record(&typed_session(text));
    let manifest = issue_sidecar(&record, text.as_bytes()).expect("issue");

    let readout = read_sidecar(&manifest, text.as_bytes()).expect("read");
    assert!(readout.valid);
    assert_eq!(
        readout.record.document_binding.final_text_sha256,
        sha256_hex(text.as_bytes())
    );
    assert_eq!(readout.record.evidence_flags.large_unkeyed_insertions, 0);
    assert!(readout.claim.contains("incremental"));
}

#[test]
fn ai_dump_is_flagged_but_incremental_typing_is_not() {
    // AT-3: a large insertion with no keystrokes is flagged; typing the same text is not.
    let text = "Here is a long paragraph that was pasted in all at once from an AI tool.";
    assert!(text.chars().count() as u64 >= LARGE_UNKEYED_THRESHOLD);

    let dump = SessionInput {
        session_id: "s2".into(),
        surface_kind: "native-ax".into(),
        surface_app: "TextEdit".into(),
        final_text: text.into(),
        events: vec![EditEvent {
            at_ms: 100,
            inserted_chars: text.chars().count() as u64,
            deleted_chars: 0,
            keystrokes: 0,
            at_offset: None,
        }],
    };
    assert!(build_record(&dump).evidence_flags.large_unkeyed_insertions >= 1);
    assert_eq!(
        build_record(&typed_session(text))
            .evidence_flags
            .large_unkeyed_insertions,
        0
    );
}

#[test]
fn credential_never_contains_the_document_text() {
    // AT-4: the issued credential (what leaves the device) contains only hashes/
    // counts/timing — never the document text.
    let secret = "MARKER_xyzzy_42 the rest of my private essay text";
    let record = build_record(&typed_session(secret));
    let manifest = issue_sidecar(&record, secret.as_bytes()).expect("issue");

    let as_text = String::from_utf8_lossy(&manifest);
    assert!(!as_text.contains("MARKER_xyzzy_42"), "credential must not contain document text");
    assert!(!as_text.contains("private essay"));
}
