//! Per-document, append-only capture log (Slice 1 of cross-session continuity).
//!
//! A `CaptureLog` accumulates a document's writing across sessions as normalized,
//! surface-agnostic ops. Core owns replay + accumulation so there is one source of
//! truth (native + WASM); adapters only capture ops and persist the serialized log.
//! The log holds the inserted *text* (needed to reconstruct the document) and is
//! local-only working state — it never enters the content-free record/credential.

use serde::{Deserialize, Serialize};

use crate::canonical::sha256_hex;
use crate::record::*;
use crate::session::{build_spans, build_timeline, ops_to_events, pauses_and_bursts};

/// Versioned log schema identifier.
pub const LOG_SCHEMA: &str = "authorshipped/log@1";

/// How "this document" is keyed across sessions. Slice 1: a Google Docs URL id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentIdentity {
    /// Surface family, e.g. `"gdocs"` (Slice 2 adds `"native"`).
    pub kind: String,
    /// Stable per-surface id (the Docs URL document id).
    pub id: String,
}

/// One normalized edit. `at_ms` is relative to its session's start.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum CapturedOp {
    /// Insert `text` at character offset `pos`. `pasted` ⇒ arrived without typing.
    Insert { at_ms: u64, pos: u64, text: String, pasted: bool },
    /// Delete `len` characters starting at offset `pos`.
    Delete { at_ms: u64, pos: u64, len: u64 },
}

impl CapturedOp {
    /// Milliseconds since this op's session started.
    pub fn at_ms(&self) -> u64 {
        match self {
            CapturedOp::Insert { at_ms, .. } | CapturedOp::Delete { at_ms, .. } => *at_ms,
        }
    }
}

/// One contiguous writing session (one page-load / app run).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaptureSession {
    pub session_id: String,
    pub surface_kind: String,
    pub surface_app: String,
    /// Absolute epoch milliseconds at session start (for cross-session ordering).
    pub started_at_ms: u64,
    pub ops: Vec<CapturedOp>,
}

/// The accumulated, persisted capture for one document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaptureLog {
    pub schema: String,
    pub identity: DocumentIdentity,
    pub sessions: Vec<CaptureSession>,
}

impl CaptureLog {
    /// A fresh log for `identity`, with no sessions yet.
    pub fn new(identity: DocumentIdentity) -> Self {
        CaptureLog { schema: LOG_SCHEMA.to_string(), identity, sessions: Vec::new() }
    }

    /// Append a captured session.
    pub fn append(&mut self, session: CaptureSession) {
        self.sessions.push(session);
    }
}

/// Why a log could not be turned into a credential.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogError {
    /// An op references a position the log never witnessed being written — the
    /// document had pre-existing content, or was edited outside our capture.
    Unwitnessed { pos: u64, buffer_len: u64 },
}

impl std::fmt::Display for LogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogError::Unwitnessed { pos, buffer_len } => write!(
                f,
                "edit at position {pos} is beyond the {buffer_len} characters humanshipd witnessed — this document wasn't captured from the start"
            ),
        }
    }
}
impl std::error::Error for LogError {}

impl CaptureLog {
    /// Replay every session's ops into the reconstructed document text. Errors with
    /// [`LogError::Unwitnessed`] when an op falls beyond the witnessed buffer.
    pub fn reconstruct_text(&self) -> Result<String, LogError> {
        Ok(self.reconstruct_buffer()?.into_iter().collect())
    }

