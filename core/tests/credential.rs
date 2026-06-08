use humanshipd_core::canonical::sha256_hex;
use humanshipd_core::credential::{self, Verdict};
use humanshipd_core::formats::extract_named;
use image::{ImageFormat, RgbaImage};
use std::io::Cursor;

mod common;
use common::{minimal_docx, sample_record};

fn small_png() -> Vec<u8> {
    let image = RgbaImage::new(8, 8);
    let mut out = Cursor::new(Vec::new());
    image.write_to(&mut out, ImageFormat::Png).expect("encode png");
    out.into_inner()
}

#[test]
fn sidecar_credential_round_trips_and_binds_to_file() {
    let file = b"the exported manuscript bytes (could be a PDF/EPUB/txt)";
    let mut record = sample_record();
    record.document_binding.final_text_sha256 = sha256_hex(file);

    let manifest = credential::issue_sidecar(&record, file).expect("issue sidecar");
    assert!(!manifest.is_empty(), "sidecar manifest must be produced");

    let readout = credential::read_sidecar(&manifest, file).expect("read sidecar");
    assert!(readout.valid, "sidecar must validate against its file");
    assert_eq!(readout.record, record);
}

#[test]
fn sidecar_credential_carries_an_iscc_soft_binding() {
    let file = "A reasonably long original document, several sentences long, so that \
        an ISCC text code can be derived for the durable soft binding."
        .as_bytes();
    let mut record = sample_record();
    record.document_binding.final_text_sha256 = sha256_hex(file);

    let manifest = credential::issue_sidecar(&record, file).expect("issue");
    let as_text = String::from_utf8_lossy(&manifest);
    assert!(
        as_text.contains("io.iscc.v0"),
        "credential should carry an ISCC soft binding"
    );
    assert!(credential::read_sidecar(&manifest, file).unwrap().valid);
}

#[test]
fn sidecar_credential_rejects_a_different_file() {
    let file = b"original exported file";
    let mut record = sample_record();
    record.document_binding.final_text_sha256 = sha256_hex(file);
    let manifest = credential::issue_sidecar(&record, file).unwrap();

    match credential::read_sidecar(&manifest, b"a tampered/different file") {
        Ok(readout) => assert!(!readout.valid, "different file must not validate"),
        Err(_) => { /* c2pa rejecting the data-hash mismatch outright is also fine */ }
    }
}

const ESSAY: &str = "Provenance beats inference. Rather than guessing whether a \
    passage was written by a machine, we record the process by which it was written \
    and bind a verifiable credential to the result. The credential attests how the \
    text came to be, not who originated the ideas behind it.";

#[test]
fn exact_same_file_reads_as_the_exact_file_tier() {
    let file = ESSAY.as_bytes();
    let mut record = sample_record();
    record.document_binding.final_text_sha256 = sha256_hex(file);
    let manifest = credential::issue_sidecar(&record, file).unwrap();

    let readout = credential::read_sidecar(&manifest, file).unwrap();
    assert!(matches!(readout.verdict, Verdict::ExactFile), "got {:?}", readout.verdict);
    assert!(readout.valid);
}

#[test]
fn reformatted_file_reads_as_a_content_match_not_invalid() {
    // The headline Decision-4 behaviour: the same writing, re-exported with
    // format whitespace noise, must read as a *content* match rather than INVALID.
    let original = ESSAY.as_bytes();
    let mut record = sample_record();
    record.document_binding.final_text_sha256 = sha256_hex(original);
    let manifest = credential::issue_sidecar(&record, original).unwrap();

    let reformatted = ESSAY.replace(' ', "  ").replace(". ", " .\n");
    let readout = credential::read_sidecar(&manifest, reformatted.as_bytes()).unwrap();

    assert!(
        matches!(readout.verdict, Verdict::SameContent { .. } | Verdict::SameWriting { .. }),
        "reformatted export should be a content match, got {:?}",
        readout.verdict
    );
    assert!(readout.valid, "a same-writing match is a valid credential");
}

#[test]
fn unrelated_file_reads_as_no_match() {
    let original = ESSAY.as_bytes();
    let mut record = sample_record();
    record.document_binding.final_text_sha256 = sha256_hex(original);
    let manifest = credential::issue_sidecar(&record, original).unwrap();

    let unrelated = "Tide charts and coral spawning cycles govern when the reef \
        releases its gametes across the atoll each year."
        .as_bytes();
    let readout = credential::read_sidecar(&manifest, unrelated).unwrap();
    assert!(matches!(readout.verdict, Verdict::NoMatch { .. }), "got {:?}", readout.verdict);
    assert!(!readout.valid);
}

#[test]
fn a_self_signed_credential_is_authentic_but_untrusted_and_identity_unverified() {
    // Decision 6: the local default must report honestly — the signature is valid
    // (authentic, tamper-evident) but the signer is neither trust-listed nor
    // identity-verified, and there's no timestamp unless one was requested.
    let file = ESSAY.as_bytes();
    let mut record = sample_record();
    record.document_binding.final_text_sha256 = sha256_hex(file);
    let manifest = credential::issue_sidecar(&record, file).unwrap();

    let trust = credential::read_sidecar(&manifest, file).unwrap().trust;
    assert!(trust.signed, "self-signed credential is still authentically signed");
    assert!(!trust.trusted, "self-signed must not claim trust-list trust");
    assert!(!trust.identity_verified, "no identity is verified by a local tool");
    assert!(trust.timestamp.is_none(), "no RFC 3161 timestamp without an opt-in TSA");
}

#[test]
fn a_docx_export_of_the_same_writing_verifies_cross_format() {
    // The full Increment-2 path: a credential is sealed to the plain text, the
    // reader drops in a .docx of the same writing, and verification extracts the
    // OOXML text and lands on a content match — not INVALID.
    let original = ESSAY.as_bytes();
    let mut record = sample_record();
    record.document_binding.final_text_sha256 = sha256_hex(original);
    let manifest = credential::issue_sidecar(&record, original).unwrap();

    let docx = minimal_docx(ESSAY);
    let text = extract_named("essay.docx", &docx).expect("extract docx text");
    let readout = credential::read_sidecar_with_text(&manifest, &docx, &text).unwrap();

    assert!(
        matches!(readout.verdict, Verdict::SameContent { .. } | Verdict::SameWriting { .. }),
        "a .docx of the same writing should verify as a content match, got {:?}",
        readout.verdict
    );
    assert!(readout.valid);
}

#[test]
fn process_record_round_trips_through_a_c2pa_manifest() {
    let record = sample_record();
    let signed = credential::issue_ephemeral(&record, "image/png", &small_png())
        .expect("issue c2pa credential");

    let readout = credential::read("image/png", &signed).expect("read c2pa credential");
    assert!(readout.valid, "signature should validate");
    assert_eq!(readout.record, record, "process record must round-trip");
}

#[test]
fn tampering_with_the_signed_asset_breaks_validation() {
    let record = sample_record();
    let mut signed = credential::issue_ephemeral(&record, "image/png", &small_png()).unwrap();

    // Corrupt the IHDR region (bytes just after the 8-byte PNG signature). It is
    // part of the asset covered by the C2PA data hash, not the excluded manifest
    // chunk c2pa inserts after IHDR — so this must break validation.
    for byte in &mut signed[16..24] {
        *byte ^= 0xff;
    }

    match credential::read("image/png", &signed) {
        Ok(readout) => assert!(!readout.valid, "tampered asset must not validate"),
        Err(_) => { /* unreadable after tamper is also acceptable */ }
    }
}
