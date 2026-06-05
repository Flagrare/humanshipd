use humanshipd_core::badge::{anchor_badge, sign_record};
use humanshipd_core::signing::KeyPair;
use humanshipd_core::timestamp::{verify_timestamp, LocalTsa};

mod common;
use common::sample_record;

fn client_key() -> KeyPair {
    KeyPair::from_seed(&[7u8; 32])
}

fn local_tsa() -> LocalTsa {
    LocalTsa::new(&[9u8; 32], "humanshipd:local-poc", "2026-06-05T12:00:00Z")
}

#[test]
fn anchoring_attaches_a_token_over_the_record_hash() {
    let badge = sign_record(sample_record(), &client_key()).unwrap();
    let anchored = anchor_badge(badge, &local_tsa()).unwrap();
    let token = anchored.integrity.timestamp.expect("token attached");
    assert_eq!(token.message_imprint_sha256, anchored.integrity.record_sha256);
    assert_eq!(token.gen_time, "2026-06-05T12:00:00Z");
}

#[test]
fn a_valid_token_verifies_against_the_record_hash() {
    let badge = sign_record(sample_record(), &client_key()).unwrap();
    let anchored = anchor_badge(badge, &local_tsa()).unwrap();
    let token = anchored.integrity.timestamp.unwrap();
    assert!(verify_timestamp(&token, &anchored.integrity.record_sha256).unwrap());
}

#[test]
fn a_token_for_a_different_hash_is_rejected() {
    let badge = sign_record(sample_record(), &client_key()).unwrap();
    let anchored = anchor_badge(badge, &local_tsa()).unwrap();
    let token = anchored.integrity.timestamp.unwrap();
    assert!(!verify_timestamp(&token, &"b".repeat(64)).unwrap());
}

#[test]
fn tampering_with_the_token_time_fails_verification() {
    let badge = sign_record(sample_record(), &client_key()).unwrap();
    let anchored = anchor_badge(badge, &local_tsa()).unwrap();
    let mut token = anchored.integrity.timestamp.unwrap();
    let imprint = anchored.integrity.record_sha256.clone();
    token.gen_time = "2030-01-01T00:00:00Z".to_string();
    assert!(!verify_timestamp(&token, &imprint).unwrap());
}
