use humanshipd_core::credential::{issue_sidecar, read_sidecar};
use humanshipd_core::record::Provenance;
use humanshipd_core::report::{render_report, NuanceSummary};
use humanshipd_core::session::{build_record, EditEvent, SessionInput};

fn session(final_text: &str, events: Vec<EditEvent>) -> SessionInput {
    SessionInput {
        session_id: "s".into(),
        surface_kind: "native-ax".into(),
        surface_app: "TextEdit".into(),
        final_text: final_text.into(),
        events,
    }
}

fn typed(at_ms: u64, chars: u64) -> EditEvent {
    EditEvent { at_ms, inserted_chars: chars, deleted_chars: 0, keystrokes: chars, at_offset: None }
}

fn pasted(at_ms: u64, chars: u64) -> EditEvent {
    EditEvent { at_ms, inserted_chars: chars, deleted_chars: 0, keystrokes: 0, at_offset: None }
}

#[test]
fn consecutive_same_class_insertions_merge_into_one_span() {
    let text = "x".repeat(30);
    let record = build_record(&session(&text, vec![typed(100, 10), typed(200, 10), typed(300, 10)]));
    assert_eq!(record.process.spans.len(), 1);
    assert_eq!(record.process.spans[0].provenance, Provenance::Typed);
    assert_eq!(record.process.spans[0].chars, 30);
    assert_eq!(record.process.spans[0].keystrokes, 30);
}

#[test]
fn a_paste_between_typing_yields_three_ordered_spans() {
    let text = "x".repeat(50);
    let record = build_record(&session(
        &text,
        vec![typed(100, 20), pasted(200, 10), typed(300, 20)],
    ));
    let kinds: Vec<Provenance> = record.process.spans.iter().map(|s| s.provenance).collect();
    assert_eq!(kinds, vec![Provenance::Typed, Provenance::Pasted, Provenance::Typed]);
}

#[test]
fn pure_deletions_do_not_break_a_typed_run() {
    let text = "x".repeat(20);
    let delete = EditEvent { at_ms: 200, inserted_chars: 0, deleted_chars: 5, keystrokes: 5, at_offset: None };
    let record = build_record(&session(&text, vec![typed(100, 10), delete, typed(300, 10)]));
    assert_eq!(record.process.spans.len(), 1, "a delete is a revision, not a span boundary");
    assert_eq!(record.process.spans[0].chars, 20);
}

#[test]
fn an_all_typed_document_reports_fully_typed_at_full_proportion() {
    let text = "x".repeat(40);
    let report = render_report(&build_record(&session(&text, vec![typed(100, 40)])));
    assert_eq!(report.summary, NuanceSummary::FullyTyped);
    assert_eq!(report.typed_chars, 40);
    assert_eq!(report.unknown_chars, 0);
    assert_eq!(report.bands.len(), 1);
    assert!((report.bands[0].fraction - 1.0).abs() < 1e-9);
}

#[test]
fn some_pasted_text_reports_typed_with_pastes() {
    let text = "x".repeat(100);
    let report = render_report(&build_record(&session(
        &text,
        vec![typed(100, 80), pasted(200, 20)],
    )));
    assert_eq!(report.summary, NuanceSummary::TypedWithPastes);
    assert_eq!(report.pasted_chars, 20);
    // bands are sorted largest-first.
    assert_eq!(report.bands[0].provenance, Provenance::Typed);
}

#[test]
fn predominantly_pasted_text_reports_mostly_pasted() {
    let text = "x".repeat(100);
    let report = render_report(&build_record(&session(
        &text,
        vec![typed(100, 20), pasted(200, 80)],
    )));
    assert_eq!(report.summary, NuanceSummary::MostlyPasted);
}

#[test]
fn final_text_not_observed_entering_is_counted_unknown_and_marks_unverified() {
    // 100-char document, but only 10 chars were observed being typed.
    let text = "x".repeat(100);
    let report = render_report(&build_record(&session(&text, vec![typed(100, 10)])));
    assert_eq!(report.unknown_chars, 90);
    assert_eq!(report.summary, NuanceSummary::Unverified);
    assert_eq!(report.bands[0].provenance, Provenance::Unknown);
}

#[test]
fn the_timeline_tracks_cumulative_length_and_marks_a_paste_as_a_jump() {
    let text = "x".repeat(50);
    let record = build_record(&session(
        &text,
        vec![typed(100, 20), pasted(200, 30)],
    ));
    let timeline = &record.process.timeline;
    assert_eq!(timeline.len(), 2);
    assert_eq!(timeline[0].length, 20);
    assert_eq!(timeline[1].length, 50);
    // the paste point arrived without keystrokes — the fingerprint's vertical jump.
    assert_eq!(timeline[1].keystrokes, 0);
    assert_eq!(timeline[1].inserted, 30);
}

#[test]
fn the_timeline_carries_per_edit_offset_when_the_adapter_supplies_it() {
    let text = "x".repeat(30);
    let mut e1 = typed(100, 20);
    e1.at_offset = Some(0);
    let mut e2 = typed(200, 10);
    e2.at_offset = Some(5); // a revisit: edit back at offset 5
    let record = build_record(&session(&text, vec![e1, e2]));
    assert_eq!(record.process.timeline[0].offset, Some(0));
    assert_eq!(record.process.timeline[1].offset, Some(5));
}

#[test]
fn a_deletion_dips_the_timeline_length() {
    let text = "x".repeat(15);
    let delete = EditEvent { at_ms: 200, inserted_chars: 0, deleted_chars: 5, keystrokes: 5, at_offset: None };
    let record = build_record(&session(&text, vec![typed(100, 20), delete]));
    let timeline = &record.process.timeline;
    assert_eq!(timeline[0].length, 20);
    assert_eq!(timeline[1].length, 15, "a deletion lowers cumulative length");
}

#[test]
fn the_report_renders_from_a_verified_credential_record() {
    // The report is derived from the same record carried in the signed credential,
    // so a verifier reaches it after validation — no separate trust surface.
    let text = "The quick brown fox jumps over the lazy dog and keeps on running.";
    let chars = text.chars().count() as u64;
    let record = build_record(&session(text, vec![typed(100, chars)]));
    let manifest = issue_sidecar(&record, text.as_bytes()).expect("issue");

    let readout = read_sidecar(&manifest, text.as_bytes()).expect("read");
    assert!(readout.valid);
    let report = render_report(&readout.record);
    assert_eq!(report.summary, NuanceSummary::FullyTyped);
    assert_eq!(report.total_chars, chars);
}
