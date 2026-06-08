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
use crate::fingerprint::{
    self, Band, SoftBinding, BAND_BORDERLINE_MAX, BAND_SAME_CONTENT_MAX, BAND_SAME_WRITING_MAX,
};
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
    issue_sidecar_signed(record, file_bytes, author, &signer)
}

/// Like [`issue_sidecar_with_author`], but also attaches an **RFC 3161 timestamp**
/// obtained from `tsa_url` — the one CA-free, standardized "existence proof" (it
/// attests the credential was signed before time T and keeps it verifiable after
/// cert expiry; Decision 6). This makes a network call to the TSA, so it is **opt-in
/// and native-only** — never part of the zero-telemetry default.
#[cfg(feature = "native")]
pub fn issue_sidecar_timestamped(
    record: &WritingSessionRecord,
    file_bytes: &[u8],
    author: Option<&str>,
    tsa_url: &str,
) -> Result<Vec<u8>, CoreError> {
    let inner = EphemeralSigner::new("humanshipd.local").map_err(c2pa_err)?;
    let signer = TimestampingSigner { inner, tsa_url: tsa_url.to_string() };
    issue_sidecar_signed(record, file_bytes, author, &signer)
}

/// Build + sign the sidecar manifest with an arbitrary [`Signer`] (the shared body
/// behind the self-signed and timestamped issuers).
fn issue_sidecar_signed(
    record: &WritingSessionRecord,
    file_bytes: &[u8],
    author: Option<&str>,
    signer: &dyn Signer,
) -> Result<Vec<u8>, CoreError> {
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
        .sign_data_hashed_embeddable(signer, &data_hash, MANIFEST_STORE_FORMAT)
        .map_err(c2pa_err)
}

/// Wraps an [`EphemeralSigner`] to request an RFC 3161 timestamp from `tsa_url`
/// during signing (Decision 6, Part B). c2pa fetches and embeds the token when a
/// signer's [`Signer::time_authority_url`] is set.
#[cfg(feature = "native")]
struct TimestampingSigner {
    inner: EphemeralSigner,
    tsa_url: String,
}

#[cfg(feature = "native")]
impl Signer for TimestampingSigner {
    fn sign(&self, data: &[u8]) -> c2pa::Result<Vec<u8>> {
        self.inner.sign(data)
    }
    fn alg(&self) -> c2pa::SigningAlg {
        self.inner.alg()
    }
    fn certs(&self) -> c2pa::Result<Vec<Vec<u8>>> {
        self.inner.certs()
    }
    /// Extra headroom over the bare signature for the embedded RFC 3161 token.
    fn reserve_size(&self) -> usize {
        self.inner.reserve_size() + 10_000
    }
    fn time_authority_url(&self) -> Option<String> {
        Some(self.tsa_url.clone())
    }
}

/// Read a standalone sidecar credential and grade it against `file_bytes` (Decision
/// 4). The manifest is read **standalone**, so signature authenticity is judged
/// independently of which file is presented; the file binding is then computed here
/// — byte-exact (SHA-256) first, falling back to the ISCC content fingerprint — and
/// returned as a tiered [`Verdict`].
pub fn read_sidecar(
    manifest: &[u8],
    file_bytes: &[u8],
) -> Result<CredentialReadout, CoreError> {
    let text = std::str::from_utf8(file_bytes).ok();
    read_sidecar_inner(manifest, file_bytes, text)
}

/// Verify against a document whose text was **extracted from a binary container**
/// (e.g. a `.docx` or `.pdf` via [`crate::formats`]). The byte-exact hard binding
/// still uses the raw `file_bytes`, but the content fingerprint is computed over
/// `text` — so the same writing in another format lands on the content tiers
/// instead of failing the UTF-8 check.
pub fn read_sidecar_with_text(
    manifest: &[u8],
    file_bytes: &[u8],
    text: &str,
) -> Result<CredentialReadout, CoreError> {
    read_sidecar_inner(manifest, file_bytes, Some(text))
}

fn read_sidecar_inner(
    manifest: &[u8],
    file_bytes: &[u8],
    text: Option<&str>,
) -> Result<CredentialReadout, CoreError> {
    let reader = Reader::from_context(Context::new())
        .with_stream(MANIFEST_STORE_FORMAT, Cursor::new(manifest.to_vec()))
        .map_err(c2pa_err)?;
    let authentic = signature_authentic(&reader);
    let manifest_obj = reader
        .active_manifest()
        .ok_or_else(|| CoreError::Crypto("no active manifest".to_string()))?;
    let record = manifest_obj
        .find_assertion::<WritingSessionRecord>(PROCESS_ASSERTION)
        .map_err(c2pa_err)?;
    let author = self_asserted_author(manifest_obj);
    let trust = trust_status(&reader, manifest_obj);

    let verdict = if !authentic {
        Verdict::Invalid
    } else if record.document_binding.final_text_sha256 == sha256_hex(file_bytes) {
        Verdict::ExactFile
    } else {
        content_verdict(manifest_obj, text)
    };
    Ok(readout(verdict, trust, record, author))
}

