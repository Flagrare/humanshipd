use humanshipd_core::canonical::sha256_hex;
use humanshipd_core::credential;
use image::{ImageFormat, RgbaImage};
use std::io::Cursor;

mod common;
use common::sample_record;

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
