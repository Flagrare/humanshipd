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
