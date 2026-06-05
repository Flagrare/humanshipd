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
fn malformed_request_type_is_an_error_not_a_panic() {
    let request: Request = serde_json::from_str(r#"{"type":"frobnicate"}"#).unwrap();
    assert!(matches!(process(request), Response::Error { .. }));
}