/// Is the credential's own signature genuine, regardless of which file we checked
/// against? True when the COSE claim signature validated and the only validation
/// failures are *expected* for our model — see [`is_expected_failure`].
fn signature_authentic(reader: &Reader) -> bool {
    let Some(codes) = reader.validation_results().and_then(|r| r.active_manifest()) else {
        return false;
    };
    let signed = codes
        .success()
        .iter()
        .any(|s| s.code() == "claimSignature.validated");
    let fatal = codes
        .failure()
        .iter()
        .any(|s| !is_expected_failure(s.code()));
    signed && !fatal
}

/// Failure codes that don't impugn authenticity. A self-signed local credential is
/// `untrusted` by design (Decision 6); `assertion.dataHash.mismatch` is expected
/// because we read the manifest standalone and bind the file ourselves; an untrusted
/// timestamp is likewise expected without a trusted TSA.
fn is_expected_failure(code: &str) -> bool {
    matches!(
        code,
        "signingCredential.untrusted" | "assertion.dataHash.mismatch" | "timeStamp.untrusted"
    )
}

/// Grade an authentic credential against a non-byte-exact file via the ISCC
/// content fingerprint stored as the `c2pa.soft-binding` assertion. `candidate_text`
/// is the document's extracted text (`None` ⇒ the file wasn't text and no extractor
/// ran, so there is nothing comparable).
fn content_verdict(manifest: &c2pa::Manifest, candidate_text: Option<&str>) -> Verdict {
    let stored = manifest.find_assertion::<SoftBinding>("c2pa.soft-binding").ok();
    let candidate = candidate_text.and_then(|text| fingerprint::text_iscc(text).ok());
    let distance = match (stored, candidate) {
        (Some(sb), Some(code)) => fingerprint::iscc_distance(&sb.value, &code),
        _ => None,
    };
    match distance {
        Some(d) => match fingerprint::classify(d) {
            Band::SameContent => Verdict::SameContent { distance: d },
            Band::SameWriting => Verdict::SameWriting { distance: d },
            Band::Borderline => Verdict::Borderline { distance: d },
            Band::NoMatch => Verdict::NoMatch { distance: Some(d) },
        },
        None => Verdict::NoMatch { distance: None },
    }
}

/// Extract the self-asserted author name, if the manifest carries one.
fn self_asserted_author(manifest: &c2pa::Manifest) -> Option<String> {
    manifest
        .find_assertion::<AuthorAssertion>(AUTHOR_ASSERTION)
        .ok()
        .map(|a| a.name)
}

/// The tiered verification verdict (Decision 4). A credential is either forged
/// (`Invalid`) or genuine — in which case the file binding grades from a byte-exact
/// match down to no match via the ISCC content fingerprint.
#[derive(Debug, Clone, PartialEq)]
pub enum Verdict {
    /// The credential's own signature failed — altered or forged. (Says nothing
    /// about the file; the credential itself is not trustworthy.)
    Invalid,
    /// Authentic, and the byte-exact SHA-256 hard binding matches this file.
    ExactFile,
    /// Authentic; content fingerprint within the same-content band (reformatted,
    /// format-converted, or lightly edited).
    SameContent { distance: f64 },
    /// Authentic; same writing, heavily edited across revisions.
    SameWriting { distance: f64 },
    /// Authentic, but the file sits in the borderline band — needs human review.
    Borderline { distance: f64 },
    /// Authentic credential, but this file is not the writing it covers (beyond the
    /// threshold, or no comparable content fingerprint).
    NoMatch { distance: Option<f64> },
}

impl Verdict {
    /// Whether this file is the writing the credential covers — exactly, reformatted,
    /// or edited. Borderline/NoMatch/Invalid are not matches.
    pub fn is_match(&self) -> bool {
        matches!(
            self,
            Verdict::ExactFile | Verdict::SameContent { .. } | Verdict::SameWriting { .. }
        )
    }
}

/// What the signature does and does not establish (Decision 6). A local-only tool
/// produces a *valid* (tamper-evident) but *untrusted* (no CA on a trust list) and
/// *identity-unverified* credential — this carries that honestly to the verifier.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustStatus {
    /// The COSE claim signature validated — the credential is authentic and the
    /// content is unaltered since issuance.
    pub signed: bool,
    /// The signing certificate chains to a CA on the validator's trust list. For
    /// the self-signed local default this is **false** by design (Decision 6).
    pub trusted: bool,
    /// The signer's real-world identity has been independently verified. Always
    /// **false** today; the slot is reserved for an opt-in CAWG identity assertion.
    pub identity_verified: bool,
    /// RFC 3161 attested signing time, if the credential was timestamped (Part B).
    pub timestamp: Option<String>,
}

