use crate::canonical::canonicalize;
use crate::error::CoreError;
use crate::signing::{verify_signature, KeyPair};
use serde::{Deserialize, Serialize};

/// A time-anchoring token for a record's hash.
///
/// **POC note:** this is *not* an RFC 3161 DER/CMS token. The POC ships a local
/// authority (see [`LocalTsa`]); production swaps in a real RFC 3161 client by
/// implementing [`TimestampAuthority`] (Strategy) without changing this type's
/// role or the verification logic. The `authority` field carries an honest label
/// so the verify surface never implies a real TSA.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TimestampToken {
    /// Honest authority label, e.g. "humanshipd:local-poc".
    pub authority: String,
    /// The record hash this token attests (hex SHA-256).
    pub message_imprint_sha256: String,
    /// ISO-8601 time asserted by the authority.
    pub gen_time: String,
    /// Hex public key of the authority.
    pub tsa_public_key: String,
    /// Hex signature over the canonical token binding.
    pub tsa_signature: String,
}

/// The exact bytes an authority signs / a verifier checks (excludes the signature).
#[derive(Serialize)]
struct TokenBinding<'a> {
    authority: &'a str,
    gen_time: &'a str,
    message_imprint_sha256: &'a str,
}

/// Pluggable time-anchoring strategy. The POC uses [`LocalTsa`]; production
/// implements this over a real RFC 3161 TSA.
pub trait TimestampAuthority {
    fn timestamp(&self, message_imprint_sha256: &str) -> Result<TimestampToken, CoreError>;
}

/// POC authority: signs the binding with a local Ed25519 key and an
/// authority-supplied clock. Time is injected (not read from the OS) so the core
/// stays WASM-safe and deterministically testable.
pub struct LocalTsa {
    keypair: KeyPair,
    authority: String,
    gen_time: String,
}

impl LocalTsa {
    pub fn new(seed: &[u8; 32], authority: impl Into<String>, gen_time: impl Into<String>) -> Self {
        Self {
            keypair: KeyPair::from_seed(seed),
            authority: authority.into(),
            gen_time: gen_time.into(),
        }
    }
}

impl TimestampAuthority for LocalTsa {
    fn timestamp(&self, message_imprint_sha256: &str) -> Result<TimestampToken, CoreError> {
        let binding = TokenBinding {
            authority: &self.authority,
            gen_time: &self.gen_time,
            message_imprint_sha256,
        };
        let canonical = canonicalize(&binding)?;
        Ok(TimestampToken {
            authority: self.authority.clone(),
            message_imprint_sha256: message_imprint_sha256.to_string(),
            gen_time: self.gen_time.clone(),
            tsa_public_key: self.keypair.public_key_hex(),
            tsa_signature: self.keypair.sign_hex(&canonical),
        })
    }
}

/// Verify a token: it must attest `expected_message_imprint` and carry a valid
/// authority signature over its binding. Returns `Ok(false)` on any mismatch.
pub fn verify_timestamp(
    token: &TimestampToken,
    expected_message_imprint: &str,
) -> Result<bool, CoreError> {
    if token.message_imprint_sha256 != expected_message_imprint {
        return Ok(false);
    }
    let binding = TokenBinding {
        authority: &token.authority,
        gen_time: &token.gen_time,
        message_imprint_sha256: &token.message_imprint_sha256,
    };
    let canonical = canonicalize(&binding)?;
    verify_signature(&token.tsa_public_key, &canonical, &token.tsa_signature)
}
