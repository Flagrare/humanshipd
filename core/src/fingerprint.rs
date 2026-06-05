//! Layer 3a: ISCC content fingerprint (ISO 24138).
//!
//! Unlike the exact SHA-256 hard binding, an ISCC Text-Code is *similarity-
//! preserving*: a lightly-edited or reformatted copy of the same document yields
//! the same or a near-identical code. That's what lets a credential survive
//! copy-paste and minor changes — the durable (soft) binding behind Durable
//! Content Credentials. Stored in the manifest as a `c2pa.soft-binding` assertion.
//!
//! Adopts `iscc-lib` (ISO 24138). See research:
//! `docs/research/2026-06-05-adopting-c2pa-credential-stack.md`.

use crate::error::CoreError;
use serde::{Deserialize, Serialize};

/// Soft-binding algorithm identifier registered in the C2PA list for ISCC.
pub const ISCC_ALG: &str = "io.iscc.v0";

/// A C2PA soft binding: the algorithm and the resulting fingerprint value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SoftBinding {
    pub alg: String,
    pub value: String,
}

/// Compute an ISCC Text-Code (ISO 24138, 64-bit) for `text`.
pub fn text_iscc(text: &str) -> Result<String, CoreError> {
    iscc_lib::gen_text_code_v0(text, 64)
        .map(|result| result.iscc)
        .map_err(|e| CoreError::Crypto(format!("iscc: {e}")))
}

/// Build an ISCC soft binding for UTF-8 `text`, if a code can be computed.
pub fn text_soft_binding(text: &str) -> Option<SoftBinding> {
    text_iscc(text).ok().map(|value| SoftBinding {
        alg: ISCC_ALG.to_string(),
        value,
    })
}
