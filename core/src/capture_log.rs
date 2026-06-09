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
