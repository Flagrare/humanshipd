use humanshipd_core::capture_log::{CaptureLog, CaptureSession, CapturedOp, DocumentIdentity, LOG_SCHEMA};

fn session(id: &str, started: u64, ops: Vec<CapturedOp>) -> CaptureSession {
    CaptureSession {
        session_id: id.into(),
        surface_kind: "gdocs".into(),
        surface_app: "docs.google.com".into(),
        started_at_ms: started,
        ops,
    }
}

#[test]
fn capture_log_round_trips_through_json() {
    let mut log = CaptureLog::new(DocumentIdentity { kind: "gdocs".into(), id: "doc-1".into() });
    log.append(session("s1", 1000, vec![
        CapturedOp::Insert { at_ms: 0, pos: 0, text: "Hello".into(), pasted: false },
        CapturedOp::Delete { at_ms: 50, pos: 4, len: 1 },
    ]));
    assert_eq!(log.schema, LOG_SCHEMA);
    let json = serde_json::to_string(&log).unwrap();
    let back: CaptureLog = serde_json::from_str(&json).unwrap();
    assert_eq!(back, log);
    assert_eq!(back.sessions.len(), 1);
}
