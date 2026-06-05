use humanshipd_core::badge::{sign_record, verify_badge_signature};
use humanshipd_core::canonical::canonical_sha256;
use humanshipd_core::signing::KeyPair;

mod common;
use common::sample_record;

fn keypair() -> KeyPair {
    KeyPair::from_seed(&[7u8; 32])
}

#[test]
fn a_freshly_signed_badge_verifies() {
    let badge = sign_record(sample_record(), &keypair()).unwrap();
    assert!(verify_badge_signature(&badge).unwrap());
}

#[test]
fn tampering_with_the_record_fails_verification() {
    let mut badge = sign_record(sample_record(), &keypair()).unwrap();
    badge.record.document_binding.char_count += 1;
    assert!(!verify_badge_signature(&badge).unwrap());
}

#[test]
fn tampering_with_the_signature_fails_verification() {
    let mut badge = sign_record(sample_record(), &keypair()).unwrap();
    let first = &badge.integrity.client_signature[0..1];
    let replacement = if first == "0" { "1" } else { "0" };
    badge.integrity.client_signature.replace_range(0..1, replacement);
    assert!(!verify_badge_signature(&badge).unwrap());
}

#[test]
fn recomputing_the_hash_for_an_altered_record_still_fails() {
    // An attacker alters the record AND updates record_sha256 to match — but the
    // signature was made over the original canonical bytes, so it still fails.
    let mut badge = sign_record(sample_record(), &keypair()).unwrap();
    badge.record.document_binding.char_count += 1;
    badge.integrity.record_sha256 = canonical_sha256(&badge.record).unwrap();
    assert!(!verify_badge_signature(&badge).unwrap());
}