    /// Build a content-free record over **all** sessions. Aggregates counts; active
    /// time is the sum of per-session writing spans; cross-session gaps are session
    /// boundaries, not pauses. Declines with [`LogError`] on unwitnessed content.
    pub fn build_record(&self) -> Result<WritingSessionRecord, LogError> {
        let text: String = self.reconstruct_buffer()?.into_iter().collect();

        let per_session: Vec<Vec<crate::session::EditEvent>> =
            self.sessions.iter().map(|s| ops_to_events(&s.ops)).collect();
        let all: Vec<crate::session::EditEvent> =
            per_session.iter().flatten().cloned().collect();

        let keystrokes: u64 = all.iter().map(|e| e.keystrokes).sum();
        let total_inserted: u64 = all.iter().map(|e| e.inserted_chars).sum();
        let keyed_inserted: u64 =
            all.iter().filter(|e| e.keystrokes > 0).map(|e| e.inserted_chars).sum();
        let keyed_fraction = if total_inserted > 0 {
            keyed_inserted as f64 / total_inserted as f64
        } else {
            1.0
        };
        let insertions_without_keystrokes: Vec<UnkeyedInsertion> = all
            .iter()
            .filter(|e| e.inserted_chars > 0 && e.keystrokes == 0)
            .map(|e| UnkeyedInsertion { size: e.inserted_chars })
            .collect();
        let large_unkeyed_insertions = insertions_without_keystrokes
            .iter()
            .filter(|u| u.size >= crate::session::LARGE_UNKEYED_THRESHOLD)
            .count() as u64;
        let revisions = RevisionStats {
            insertions: all.iter().filter(|e| e.inserted_chars > 0).count() as u64,
            deletions: all.iter().filter(|e| e.deleted_chars > 0).count() as u64,
            reformulations: 0,
        };

        let mut active_ms = 0u64;
        let mut gt_2s = 0u64;
        let mut burst_count = 0u64;
        let mut burst_total = 0f64;
        let mut spans: Vec<ProvenanceSpan> = Vec::new();
        let mut timeline: Vec<TimelinePoint> = Vec::new();
        let mut clock = 0u64;
        for events in &per_session {
            if let (Some(first), Some(last)) = (events.first(), events.last()) {
                active_ms += last.at_ms.saturating_sub(first.at_ms);
            }
            let (p, b) = pauses_and_bursts(events);
            gt_2s += p.gt_2s;
            burst_count += b.count;
            burst_total += b.mean_len * b.count as f64;
            spans.extend(build_spans(events));
            for mut pt in build_timeline(events) {
                pt.at_ms += clock;
                timeline.push(pt);
            }
            if let Some(last) = events.last() {
                clock += last.at_ms + 1;
            }
        }
        let pauses = PauseStats { gt_2s, buckets: Vec::new() };
        let bursts = BurstStats {
            count: burst_count,
            mean_len: if burst_count > 0 { burst_total / burst_count as f64 } else { 0.0 },
            buckets: Vec::new(),
        };

        let surface = self
            .sessions
            .last()
            .map(|s| Surface { kind: s.surface_kind.clone(), app: s.surface_app.clone() })
            .unwrap_or(Surface { kind: self.identity.kind.clone(), app: String::new() });
        let session_id = self
            .sessions
            .last()
            .map(|s| s.session_id.clone())
            .unwrap_or_default();

        Ok(WritingSessionRecord {
            schema: SCHEMA.to_string(),
            session_id,
            surface,
            document_binding: DocumentBinding {
                final_text_sha256: sha256_hex(text.as_bytes()),
                char_count: text.chars().count() as u64,
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
            evidence_flags: EvidenceFlags { large_unkeyed_insertions },
            replay: Replay { available: false, log_sha256: None },
            session_count: self.sessions.len() as u64,
            first_capture_at_ms: self.sessions.first().map(|s| s.started_at_ms).unwrap_or(0),
            // Wall-clock time of the last witnessed edit: the last session's start
            // plus its final op's offset — not merely when that session opened.
            last_capture_at_ms: self
                .sessions
                .last()
                .map(|s| s.started_at_ms + s.ops.last().map(CapturedOp::at_ms).unwrap_or(0))
                .unwrap_or(0),
        })
    }

    fn reconstruct_buffer(&self) -> Result<Vec<char>, LogError> {
        let mut buf: Vec<char> = Vec::new();
        for session in &self.sessions {
            for op in &session.ops {
                match op {
                    CapturedOp::Insert { pos, text, .. } => {
                        let pos_u64 = *pos;
                        let pos = *pos as usize;
                        if pos > buf.len() {
                            return Err(LogError::Unwitnessed {
                                pos: pos_u64,
                                buffer_len: buf.len() as u64,
                            });
                        }
                        buf.splice(pos..pos, text.chars());
                    }
                    CapturedOp::Delete { pos, len, .. } => {
                        let start = *pos as usize;
                        let end = start + *len as usize;
                        if end > buf.len() {
                            return Err(LogError::Unwitnessed {
                                pos: *pos,
                                buffer_len: buf.len() as u64,
                            });
                        }
                        buf.drain(start..end);
                    }
                }
            }
        }
        Ok(buf)
    }
}
