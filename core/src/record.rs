use serde::{Deserialize, Serialize};

/// The current record schema identifier (spec §5).
pub const SCHEMA: &str = "authorshipped/record@0.4";

/// A metadata-only record of how a piece of text was written.
///
/// Contains **no document content** — only counts, timing, and hashes (spec §5).
/// This is the object that gets hashed, signed, and time-anchored.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WritingSessionRecord {
    pub schema: String,
    /// Random, unlinkable session identifier.
    pub session_id: String,
    pub surface: Surface,
    pub document_binding: DocumentBinding,
    pub process: ProcessStats,
    pub evidence_flags: EvidenceFlags,
    pub replay: Replay,
}

/// Where the writing happened (capture adapter + app), no content.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Surface {
    /// One of: "gdocs" | "native-ax" | "web" | "ocr".
    pub kind: String,
    pub app: String,
}

/// Binds the record to a specific final text by hash only.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DocumentBinding {
    pub final_text_sha256: String,
    pub char_count: u64,
}

/// Metadata-only process statistics (no content).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProcessStats {
    pub active_ms: u64,
    pub keystrokes: u64,
    pub bursts: BurstStats,
    pub pauses: PauseStats,
    pub revisions: RevisionStats,
    /// Insertions that appeared with no correlated keystrokes (AI-dump signal).
    pub insertions_without_keystrokes: Vec<UnkeyedInsertion>,
    /// Fraction of inserted characters accompanied by keystrokes (typed vs appeared).
    pub keyed_fraction: f64,
    /// Ordered provenance of how text entered the document (signals spec §4).
    /// Order-based, not offset-based: spans reflect insertion order, not final-text
    /// character ranges (exact offsets await positional capture).
    pub spans: Vec<ProvenanceSpan>,
    /// Content-free timeline for the fingerprint graph (signals spec §7): document
    /// length over time, with per-point edit deltas. Downsampled. Carries only
    /// counts and timestamps — never text or positions within the text.
    pub timeline: Vec<TimelinePoint>,
}

/// One sampled moment in the writing timeline (signals spec §7). Content-free.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimelinePoint {
    /// Milliseconds since session start.
    pub at_ms: u64,
    /// Cumulative document length (chars) after this point's edit.
    pub length: u64,
    /// Character offset where the edit occurred, when known (caret/diff locus).
    /// `None` ⇒ position unknown; the fingerprint falls back to `length`.
    pub offset: Option<u64>,
    pub inserted: u64,
    pub deleted: u64,
    /// Physical keystrokes for this point (0 ⇒ text appeared without typing).
    pub keystrokes: u64,
}

/// How a contiguous run of inserted text entered the document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Provenance {
    /// Arrived via keystrokes in a tracked editor.
    Typed,
    /// Arrived via paste (origin captured separately, never the content).
    Pasted,
    /// Accepted from a recognized AI integration / pasted from a known AI tool.
    AiTool,
    /// Entered outside any tracked surface, or pre-dates capture.
    Unknown,
}

/// One contiguous run of inserted text with a single provenance (signals spec §4).
/// Carries counts only — never the text itself.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProvenanceSpan {
    pub provenance: Provenance,
    pub chars: u64,
    pub keystrokes: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BurstStats {
    pub count: u64,
    pub mean_len: f64,
    pub buckets: Vec<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PauseStats {
    pub gt_2s: u64,
    pub buckets: Vec<u64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RevisionStats {
    pub insertions: u64,
    pub deletions: u64,
    pub reformulations: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UnkeyedInsertion {
    pub size: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceFlags {
    pub large_unkeyed_insertions: u64,
}

/// Optional, local-only replay metadata (spec §7.1). The hash binds a shared
/// replay to this session; the replay content itself never lives in the record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Replay {
    pub available: bool,
    pub log_sha256: Option<String>,
}
