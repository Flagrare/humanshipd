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

/// Content-Code length in bits. 256 (not the coarse 64) — corpus calibration showed
/// real multi-revision edits stay ≤ ~0.12 normalized Hamming while unrelated text is
/// ≥ ~0.44, a clean gap that 64 bits smears (Decision 4 spec, 2026-06-08).
const CONTENT_CODE_BITS: u32 = 256;

/// Locked verdict bands (256-bit Content-Code, normalized Hamming) — Decision 4.
/// Reformatting / format conversion / light edits.
pub const BAND_SAME_CONTENT_MAX: f64 = 0.05;
/// Heavy multi-revision editing (observed real-corpus max ≈ 0.12).
pub const BAND_SAME_WRITING_MAX: f64 = 0.20;
/// The empty zone between same-writing and unrelated — surfaced for human review.
pub const BAND_BORDERLINE_MAX: f64 = 0.35;

/// A C2PA soft binding: the algorithm and the resulting fingerprint value.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SoftBinding {
    pub alg: String,
    pub value: String,
}

/// Where a candidate's Content-Code falls relative to a credential's stored one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Band {
    /// ≤ 0.05 — same content, reformatted/converted/lightly edited.
    SameContent,
    /// ≤ 0.20 — same writing, heavily edited across revisions.
    SameWriting,
    /// 0.20–0.35 — borderline; show honestly, needs review.
    Borderline,
    /// > 0.35 — unrelated content.
    NoMatch,
}

/// Classify a normalized Hamming distance into a locked verdict band.
pub fn classify(distance: f64) -> Band {
    if distance <= BAND_SAME_CONTENT_MAX {
        Band::SameContent
    } else if distance <= BAND_SAME_WRITING_MAX {
        Band::SameWriting
    } else if distance <= BAND_BORDERLINE_MAX {
        Band::Borderline
    } else {
        Band::NoMatch
    }
}

/// Compute an ISCC Text-Code (ISO 24138) for `text` at [`CONTENT_CODE_BITS`].
pub fn text_iscc(text: &str) -> Result<String, CoreError> {
    iscc_lib::gen_text_code_v0(text, CONTENT_CODE_BITS)
        .map(|result| result.iscc)
        .map_err(|e| CoreError::Crypto(format!("iscc: {e}")))
}

/// Normalized Hamming distance (bits-different ÷ total-bits) between two ISCC
/// Content-Codes. `None` if either code can't be decoded to a full 256-bit digest
/// or the digests differ in length (e.g. an old 64-bit code vs a 256-bit one) —
/// incomparable codes must not masquerade as a distance of 0.
pub fn iscc_distance(a: &str, b: &str) -> Option<f64> {
    let (da, db) = (iscc_digest(a)?, iscc_digest(b)?);
    if da.len() != db.len() {
        return None;
    }
    let diff: u32 = da.iter().zip(&db).map(|(x, y)| (x ^ y).count_ones()).sum();
    Some(diff as f64 / (da.len() * 8) as f64)
}

/// Decode an `ISCC:…` code to its trailing 256-bit (32-byte) digest, or `None` if
/// the body is shorter than 256 bits.
fn iscc_digest(code: &str) -> Option<Vec<u8>> {
    let body = code.strip_prefix("ISCC:").unwrap_or(code);
    let bytes = base32_decode(body);
    (bytes.len() >= 32).then(|| bytes[bytes.len() - 32..].to_vec())
}

/// RFC 4648 base32 decode (uppercase, no padding) — the ISCC code alphabet.
fn base32_decode(s: &str) -> Vec<u8> {
    const ALPHA: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    let (mut bits, mut nbits, mut out) = (0u32, 0u32, Vec::new());
    for c in s.bytes() {
        let Some(v) = ALPHA.iter().position(|&x| x == c) else {
            continue;
        };
        bits = (bits << 5) | v as u32;
        nbits += 5;
        if nbits >= 8 {
            nbits -= 8;
            out.push((bits >> nbits) as u8);
        }
    }
    out
}

/// Build an ISCC soft binding for UTF-8 `text`, if a code can be computed.
pub fn text_soft_binding(text: &str) -> Option<SoftBinding> {
    text_iscc(text).ok().map(|value| SoftBinding {
        alg: ISCC_ALG.to_string(),
        value,
    })
}
