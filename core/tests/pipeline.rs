use humanshipd_core::badge::{anchor_badge, sign_record};
use humanshipd_core::canonical::sha256_hex;
use humanshipd_core::session::{build_record, EditEvent, SessionInput, LARGE_UNKEYED_THRESHOLD};
use humanshipd_core::signing::KeyPair;
use humanshipd_core::timestamp::LocalTsa;
use humanshipd_core::verify::verify_badge;

fn client_key() -> KeyPair {
    KeyPair::from_seed(&[7u8; 32])
}

fn local_tsa() -> LocalTsa {
    LocalTsa::new(&[9u8; 32], "humanshipd:local-poc", "2026-06-05T12:00:00Z")
}

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
        });
    }
    SessionInput {
        session_id: "s1".into(),
        surface_kind: "gdocs".into(),
        surface_app: "docs.google.com".into(),
        final_text: text.into(),
        events,
    }
}

#[test]
fn round_trip_typed_session_verifies_and_matches_document() {
    // AT-1: build → sign → anchor → verify is valid and binds the document hash.
    let text = "The quick brown fox jumps over the lazy dog.";
    let record = build_record(&typed_session(text));
    let badge = anchor_badge(sign_record(record, &client_key()).unwrap(), &local_tsa()).unwrap();

    let result = verify_badge(&badge).unwrap();
    assert!(result.signature_valid);
    assert!(result.timestamp_valid);
    assert_eq!(result.document_sha256, sha256_hex(text.as_bytes()));
    assert_eq!(result.large_unkeyed_insertions, 0);
    assert!(result.claim.contains("incremental, human-like process"));
}

#[test]
fn ai_dump_is_flagged_but_incremental_typing_is_not() {
    // AT-3: a large insertion with no keystrokes is flagged; typing the same text is not.
    let text = "Here is a long paragraph that was pasted in all at once from an AI tool.";
    assert!(text.chars().count() as u64 >= LARGE_UNKEYED_THRESHOLD);

    let dump = SessionInput {
        session_id: "s2".into(),
        surface_kind: "gdocs".into(),
        surface_app: "docs.google.com".into(),
        final_text: text.into(),
        events: vec![EditEvent {
            at_ms: 100,
            inserted_chars: text.chars().count() as u64,
            deleted_chars: 0,
            keystrokes: 0,
        }],
    };
    let dump_badge =
        anchor_badge(sign_record(build_record(&dump), &client_key()).unwrap(), &local_tsa())
            .unwrap();
    let dump_result = verify_badge(&dump_badge).unwrap();
    assert!(dump_result.large_unkeyed_insertions >= 1);
    assert!(dump_result.claim.contains("WARNING"));

    let typed_record = build_record(&typed_session(text));
    assert_eq!(typed_record.evidence_flags.large_unkeyed_insertions, 0);
}

#[test]
fn record_and_badge_never_contain_the_document_text() {
    // AT-4: the produced badge contains only hashes/counts/timing — never content.
    let secret = "MARKER_xyzzy_42 the rest of my private essay text";
    let record = build_record(&typed_session(secret));
    let badge = anchor_badge(sign_record(record, &client_key()).unwrap(), &local_tsa()).unwrap();

    let serialized = serde_json::to_string(&badge).unwrap();
    assert!(
        !serialized.contains("MARKER_xyzzy_42"),
        "badge must not contain the document text"
    );
    assert!(!serialized.contains("private essay"));
}
