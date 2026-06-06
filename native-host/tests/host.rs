use base64::Engine;
use humanshipd_core::canonical::sha256_hex;
use humanshipd_core::credential::read_sidecar;
use humanshipd_host::handler::process;
use humanshipd_host::messages::{Request, Response};
use humanshipd_host::protocol::{read_message, write_message};
use std::io::Cursor;

#[test]
fn message_frame_round_trips_and_reports_eof() {
    let mut buf = Vec::new();
    write_message(&mut buf, br#"{"hello":1}"#).unwrap();

    let mut cursor = Cursor::new(buf);
    let got = read_message(&mut cursor).unwrap().expect("a message");
    assert_eq!(got, br#"{"hello":1}"#);
    assert!(read_message(&mut cursor).unwrap().is_none(), "clean EOF");
}

#[test]
fn ping_returns_pong() {
    let request: Request = serde_json::from_str(r#"{"type":"ping"}"#).unwrap();
    assert!(matches!(process(request), Response::Pong { .. }));
}

#[test]
fn issue_request_produces_a_verifying_credential() {
    let json = r#"{
        "type": "issue",
        "session_id": "s1",
        "surface_kind": "native-ax",
        "surface_app": "TextEdit",
        "final_text": "hello world",
        "events": [{"at_ms": 0, "inserted_chars": 11, "deleted_chars": 0, "keystrokes": 11}]
    }"#;
    let request: Request = serde_json::from_str(json).unwrap();

    match process(request) {
        Response::Credential { manifest_b64 } => {
            let manifest = base64::engine::general_purpose::STANDARD
                .decode(manifest_b64)
                .expect("base64");
            let readout = read_sidecar(&manifest, b"hello world").expect("read");
            assert!(readout.valid);
            assert_eq!(
                readout.record.document_binding.final_text_sha256,
                sha256_hex(b"hello world")
            );
        }
        other => panic!("expected a credential response, got {other:?}"),
    }
}

#[test]
fn extension_shaped_session_with_a_paste_flags_ai_dump() {
    // Mirrors the browser smoke test: a typed prefix followed by a large paste
    // (keystroke-less) must yield a verifying credential whose claim warns of the
    // un-keyed insertion — the AI-dump signal end to end.
    let json = r#"{
        "type": "issue",
        "session_id": "web-1",
        "surface_kind": "web",
        "surface_app": "web",
        "final_text": "typed prefix PASTED-AI-BLOCK-OF-FORTY-PLUS-CHARACTERS-XX",
        "events": [
            {"at_ms": 100, "inserted_chars": 6, "deleted_chars": 0, "keystrokes": 6},
            {"at_ms": 400, "inserted_chars": 7, "deleted_chars": 0, "keystrokes": 7},
            {"at_ms": 700, "inserted_chars": 41, "deleted_chars": 0, "keystrokes": 0}
        ]
    }"#;
    let request: Request = serde_json::from_str(json).unwrap();

    match process(request) {
        Response::Credential { manifest_b64 } => {
            let manifest = base64::engine::general_purpose::STANDARD
                .decode(manifest_b64)
                .unwrap();
            let readout = read_sidecar(
                &manifest,
                b"typed prefix PASTED-AI-BLOCK-OF-FORTY-PLUS-CHARACTERS-XX",
            )
            .unwrap();
            assert!(readout.valid);
            assert!(readout.record.evidence_flags.large_unkeyed_insertions >= 1);
            assert!(readout.claim.contains("WARNING"));
        }
        other => panic!("expected a credential, got {other:?}"),
    }
}

#[test]
fn malformed_request_type_is_an_error_not_a_panic() {
    let request: Request = serde_json::from_str(r#"{"type":"frobnicate"}"#).unwrap();
    assert!(matches!(process(request), Response::Error { .. }));
}
