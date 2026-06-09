//! WASM bridge: verify a humanshipd credential in the browser using the exact
//! same `humanshipd-core::credential::read_sidecar` logic as the CLI. The browser
//! shows the same verdict and the same honest claim — no separate trust surface.

use humanshipd_core::credential::{
    issue_sidecar_with_author, read_sidecar_with_text, CredentialReadout, TrustStatus, Verdict,
};
use humanshipd_core::record::TimelinePoint;
use humanshipd_core::report::{render_process_shape, render_report, ProcessShape, ProvenanceReport};
use humanshipd_core::session::{build_record, EditEvent, SessionInput};
use humanshipd_core::formats::extract_named;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

// --- SPIKE: in-browser issuance (Decision: WASM-in-worker issuance) -----------
// Proves the issuance path (ephemeral key-gen + c2pa signing) runs under WASM, so
// the extension can issue without a native host. If this works, it graduates into
// the real worker-backed flow.

#[derive(Deserialize)]
struct SessionDto {
    session_id: String,
    surface_kind: String,
    surface_app: String,
    final_text: String,
    events: Vec<EventDto>,
    #[serde(default)]
    author: Option<String>,
}

#[derive(Deserialize)]
struct EventDto {
    at_ms: u64,
    inserted_chars: u64,
    deleted_chars: u64,
    keystrokes: u64,
    #[serde(default)]
    at_offset: Option<u64>,
}

/// Issue a credential entirely in the browser from a captured session (JSON).
/// Returns the `.c2pa` manifest bytes. The key-gen + signing run in WASM.
#[wasm_bindgen]
pub fn issue_credential(session_json: &str) -> Result<Vec<u8>, JsValue> {
    let dto: SessionDto = serde_json::from_str(session_json)
        .map_err(|e| JsValue::from_str(&format!("bad session json: {e}")))?;
    let input = SessionInput {
        session_id: dto.session_id,
        surface_kind: dto.surface_kind,
        surface_app: dto.surface_app,
        final_text: dto.final_text.clone(),
        events: dto
            .events
            .into_iter()
            .map(|e| EditEvent {
                at_ms: e.at_ms,
                inserted_chars: e.inserted_chars,
                deleted_chars: e.deleted_chars,
                keystrokes: e.keystrokes,
                at_offset: e.at_offset,
            })
            .collect(),
    };
    let record = build_record(&input);
    issue_sidecar_with_author(&record, dto.final_text.as_bytes(), dto.author.as_deref())
        .map_err(|e| JsValue::from_str(&format!("issue failed: {e}")))
}

/// Serializable view of the tiered [`Verdict`] for the page (Decision 4).
#[derive(Serialize)]
struct VerdictView {
    /// One of: exact_file | same_content | same_writing | borderline | no_match | invalid.
    tier: &'static str,
    /// Normalized Hamming content distance (0.0–1.0), when a content comparison was made.
    distance: Option<f64>,
}

impl From<&Verdict> for VerdictView {
    fn from(v: &Verdict) -> Self {
        match v {
            Verdict::Invalid => VerdictView { tier: "invalid", distance: None },
            Verdict::ExactFile => VerdictView { tier: "exact_file", distance: None },
            Verdict::SameContent { distance } => {
                VerdictView { tier: "same_content", distance: Some(*distance) }
            }
            Verdict::SameWriting { distance } => {
                VerdictView { tier: "same_writing", distance: Some(*distance) }
            }
            Verdict::Borderline { distance } => {
                VerdictView { tier: "borderline", distance: Some(*distance) }
            }
            Verdict::NoMatch { distance } => VerdictView { tier: "no_match", distance: *distance },
        }
    }
}

/// Serializable view of [`TrustStatus`] for the page (Decision 6).
#[derive(Serialize)]
struct TrustView {
    signed: bool,
    trusted: bool,
    identity_verified: bool,
    timestamp: Option<String>,
}

impl From<&TrustStatus> for TrustView {
    fn from(t: &TrustStatus) -> Self {
        TrustView {
            signed: t.signed,
            trusted: t.trusted,
            identity_verified: t.identity_verified,
            timestamp: t.timestamp.clone(),
        }
    }
}

#[derive(Serialize)]
struct VerifyResult {
    valid: bool,
    /// The tiered match verdict; absent only when the credential couldn't be read.
    verdict: Option<VerdictView>,
    /// What the signature establishes — and what it doesn't (Decision 6).
    trust: Option<TrustView>,
    claim: String,
    document_sha256: String,
    char_count: u64,
    ai_dump_flags: u64,
    /// Banded provenance report (signals spec §5); present only when the
    /// credential reads successfully.
    report: Option<ProvenanceReport>,
    /// Content-free writing timeline for the fingerprint graph (signals spec §7).
    timeline: Vec<TimelinePoint>,
    /// Weak, positive-only process-shape corroboration (signals spec §3 Tier 2 / §6).
    process_shape: Option<ProcessShape>,
    /// Self-asserted (unverified) author name, if the credential carries one.
    author: Option<String>,
    error: Option<String>,
}

/// Verify a credential (`.c2pa` bytes) against the document bytes it should bind to,
/// treating the document as plain text. Kept for callers without a filename.
#[wasm_bindgen]
pub fn verify_credential(manifest: &[u8], document: &[u8]) -> JsValue {
    verify_credential_named(manifest, document, "document.txt")
}

/// Verify against a document whose `filename` selects the text extractor (Decision
/// 4): `.txt`/`.docx` work in-browser; `.pdf` returns an error directing the user to
/// the CLI. So the same writing exported as a `.docx` reaches the content engine.
#[wasm_bindgen]
pub fn verify_credential_named(manifest: &[u8], document: &[u8], filename: &str) -> JsValue {
    let result = match extract_named(filename, document) {
        Ok(text) => match read_sidecar_with_text(manifest, document, &text) {
            Ok(readout) => success_result(readout),
            Err(e) => error_result(e.to_string()),
        },
        Err(e) => error_result(e.to_string()),
    };
    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}

fn success_result(readout: CredentialReadout) -> VerifyResult {
    VerifyResult {
        valid: readout.valid,
        verdict: Some(VerdictView::from(&readout.verdict)),
        trust: Some(TrustView::from(&readout.trust)),
        claim: readout.claim,
        document_sha256: readout.record.document_binding.final_text_sha256.clone(),
        char_count: readout.record.document_binding.char_count,
        ai_dump_flags: readout.record.evidence_flags.large_unkeyed_insertions,
        report: Some(render_report(&readout.record)),
        timeline: readout.record.process.timeline.clone(),
        process_shape: Some(render_process_shape(&readout.record)),
        author: readout.author.clone(),
        error: None,
    }
}

fn error_result(message: String) -> VerifyResult {
    VerifyResult {
        valid: false,
        verdict: None,
        trust: None,
        claim: String::new(),
        document_sha256: String::new(),
        char_count: 0,
        ai_dump_flags: 0,
        report: None,
        timeline: Vec::new(),
        process_shape: None,
        author: None,
        error: Some(message),
    }
}
