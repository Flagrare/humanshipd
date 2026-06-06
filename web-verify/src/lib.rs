//! WASM bridge: verify a humanshipd credential in the browser using the exact
//! same `humanshipd-core::credential::read_sidecar` logic as the CLI. The browser
//! shows the same verdict and the same honest claim — no separate trust surface.

use humanshipd_core::credential::read_sidecar;
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[derive(Serialize)]
struct VerifyResult {
    valid: bool,
    claim: String,
    document_sha256: String,
    char_count: u64,
    ai_dump_flags: u64,
    error: Option<String>,
}

/// Verify a credential (`.c2pa` bytes) against the document bytes it should bind to.
#[wasm_bindgen]
pub fn verify_credential(manifest: &[u8], document: &[u8]) -> JsValue {
    let result = match read_sidecar(manifest, document) {
        Ok(readout) => VerifyResult {
            valid: readout.valid,
            claim: readout.claim,
            document_sha256: readout.record.document_binding.final_text_sha256,
            char_count: readout.record.document_binding.char_count,
            ai_dump_flags: readout.record.evidence_flags.large_unkeyed_insertions,
            error: None,
        },
        Err(e) => VerifyResult {
            valid: false,
            claim: String::new(),
            document_sha256: String::new(),
            char_count: 0,
            ai_dump_flags: 0,
            error: Some(e.to_string()),
        },
    };
    serde_wasm_bindgen::to_value(&result).unwrap_or(JsValue::NULL)
}
