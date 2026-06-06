// Google Docs revision-log → credential, end to end. The sample below mirrors the
// shape of a real `/revisions/load` response captured live on 2026-06-06 (the
// `)]}'` guard, the `changelog` array of `[op, ts, user, rev, sid, …]` entries,
// and the `is`/`ds`/`mlti` ops). See docs/research/2026-06-06-google-docs-writing-capture.md.

use humanshipd_core::canonical::sha256_hex;
use humanshipd_core::credential::{issue_sidecar, read_sidecar};
use humanshipd_core::session::build_record;
use humanshipd_core::session_from_changelog;

const SAMPLE: &str = r#")]}'
{"chunkedSnapshot":[[{"ty":"as","st":"text","si":0,"ei":1,"sm":{}}]],"changelog":[
[{"ty":"mlti","mts":[{"ty":"as","st":"document","si":0,"ei":0,"sm":{}},{"ty":"as","st":"text","si":0,"ei":1,"sm":{}}]},1780782958417,"u1",1,null,null,null,null,false],
[{"ty":"is","ibi":1,"s":"The quick brown fox "},1780782996913,"u1",2,"s1",0,null,null,false],
[{"ty":"is","ibi":21,"s":"jumps."},1780783000000,"u1",3,"s1",0,null,null,false],
[{"ty":"mlti","mts":[{"ty":"is","ibi":27,"s":" Extra."}]},1780783004000,"u1",4,"s1",0,null,null,false],
[{"ty":"ds","si":1,"ei":4},1780783006000,"u1",5,"s1",0,null,null,false]
]}"#;

#[test]
fn parses_a_changelog_into_a_reconstructed_session() {
    let input = session_from_changelog(SAMPLE, "gdoc-test", "docs.google.com").expect("parse");

    assert_eq!(input.surface_kind, "gdocs");
    // "The quick brown fox jumps. Extra." with the leading "The " deleted.
    assert_eq!(input.final_text, "quick brown fox jumps. Extra.");
    // 3 inserts (two direct + one inside the mlti bundle) + 1 delete; style ops skipped.
    assert_eq!(input.events.len(), 4);
    assert_eq!(input.events[0].inserted_chars, 20);
    assert_eq!(input.events[0].at_offset, Some(0));
    assert_eq!(input.events.last().unwrap().deleted_chars, 4);
    // timestamps are made relative to the first changelog entry.
    assert_eq!(input.events[0].at_ms, 1780782996913 - 1780782958417);
}

#[test]
fn a_google_doc_changelog_becomes_a_verifiable_credential() {
    // The smoking gun, at the data level: real-shaped Docs edit history → signed
    // credential → verifies, bound to the reconstructed document text.
    let input = session_from_changelog(SAMPLE, "gdoc-test", "docs.google.com").expect("parse");
    let text = input.final_text.clone();
    let record = build_record(&input);

    let manifest = issue_sidecar(&record, text.as_bytes()).expect("issue");
    let readout = read_sidecar(&manifest, text.as_bytes()).expect("read");

    assert!(readout.valid);
    assert_eq!(
        readout.record.document_binding.final_text_sha256,
        sha256_hex(text.as_bytes())
    );
    assert_eq!(readout.record.document_binding.char_count, text.chars().count() as u64);
    // the edit positions survived into the timeline (true offsets from Docs).
    assert!(readout.record.process.timeline.iter().all(|p| p.offset.is_some()));
}

#[test]
fn a_body_without_the_xssi_prefix_still_parses() {
    let no_prefix = SAMPLE.trim_start_matches(")]}'\n");
    let input = session_from_changelog(no_prefix, "g", "docs.google.com").expect("parse");
    assert_eq!(input.final_text, "quick brown fox jumps. Extra.");
}
