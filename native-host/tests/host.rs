use humanshipd_core::canonical::sha256_hex;
use humanshipd_core::{verify_badge, KeyPair, LocalTsa};
use humanshipd_host::handler::{process, Ctx};
use humanshipd_host::messages::{Request, Response};
use humanshipd_host::protocol::{read_message, write_message};
use std::io::Cursor;

fn ctx_fixture() -> (KeyPair, LocalTsa) {
    (
        KeyPair::from_seed(&[7u8; 32]),
        LocalTsa::new(&[9u8; 32], "humanshipd:local-poc", "2026-06-05T12:00:00Z"),
    )
}

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
    let (key, tsa) = ctx_fixture();
    let ctx = Ctx {
        client_key: &key,
        tsa,
    };
    let request: Request = serde_json::from_str(r#"{"type":"ping"}"#).unwrap();
    assert!(matches!(process(request, &ctx), Response::Pong { .. }));
}

#[test]
fn issue_request_produces_a_verifying_badge() {
    let (key, tsa) = ctx_fixture();
    let ctx = Ctx {
        client_key: &key,
        tsa,
    };
    let json = r#"{
        "type": "issue",
        "session_id": "s1",
        "surface_kind": "gdocs",
        "surface_app": "docs.google.com",
        "final_text": "hello world",
        "events": [{"at_ms": 0, "inserted_chars": 11, "deleted_chars": 0, "keystrokes": 11}]
    }"#;
    let request: Request = serde_json::from_str(json).unwrap();

    match process(request, &ctx) {
        Response::Badge { badge } => {
            let result = verify_badge(&badge).unwrap();
            assert!(result.signature_valid);
            assert!(result.timestamp_valid);
            assert_eq!(result.document_sha256, sha256_hex(b"hello world"));
        }
        other => panic!("expected a badge response, got {other:?}"),
    }
}

#[test]
fn malformed_request_type_is_an_error_not_a_panic() {
    let (key, tsa) = ctx_fixture();
    let ctx = Ctx {
        client_key: &key,
        tsa,
    };
    let request: Request = serde_json::from_str(r#"{"type":"frobnicate"}"#).unwrap();
    assert!(matches!(process(request, &ctx), Response::Error { .. }));
}
