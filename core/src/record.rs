use serde::{Deserialize, Serialize};

/// The current record schema identifier (spec §5).
pub const SCHEMA: &str = "authorshipped/record@0.1";

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