/// Verification outcome from reading a signed manifest.
pub struct CredentialReadout {
    /// True if the credential is authentic **and** this file is the writing it
    /// covers (exact, reformatted, or edited). Convenience over [`Self::verdict`].
    pub valid: bool,
    /// The full tiered verdict (Decision 4).
    pub verdict: Verdict,
    /// What the signature establishes — and what it deliberately doesn't (Decision 6).
    pub trust: TrustStatus,
    pub record: WritingSessionRecord,
    /// Honest, non-overclaiming human-readable claim.
    pub claim: String,
    /// Self-asserted author name, if the credential carries one. **Not**
    /// independently verified — see [`issue_sidecar_with_author`].
    pub author: Option<String>,
}

/// Read the honest trust status from a parsed manifest + its validation results.
fn trust_status(reader: &Reader, manifest: &c2pa::Manifest) -> TrustStatus {
    let codes = reader.validation_results().and_then(|r| r.active_manifest());
    let has = |code: &str| {
        codes.is_some_and(|c| c.success().iter().any(|s| s.code() == code))
    };
    TrustStatus {
        signed: has("claimSignature.validated"),
        trusted: has("signingCredential.trusted"),
        // No verified identity assertion is consumed yet — the self-asserted author
        // name is explicitly unverified. Flips on when a CAWG identity assertion lands.
        identity_verified: false,
        timestamp: manifest.time(),
    }
}

fn readout(
    verdict: Verdict,
    trust: TrustStatus,
    record: WritingSessionRecord,
    author: Option<String>,
) -> CredentialReadout {
    let claim = render_claim(&verdict, record.evidence_flags.large_unkeyed_insertions);
    CredentialReadout {
        valid: verdict.is_match(),
        verdict,
        trust,
        record,
        claim,
        author,
    }
}

/// Render the honest claim for a verdict. Never asserts a human originated the ideas.
fn render_claim(verdict: &Verdict, large_unkeyed_insertions: u64) -> String {
    let pct = |d: f64| format!("{:.0}%", d * 100.0);
    let process = process_note(large_unkeyed_insertions);
    match verdict {
        Verdict::Invalid => "INVALID — this credential failed verification: its signature is \
            broken or forged, so nothing it claims can be trusted."
            .to_string(),
        Verdict::ExactFile => {
            format!("Verified: signed and unaltered, bound to this exact file. {process}")
        }
        Verdict::SameContent { distance } => format!(
            "Verified credential for this writing — same content, reformatted or lightly edited \
             (content distance {}, within the {} same-content threshold). {process}",
            pct(*distance),
            pct(BAND_SAME_CONTENT_MAX)
        ),
        Verdict::SameWriting { distance } => format!(
            "Verified credential for this writing — an edited version of the credentialed text \
             (content distance {}, within the {} same-writing threshold). {process}",
            pct(*distance),
            pct(BAND_SAME_WRITING_MAX)
        ),
        Verdict::Borderline { distance } => format!(
            "BORDERLINE — this file sits near the edge of the match threshold (content distance \
             {}, just past the {} same-writing line). A human should judge whether it is the \
             same writing.",
            pct(*distance),
            pct(BAND_SAME_WRITING_MAX)
        ),
        Verdict::NoMatch { distance } => match distance {
            Some(d) => format!(
                "NO MATCH — this file is not the writing this credential covers (content distance \
                 {}, beyond the {} threshold).",
                pct(*d),
                pct(BAND_BORDERLINE_MAX)
            ),
            None => "NO MATCH — this file is not the writing this credential covers, and no \
                content fingerprint could be compared."
                .to_string(),
        },
    }
}

/// The process-integrity sentence shared by the matching tiers.
fn process_note(large_unkeyed_insertions: u64) -> String {
    let tail = "This attests process integrity, not that a human originated the ideas.";
    if large_unkeyed_insertions > 0 {
        let noun = if large_unkeyed_insertions == 1 {
            "insertion"
        } else {
            "insertions"
        };
        format!(
            "WARNING: {large_unkeyed_insertions} large {noun} appeared without typing — possible \
             paste of AI-generated text. {tail}"
        )
    } else {
        format!("The writing showed an incremental, human-like process with no large un-keyed insertions. {tail}")
    }
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
    let trust = trust_status(&reader, manifest);
    // Embedded assets are covered byte-for-byte by the C2PA data hash, so a valid
    // state is an exact-file match; otherwise the manifest itself failed.
    let verdict = if valid { Verdict::ExactFile } else { Verdict::Invalid };
    Ok(readout(verdict, trust, record, author))
}
