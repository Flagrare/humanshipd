//! C2PA-based credential issuance/verification (adopts the standard envelope).
//!
//! The writing-process record is carried as a C2PA **custom assertion**; signing,
//! timestamping, and verification are delegated to `c2pa-rs`. This replaces the
//! bespoke Badge/Ed25519/timestamp envelope.
//!
//! The POC signs with c2pa's `EphemeralSigner` (self-signed → validates but is
//! untrusted), which matches the honest local-issuance model. Production swaps in
//! a real cert chain via `Context::with_settings`.

use crate::error::CoreError;
use crate::record::WritingSessionRecord;
use c2pa::{Builder, Context, EphemeralSigner, Reader, ValidationState};
use std::io::Cursor;

/// Reverse-DNS label for our process-metadata assertion.
/// (No `.vN` suffix — c2pa interprets that as an assertion version and strips it.)
pub const PROCESS_ASSERTION: &str = "org.humanshipd.process";
const DIGITAL_CAPTURE: &str = "http://cv.iptc.org/newscodes/digitalsourcetype/digitalCapture";

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
            "digitalSourceType": DIGITAL_CAPTURE
        }))
        .map_err(c2pa_err)?;

    let mut source = Cursor::new(asset.to_vec());
    let mut dest = Cursor::new(Vec::new());
    builder
        .save_to_stream(asset_format, &mut source, &mut dest)
        .map_err(c2pa_err)?;
    Ok(dest.into_inner())
}

/// Verification outcome from reading a signed manifest.
pub struct CredentialReadout {
    /// True if the C2PA signature + bindings validate (Valid or Trusted).
    pub valid: bool,
    pub record: WritingSessionRecord,
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
    Ok(CredentialReadout { valid, record })
}
