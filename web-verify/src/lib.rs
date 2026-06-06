//! WASM bridge: verify a humanshipd credential in the browser using the exact
//! same `humanshipd-core::credential::read_sidecar` logic as the CLI. The browser
//! shows the same verdict and the same honest claim — no separate trust surface.

use humanshipd_core::credential::read_sidecar;
use humanshipd_core::record::TimelinePoint;
use humanshipd_core::report::{render_process_shape, render_report, ProcessShape, ProvenanceReport};
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[derive(Serialize)]
struct VerifyResult {
    valid: bool,
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
    error: Option<String>,
}

/// Verify a credential (`.c2pa` bytes) against the document bytes it should bind to.
#[wasm_bindgen]
pub fn verify_credential(manifest: &[u8], document: &[u8]) -> JsValue {
    let result = match read_sidecar(manifest, document) {
        Ok(readout) => VerifyResult {
            valid: readout.valid,
            claim: readout.claim,
            document_sha256: readout.record.document_binding.final_text_sha256.clone(),
            char_count: readout.record.document_binding.char_count,
            ai_dump_flags: readout.record.evidence_flags.large_unkeyed_insertions,
            report: Some(render_report(&readout.record)),
            timeline: readout.record.process.timeline.clone(),
            process_shape: Some(render_process_shape(&readout.record)),
            error: None,
        },
        Err(e) => VerifyResult {
            valid: false,
            claim: String::new(),
            document_sha256: String::new(),
            char_count: 0,
            ai_dump_flags: 0,
            report: None,
            timeline: Vec::new(),
            process_shape: None,
            error: Some(e.to_string()),
        },
    };
    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}
