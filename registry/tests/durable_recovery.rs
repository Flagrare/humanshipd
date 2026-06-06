//! Durable Content Credentials, end to end: a credential whose sidecar is lost
//! can still be recovered from the document alone, via its ISCC fingerprint, and
//! then verified. This exercises Layer 1 (the credential) + Layer 3 (fingerprint
//! soft binding + registry) together.

use humanshipd_core::{build_record, credential, fingerprint, EditEvent, SessionInput};
use humanshipd_registry::Registry;

const MANUSCRIPT: &str = "This is an original manuscript paragraph, written for the \
    durable-recovery test. It runs several sentences so the ISCC content code has \
    enough material. The point being demonstrated is recovery without the sidecar.";

/// Incremental typing: ~5 chars per keystroke-backed event.
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
        session_id: "rec-1".into(),
        surface_kind: "native-ax".into(),
        surface_app: "TextEdit".into(),
        final_text: text.into(),
        events,
    }
}

#[test]
fn a_lost_credential_is_recovered_from_the_document_and_verifies() {
    // Author: write, issue a credential, publish it to the registry by fingerprint.
    let record = build_record(&typed_session(MANUSCRIPT));
    let credential_bytes =
        credential::issue_sidecar(&record, MANUSCRIPT.as_bytes()).expect("issue");
    let published_fingerprint = fingerprint::text_iscc(MANUSCRIPT).expect("iscc");

    let registry = Registry::new();
    registry.register(&published_fingerprint, credential_bytes);

    // Reader: has only the document — the sidecar is gone. Recompute the
    // fingerprint locally and recover the credential, then verify it.
    let recovered_fingerprint = fingerprint::text_iscc(MANUSCRIPT).expect("iscc");
    let recovered = registry
        .lookup(&recovered_fingerprint)
        .expect("credential recovered from registry");

    let readout = credential::read_sidecar(&recovered, MANUSCRIPT.as_bytes()).expect("read");
    assert!(readout.valid, "recovered credential must verify against the document");
    assert!(readout.claim.contains("incremental"));
}
