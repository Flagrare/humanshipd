//! C2PA-based credential issuance/verification (adopts the standard envelope).
//!
//! The writing-process record is carried as a C2PA **custom assertion**; signing,
//! timestamping, and verification are delegated to `c2pa-rs`. This replaces the
//! bespoke Badge/Ed25519/timestamp envelope.
//!
//! The POC signs with c2pa's `EphemeralSigner` (self-signed → validates but is
//! untrusted), which matches the honest local-issuance model. Production swaps in
//! a real cert chain via `Context::with_settings`.

use crate::canonical::sha256_hex;
use crate::error::CoreError;
use crate::record::WritingSessionRecord;
use c2pa::assertions::DataHash;
use c2pa::{Builder, Context, EphemeralSigner, Reader, Signer, ValidationState};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::io::Cursor;

/// MIME for a standalone C2PA manifest store (the sidecar/external-manifest form).
const MANIFEST_STORE_FORMAT: &str = "application/c2pa";

/// Reverse-DNS label for our process-metadata assertion.
/// (No `.vN` suffix — c2pa interprets that as an assertion version and strips it.)
pub const PROCESS_ASSERTION: &str = "org.humanshipd.process";
/// Reverse-DNS label for the **self-asserted** (unverified) author name.
/// (Verified identity is the future CAWG-identity-assertion path — not this.)
pub const AUTHOR_ASSERTION: &str = "org.humanshipd.author";
/// IPTC digitalSourceType for text composed by a human with non-generative tools
/// — the de-facto "human, not AI-generated" baseline (signals spec §9).
const DIGITAL_CREATION: &str = "http://cv.iptc.org/newscodes/digitalsourcetype/digitalCreation";

/// A self-asserted author name carried in the signed manifest (tamper-evident, but
/// not independently verified — a local-only tool cannot attest identity).
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AuthorAssertion {
    name: String,
}

fn c2pa_err(e: impl std::fmt::Display) -> CoreError {
    CoreError::Crypto(e.to_string())
}

/// Issue a credential (POC): embed a signed C2PA manifest into `asset`, with the
/// process record as a custom assertion, using a self-signed ephemeral signer.
pub fn issue_ephemeral(
    record: &WritingSessionRecord,
    asset_format: &str,
    asset: &[u8],
) -> Result<Vec<u8>, CoreError> {
    let signer = EphemeralSigner::new("humanshipd.local").map_err(c2pa_err)?;
    let context = Context::new().with_signer(signer);
    issue_with_context(record, asset_format, asset, context)
}

fn issue_with_context(
    record: &WritingSessionRecord,
    asset_format: &str,
    asset: &[u8],
    context: Context,
) -> Result<Vec<u8>, CoreError> {
    let mut builder = Builder::from_context(context)
        .with_definition(serde_json::json!({
            "title": "Human Authored credential",
            "claim_generator_info": [{ "name": "humanshipd", "version": env!("CARGO_PKG_VERSION") }]
        }))
        .map_err(c2pa_err)?;

    builder
        .add_assertion(PROCESS_ASSERTION, record)
        .map_err(c2pa_err)?;
    builder
        .add_action(serde_json::json!({
            "action": "c2pa.created",
            "digitalSourceType": DIGITAL_CREATION
        }))
        .map_err(c2pa_err)?;

    let mut source = Cursor::new(asset.to_vec());
    let mut dest = Cursor::new(Vec::new());
    builder
        .save_to_stream(asset_format, &mut source, &mut dest)
        .map_err(c2pa_err)?;
    Ok(dest.into_inner())
}

/// Issue a **standalone (sidecar) `.c2pa` credential** bound to `file_bytes` by a
/// data hash (the first-class, format-agnostic Layer-1 path). The credential is a
/// manifest store carrying the process record + `c2pa.created` action; it binds to
/// the exported file by `sha256(file_bytes)` without storing the file.
pub fn issue_sidecar(
    record: &WritingSessionRecord,
    file_bytes: &[u8],
) -> Result<Vec<u8>, CoreError> {
    issue_sidecar_with_author(record, file_bytes, None)
}

