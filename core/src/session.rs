use crate::canonical::sha256_hex;
use crate::record::*;

/// Gaps longer than this (ms) count as planning pauses and break a burst.
pub const PAUSE_THRESHOLD_MS: u64 = 2000;
/// An un-keyed insertion at least this many chars is "large" (paste/AI-dump signal).
pub const LARGE_UNKEYED_THRESHOLD: u64 = 20;
/// Cap the timeline so the credential stays small; longer sessions are stride-sampled.
pub const MAX_TIMELINE_POINTS: usize = 300;

/// Raw capture input from an adapter. `final_text` is used only to compute the
/// document hash + char count; it is **never** copied into the record.
#[derive(Debug, Clone)]
pub struct SessionInput {
    pub session_id: String,
    pub surface_kind: String,
    pub surface_app: String,
    pub final_text: String,
    pub events: Vec<EditEvent>,
}

/// One observed edit, with timing and how many physical keystrokes drove it.
#[derive(Debug, Clone)]
pub struct EditEvent {
    /// Milliseconds since session start.
    pub at_ms: u64,
    pub inserted_chars: u64,
    pub deleted_chars: u64,
    /// Physical keystrokes attributed to this event (0 ⇒ text appeared without typing).
    pub keystrokes: u64,
    /// Character offset where the edit occurred, when the adapter can determine it
    /// (caret position / text-diff locus). `None` ⇒ position unknown — the
    /// fingerprint then falls back to cumulative length instead of true position.
    pub at_offset: Option<u64>,
}

/// Build a metadata-only record from captured events. No document content is retained.
pub fn build_record(input: &SessionInput) -> WritingSessionRecord {
    let events = &input.events;

    let keystrokes: u64 = events.iter().map(|e| e.keystrokes).sum();
    let total_inserted: u64 = events.iter().map(|e| e.inserted_chars).sum();
    let keyed_inserted: u64 = events
        .iter()
        .filter(|e| e.keystrokes > 0)
        .map(|e| e.inserted_chars)
        .sum();
    let keyed_fraction = if total_inserted > 0 {
        keyed_inserted as f64 / total_inserted as f64
    } else {
        1.0
    };

    let insertions_without_keystrokes: Vec<UnkeyedInsertion> = events
        .iter()
        .filter(|e| e.inserted_chars > 0 && e.keystrokes == 0)
        .map(|e| UnkeyedInsertion {
            size: e.inserted_chars,
        })
        .collect();
    let large_unkeyed_insertions = insertions_without_keystrokes
        .iter()
        .filter(|u| u.size >= LARGE_UNKEYED_THRESHOLD)
        .count() as u64;

    let active_ms = match (events.first(), events.last()) {
        (Some(first), Some(last)) => last.at_ms.saturating_sub(first.at_ms),
        _ => 0,
    };

    let revisions = RevisionStats {
        insertions: events.iter().filter(|e| e.inserted_chars > 0).count() as u64,
        deletions: events.iter().filter(|e| e.deleted_chars > 0).count() as u64,
        reformulations: 0,
    };

    let (pauses, bursts) = pauses_and_bursts(events);
    let spans = build_spans(events);
    let timeline = build_timeline(events);

    WritingSessionRecord {
        schema: SCHEMA.to_string(),
        session_id: input.session_id.clone(),
        surface: Surface {
            kind: input.surface_kind.clone(),
            app: input.surface_app.clone(),
        },
        document_binding: DocumentBinding {
            final_text_sha256: sha256_hex(input.final_text.as_bytes()),
            char_count: input.final_text.chars().count() as u64,
        },
        process: ProcessStats {
            active_ms,
            keystrokes,
            bursts,
            pauses,
            revisions,
            insertions_without_keystrokes,
            keyed_fraction,
            spans,
            timeline,
        },
        evidence_flags: EvidenceFlags {
            large_unkeyed_insertions,
        },
        replay: Replay {
            available: false,
            log_sha256: None,
        },
    }
}

/// Derive ordered provenance spans by merging consecutive insertions of the same
/// class (typed vs. pasted). Pure-deletion events are skipped — they are revisions,
/// captured in `RevisionStats`, and do not break a same-class run. Origin detail
/// (AI tool, paste source) is not yet available from the event stream, so the
/// builder emits only `Typed` and `Pasted`.
fn build_spans(events: &[EditEvent]) -> Vec<ProvenanceSpan> {
    let mut spans: Vec<ProvenanceSpan> = Vec::new();
    for event in events.iter().filter(|e| e.inserted_chars > 0) {
        let provenance = if event.keystrokes > 0 {
            Provenance::Typed
        } else {
            Provenance::Pasted
        };
        match spans.last_mut() {
            Some(span) if span.provenance == provenance => {
                span.chars += event.inserted_chars;
                span.keystrokes += event.keystrokes;
            }
            _ => spans.push(ProvenanceSpan {
                provenance,
                chars: event.inserted_chars,
                keystrokes: event.keystrokes,
            }),
        }
    }
    spans
}

/// Build the content-free writing timeline: cumulative document length after each
/// edit, paired with that edit's deltas (signals spec §7). The fingerprint graph
/// plots `length` (y) against `at_ms` (x) — a paste appears as a vertical jump, a
/// deletion as a dip. Stride-sampled to `MAX_TIMELINE_POINTS`, always keeping the
/// last point so the final length is faithful.
fn build_timeline(events: &[EditEvent]) -> Vec<TimelinePoint> {
    let mut length: u64 = 0;
    let full: Vec<TimelinePoint> = events
        .iter()
        .map(|e| {
            length = (length + e.inserted_chars).saturating_sub(e.deleted_chars);
            TimelinePoint {
                at_ms: e.at_ms,
                length,
                offset: e.at_offset,
                inserted: e.inserted_chars,
                deleted: e.deleted_chars,
                keystrokes: e.keystrokes,
            }
        })
        .collect();

    if full.len() <= MAX_TIMELINE_POINTS {
        return full;
    }
    let stride = full.len().div_ceil(MAX_TIMELINE_POINTS);
    let mut sampled: Vec<TimelinePoint> = full.iter().step_by(stride).cloned().collect();
    if let Some(last) = full.last() {
        if sampled.last() != Some(last) {
            sampled.push(last.clone());
        }
    }
    sampled
}

/// Derive pause and burst statistics from inter-event timing.
fn pauses_and_bursts(events: &[EditEvent]) -> (PauseStats, BurstStats) {
    let mut gt_2s = 0u64;
    let mut burst_lengths: Vec<u64> = Vec::new();
    let mut current_burst = 0u64;
    let mut prev_at: Option<u64> = None;

    for event in events {
        if let Some(prev) = prev_at {
            if event.at_ms.saturating_sub(prev) > PAUSE_THRESHOLD_MS {
                gt_2s += 1;
                burst_lengths.push(current_burst);
                current_burst = 0;
            }
        }
        current_burst += event.inserted_chars;
        prev_at = Some(event.at_ms);
    }
    if prev_at.is_some() {
        burst_lengths.push(current_burst);
    }

    let count = burst_lengths.len() as u64;
    let mean_len = if count > 0 {
        burst_lengths.iter().sum::<u64>() as f64 / count as f64
    } else {
        0.0
    };

    (
        PauseStats {
            gt_2s,
            buckets: Vec::new(),
        },
        BurstStats {
            count,
            mean_len,
            buckets: Vec::new(),
        },
    )
}
