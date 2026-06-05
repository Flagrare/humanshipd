use crate::canonical::{canonicalize, sha256_hex};
use crate::error::CoreError;
use crate::record::WritingSessionRecord;
use crate::signing::{verify_signature, KeyPair};
use serde::{Deserialize, Serialize};

/// A signed, verifiable credential: the record plus its integrity block.
///
/// The integrity block wraps the record rather than nesting inside it, so the
/// signed payload (the canonical record) never contains its own signature.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Badge {
    pub record: WritingSessionRecord,
    pub integrity: Integrity,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Integrity {
    /// Hex SHA-256 of the canonical record (what the signature/timestamp cover).
    pub record_sha256: String,
    /// Hex Ed25519 public key of the issuing client.
    pub public_key: String,
    /// Hex Ed25519 signature over the canonical record bytes.
    pub client_signature: String,
    /// RFC 3161 timestamp token (base64), added by the trust-anchor step.
    pub rfc3161_token: Option<String>,
}

/// Sign a record, producing a badge with no timestamp yet (added later).
pub fn sign_record(record: WritingSessionRecord, keypair: &KeyPair) -> Result<Badge, CoreError> {
    let canonical = canonicalize(&record)?;
    Ok(Badge {
        integrity: Integrity {
            record_sha256: sha256_hex(&canonical),
            public_key: keypair.public_key_hex(),
            client_signature: keypair.sign_hex(&canonical),
            rfc3161_token: None,
        },
        record,
    })
}

/// Verify the signature leg of a badge: the stored hash must match the
/// recomputed canonical hash, and the signature must validate over the canonical
/// record bytes. Returns `Ok(false)` for any tamper; `Err` only for malformed inputs.
pub fn verify_badge_signature(badge: &Badge) -> Result<bool, CoreError> {
    let canonical = canonicalize(&badge.record)?;
    if sha256_hex(&canonical) != badge.integrity.record_sha256 {
        return Ok(false);
    }
    verify_signature(
        &badge.integrity.public_key,
        &canonical,
        &badge.integrity.client_signature,
    )
}
