use humanshipd_core::canonical::{canonical_sha256, canonicalize};
use humanshipd_core::record::*;

fn sample_record() -> WritingSessionRecord {
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

#[test]
fn canonical_bytes_are_deterministic() {
    let record = sample_record();
    let first = canonicalize(&record).expect("canonicalize");
    let second = canonicalize(&record).expect("canonicalize");
    assert_eq!(first, second, "canonical bytes must be byte-identical");
}

#[test]
fn canonical_json_has_sorted_keys() {
    // JCS (RFC 8785) sorts object keys lexicographically. "document_binding"
    // must therefore appear before "evidence_flags", before "process", etc.
    let record = sample_record();
    let bytes = canonicalize(&record).expect("canonicalize");
    let json = String::from_utf8(bytes).expect("utf8");
    let doc = json.find("document_binding").expect("has document_binding");
    let evidence = json.find("evidence_flags").expect("has evidence_flags");
    let process = json.find("\"process\"").expect("has process");
    assert!(doc < evidence, "keys must be sorted: document_binding < evidence_flags");
    assert!(evidence < process, "keys must be sorted: evidence_flags < process");
}

#[test]
fn canonical_hash_is_stable_and_hex_sha256() {
    let record = sample_record();
    let h1 = canonical_sha256(&record).expect("hash");
    let h2 = canonical_sha256(&record).expect("hash");
    assert_eq!(h1, h2, "hash of an unchanged record must be stable");
    assert_eq!(h1.len(), 64, "sha256 hex is 64 chars");
    assert!(h1.chars().all(|c| c.is_ascii_hexdigit()));
}

#[test]
fn changing_a_field_changes_the_hash() {
    let record = sample_record();
    let mut altered = record.clone();
    altered.document_binding.char_count += 1;
    assert_ne!(
        canonical_sha256(&record).unwrap(),
        canonical_sha256(&altered).unwrap(),
        "any field change must change the canonical hash"
    );
}