/// Like [`issue_sidecar`], but also embeds a **self-asserted** author name as a
/// schema.org `CreativeWork` assertion (signals spec §9 / Phase 4). The name is
/// signed (so it is tamper-evident) but **not** independently verified — a
/// local-only tool cannot attest identity. Maps onto a CAWG identity assertion
/// later if a real identity signer is ever supplied.
pub fn issue_sidecar_with_author(
    record: &WritingSessionRecord,
    file_bytes: &[u8],
    author: Option<&str>,
) -> Result<Vec<u8>, CoreError> {
    let signer = EphemeralSigner::new("humanshipd.local").map_err(c2pa_err)?;
    let mut builder = Builder::from_context(Context::new())
        .with_definition(serde_json::json!({
            "title": "Human Authored credential",
            "claim_generator_info": [{ "name": "humanshipd", "version": env!("CARGO_PKG_VERSION") }]
        }))
        .map_err(c2pa_err)?;
    builder
        .add_assertion(PROCESS_ASSERTION, record)
        .map_err(c2pa_err)?;
    if let Some(name) = author.map(str::trim).filter(|n| !n.is_empty()) {
        builder
            .add_assertion(AUTHOR_ASSERTION, &AuthorAssertion { name: name.to_string() })
            .map_err(c2pa_err)?;
    }
    builder
        .add_action(serde_json::json!({
            "action": "c2pa.created",
            "digitalSourceType": DIGITAL_CREATION
        }))
        .map_err(c2pa_err)?;

    // Durable (soft) binding: an ISCC content fingerprint over the text, so a
    // reformatted/lightly-edited copy can still be matched back to this credential.
    if let Ok(text) = std::str::from_utf8(file_bytes) {
        if let Some(soft_binding) = crate::fingerprint::text_soft_binding(text) {
            builder
                .add_assertion("c2pa.soft-binding", &soft_binding)
                .map_err(c2pa_err)?;
        }
    }

    builder
        .data_hashed_placeholder(signer.reserve_size(), MANIFEST_STORE_FORMAT)
        .map_err(c2pa_err)?;

    let mut data_hash = DataHash::new("humanshipd.file", "sha256");
    data_hash.set_hash(Sha256::digest(file_bytes).to_vec());

    builder
        .sign_data_hashed_embeddable(&signer, &data_hash, MANIFEST_STORE_FORMAT)
        .map_err(c2pa_err)
}

/// Read a standalone sidecar credential and re-bind it to `file_bytes`: validates
/// the C2PA signature, extracts the process record, and confirms the bound hash
/// matches `sha256(file_bytes)`.
pub fn read_sidecar(
    manifest: &[u8],
    file_bytes: &[u8],
) -> Result<CredentialReadout, CoreError> {
    let reader = Reader::from_context(Context::new())
        .with_manifest_data_and_stream(
            manifest,
            MANIFEST_STORE_FORMAT,
            Cursor::new(file_bytes.to_vec()),
        )
        .map_err(c2pa_err)?;
    let signature_ok = matches!(
        reader.validation_state(),
        ValidationState::Valid | ValidationState::Trusted
    );
    let manifest_obj = reader
        .active_manifest()
        .ok_or_else(|| CoreError::Crypto("no active manifest".to_string()))?;
    let record = manifest_obj
        .find_assertion::<WritingSessionRecord>(PROCESS_ASSERTION)
        .map_err(c2pa_err)?;
    let hash_ok = record.document_binding.final_text_sha256 == sha256_hex(file_bytes);
    let author = self_asserted_author(manifest_obj);
    Ok(readout(signature_ok && hash_ok, record, author))
}

/// Extract the self-asserted author name, if the manifest carries one.
fn self_asserted_author(manifest: &c2pa::Manifest) -> Option<String> {
    manifest
        .find_assertion::<AuthorAssertion>(AUTHOR_ASSERTION)
        .ok()
        .map(|a| a.name)
}

/// Verification outcome from reading a signed manifest.
pub struct CredentialReadout {
    /// True if the C2PA signature + bindings validate (Valid or Trusted).
    pub valid: bool,
    pub record: WritingSessionRecord,
    /// Honest, non-overclaiming human-readable claim.
    pub claim: String,
    /// Self-asserted author name, if the credential carries one. **Not**
    /// independently verified — see [`issue_sidecar_with_author`].
    pub author: Option<String>,
}

fn readout(valid: bool, record: WritingSessionRecord, author: Option<String>) -> CredentialReadout {
    let claim = render_claim(valid, record.evidence_flags.large_unkeyed_insertions);
    CredentialReadout {
        valid,
        record,
        claim,
        author,
    }
}

/// Render the honest claim. Never asserts a human originated the ideas.
fn render_claim(valid: bool, large_unkeyed_insertions: u64) -> String {
    if !valid {
        return "INVALID — this credential failed verification (altered, or the document does not match).".to_string();
    }
    let mut parts = vec![
        "Verified C2PA credential: signed and unaltered since issuance, bound to this exact document.".to_string(),
    ];
    if large_unkeyed_insertions > 0 {
        let (n, noun) = (
            large_unkeyed_insertions,
            if large_unkeyed_insertions == 1 { "insertion" } else { "insertions" },
        );
        parts.push(format!(
            "WARNING: {n} large {noun} appeared without typing — possible paste of AI-generated text."
        ));
    } else {
        parts.push(
            "The writing showed an incremental, human-like process with no large un-keyed insertions."
                .to_string(),
        );
    }
    parts.push("This attests process integrity, not that a human originated the ideas.".to_string());
    parts.join(" ")
}

/// Read + validate a signed asset, returning the embedded process record.
pub fn read(asset_format: &str, signed_asset: &[u8]) -> Result<CredentialReadout, CoreError> {
    let reader = Reader::from_context(Context::new())
        .with_stream(asset_format, Cursor::new(signed_asset.to_vec()))
        .map_err(c2pa_err)?;
    let valid = matches!(
        reader.validation_state(),
        ValidationState::Valid | ValidationState::Trusted
    );
    let manifest = reader
        .active_manifest()
        .ok_or_else(|| CoreError::Crypto("no active manifest".to_string()))?;
    let record = manifest
        .find_assertion::<WritingSessionRecord>(PROCESS_ASSERTION)
        .map_err(c2pa_err)?;
    let author = self_asserted_author(manifest);
    Ok(readout(valid, record, author))
}
