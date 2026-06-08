// Each integration-test binary compiles this module independently and uses only a
// subset of the helpers, so unused-here items are expected (and clippy runs with
// `-D warnings` in CI).
#![allow(dead_code)]

use humanshipd_core::record::*;
use std::io::Write;

/// Synthesize a minimal `.docx`: a zip whose only entry is `word/document.xml`
/// wrapping `text` in one paragraph. Enough for the OOXML text extractor, which
/// only reads `word/document.xml`.
pub fn minimal_docx(text: &str) -> Vec<u8> {
    let escaped = text
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;");
    let xml = format!(
        "<?xml version=\"1.0\"?><w:document xmlns:w=\"x\"><w:body><w:p><w:r><w:t>{escaped}</w:t></w:r></w:p></w:body></w:document>"
    );
    let mut zip = zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
    zip.start_file("word/document.xml", zip::write::SimpleFileOptions::default())
        .expect("start docx entry");
    zip.write_all(xml.as_bytes()).expect("write docx xml");
    zip.finish().expect("finish docx").into_inner()
}

/// A representative, fully-populated record for tests.
pub fn sample_record() -> WritingSessionRecord {
    WritingSessionRecord {
        schema: SCHEMA.to_string(),
        session_id: "test-session-0001".to_string(),
        surface: Surface {
            kind: "gdocs".to_string(),
            app: "docs.google.com".to_string(),
        },
        document_binding: DocumentBinding {
            final_text_sha256: "a".repeat(64),
            char_count: 1234,
        },
        process: ProcessStats {
            active_ms: 60_000,
            keystrokes: 1200,
            bursts: BurstStats {
                count: 12,
                mean_len: 18.5,
                buckets: vec![3, 5, 4],
            },
            pauses: PauseStats {
                gt_2s: 7,
                buckets: vec![2, 3, 2],
            },
            revisions: RevisionStats {
                insertions: 40,
                deletions: 9,
                reformulations: 3,
            },
            insertions_without_keystrokes: vec![],
            keyed_fraction: 1.0,
            spans: vec![ProvenanceSpan {
                provenance: Provenance::Typed,
                chars: 1234,
                keystrokes: 1200,
            }],
            timeline: vec![
                TimelinePoint { at_ms: 300, length: 600, offset: Some(0), inserted: 600, deleted: 0, keystrokes: 600 },
                TimelinePoint { at_ms: 60_000, length: 1234, offset: Some(600), inserted: 634, deleted: 0, keystrokes: 600 },
            ],
        },
        evidence_flags: EvidenceFlags {
            large_unkeyed_insertions: 0,
        },
        replay: Replay {
            available: false,
            log_sha256: None,
        },
    }
}
