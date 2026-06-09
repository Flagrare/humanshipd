use humanshipd_core::capture_log::{CaptureLog, CaptureSession, CapturedOp, DocumentIdentity, LOG_SCHEMA, LogError};

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

#[test]
fn reconstructs_text_across_two_sessions() {
    let mut log = CaptureLog::new(DocumentIdentity { kind: "gdocs".into(), id: "d".into() });
    log.append(session("s1", 1000, vec![
        CapturedOp::Insert { at_ms: 0, pos: 0, text: "Hello".into(), pasted: false },
    ]));
    log.append(session("s2", 90_000, vec![
        CapturedOp::Insert { at_ms: 0, pos: 5, text: " world".into(), pasted: false },
    ]));
    assert_eq!(log.reconstruct_text().unwrap(), "Hello world");
}

#[test]
fn declines_when_an_op_lands_beyond_the_witnessed_buffer() {
    let mut log = CaptureLog::new(DocumentIdentity { kind: "gdocs".into(), id: "d".into() });
    log.append(session("s1", 1000, vec![
        CapturedOp::Insert { at_ms: 0, pos: 5, text: "x".into(), pasted: false },
    ]));
    assert!(matches!(log.reconstruct_text(), Err(LogError::Unwitnessed { .. })));
}

#[test]
fn build_record_reports_two_sessions_and_aggregates_active_time() {
    let mut log = CaptureLog::new(DocumentIdentity { kind: "gdocs".into(), id: "d".into() });
    log.append(session("s1", 1_000, vec![
        CapturedOp::Insert { at_ms: 0,    pos: 0, text: "Hello".into(), pasted: false },
        CapturedOp::Insert { at_ms: 2_000, pos: 5, text: "!".into(),     pasted: false },
    ]));
    log.append(session("s2", 90_000_000, vec![
        CapturedOp::Insert { at_ms: 0,    pos: 6, text: " more".into(), pasted: false },
        CapturedOp::Insert { at_ms: 1_000, pos: 11, text: ".".into(),    pasted: false },
    ]));
    let record = log.build_record().unwrap();
    assert_eq!(record.session_count, 2);
    assert_eq!(record.process.active_ms, 3_000);
    assert_eq!(record.document_binding.char_count, "Hello! more.".chars().count() as u64);
}

#[test]
fn cross_session_gap_is_not_a_pause_and_timeline_is_monotonic() {
    let mut log = CaptureLog::new(DocumentIdentity { kind: "gdocs".into(), id: "d".into() });
    log.append(session("s1", 1_000, vec![
        CapturedOp::Insert { at_ms: 0, pos: 0, text: "ab".into(), pasted: false },
        CapturedOp::Insert { at_ms: 100, pos: 2, text: "cd".into(), pasted: false },
    ]));
    // A day later — the gap between sessions must NOT count as a pause.
    log.append(session("s2", 90_000_000, vec![
        CapturedOp::Insert { at_ms: 0, pos: 4, text: "ef".into(), pasted: false },
        CapturedOp::Insert { at_ms: 100, pos: 6, text: "gh".into(), pasted: false },
    ]));
    let r = log.build_record().unwrap();
    assert_eq!(r.process.pauses.gt_2s, 0, "cross-session gap must not be a pause");
    let mut prev = 0u64;
    for pt in &r.process.timeline {
        assert!(pt.at_ms >= prev, "timeline must be monotonic across sessions");
        prev = pt.at_ms;
    }
    assert_eq!(r.first_capture_at_ms, 1_000);
    assert_eq!(r.last_capture_at_ms, 90_000_000);
}

#[test]
fn a_paste_in_a_later_session_flags_an_unkeyed_insertion() {
    let mut log = CaptureLog::new(DocumentIdentity { kind: "gdocs".into(), id: "d".into() });
    log.append(session("s1", 1_000, vec![
        CapturedOp::Insert { at_ms: 0, pos: 0, text: "typed ".into(), pasted: false },
    ]));
    log.append(session("s2", 2_000, vec![
        CapturedOp::Insert { at_ms: 0, pos: 6, text: "PASTED BLOCK OF TWENTYPLUS CHARS".into(), pasted: true },
    ]));
    let r = log.build_record().unwrap();
    assert_eq!(r.evidence_flags.large_unkeyed_insertions, 1);
    assert!(r.process.keyed_fraction < 1.0);
}

#[test]
fn build_record_declines_when_a_later_session_edits_unwitnessed_content() {
    let mut log = CaptureLog::new(DocumentIdentity { kind: "gdocs".into(), id: "d".into() });
    log.append(session("s1", 1_000, vec![
        CapturedOp::Insert { at_ms: 0, pos: 0, text: "hi".into(), pasted: false },
    ]));
    // Delete a 5-char range from a 2-char buffer → beyond what we witnessed.
    log.append(session("s2", 2_000, vec![
        CapturedOp::Delete { at_ms: 0, pos: 0, len: 5 },
    ]));
    assert!(matches!(log.build_record(), Err(LogError::Unwitnessed { .. })));
}
