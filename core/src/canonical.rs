use crate::error::CoreError;
use serde::Serialize;
use sha2::{Digest, Sha256};

/// Canonicalize a serializable value to deterministic JCS bytes (RFC 8785).
///
/// Determinism is the load-bearing property: the native build and the WASM
/// build run this exact code, so a record produced by either hashes identically.
pub fn canonicalize<T: Serialize>(value: &T) -> Result<Vec<u8>, CoreError> {
    serde_jcs::to_vec(value).map_err(|e| CoreError::Serialization(e.to_string()))
}

/// Hex-encoded SHA-256 of the canonical form of `value`.
pub fn canonical_sha256<T: Serialize>(value: &T) -> Result<String, CoreError> {
    Ok(sha256_hex(&canonicalize(value)?))
}

/// Hex-encoded SHA-256 of arbitrary bytes.
pub fn sha256_hex(bytes: &[u8]) -> String {
    hex::encode(Sha256::digest(bytes))
}
