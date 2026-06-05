use crate::error::CoreError;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};

/// An Ed25519 keypair. Key generation/storage is the host's responsibility;
/// the core only needs to construct a keypair from a 32-byte seed and sign.
pub struct KeyPair {
    signing: SigningKey,
}

impl KeyPair {
    /// Build a keypair from a 32-byte seed (the host supplies randomness).
    pub fn from_seed(seed: &[u8; 32]) -> Self {
        Self {
            signing: SigningKey::from_bytes(seed),
        }
    }

    /// Hex-encoded 32-byte public (verifying) key.
    pub fn public_key_hex(&self) -> String {
        hex::encode(self.signing.verifying_key().to_bytes())
    }

    /// Hex-encoded 64-byte Ed25519 signature over `message`.
    pub fn sign_hex(&self, message: &[u8]) -> String {
        hex::encode(self.signing.sign(message).to_bytes())
    }
}

/// Verify a hex-encoded Ed25519 signature over `message` against a hex public key.
/// Returns `Ok(false)` for a well-formed-but-invalid signature; `Err` only for
/// malformed inputs (bad hex / wrong length / invalid key point).
pub fn verify_signature(
    public_key_hex: &str,
    message: &[u8],
    signature_hex: &str,
) -> Result<bool, CoreError> {
    let pk_bytes = decode_fixed::<32>(public_key_hex, "public key")?;
    let sig_bytes = decode_fixed::<64>(signature_hex, "signature")?;
    let verifying_key =
        VerifyingKey::from_bytes(&pk_bytes).map_err(|e| CoreError::Crypto(e.to_string()))?;
    let signature = Signature::from_bytes(&sig_bytes);
    Ok(verifying_key.verify(message, &signature).is_ok())
}

fn decode_fixed<const N: usize>(hex_str: &str, what: &str) -> Result<[u8; N], CoreError> {
    let bytes = hex::decode(hex_str).map_err(|e| CoreError::Encoding(format!("{what}: {e}")))?;
    bytes
        .as_slice()
        .try_into()
        .map_err(|_| CoreError::Encoding(format!("{what}: expected {N} bytes, got {}", bytes.len())))
}
