//! Per-document, append-only capture log (Slice 1 of cross-session continuity).
//!
//! A `CaptureLog` accumulates a document's writing across sessions as normalized,
//! surface-agnostic ops. Core owns replay + accumulation so there is one source of
//! truth (native + WASM); adapters only capture ops and persist the serialized log.
//! The log holds the inserted *text* (needed to reconstruct the document) and is
//! local-only working state — it never enters the content-free record/credential.

use serde::{Deserialize, Serialize};

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
