use crate::badge::{verify_badge_signature, Badge};
use crate::error::CoreError;
use crate::timestamp::verify_timestamp;
use serde::{Deserialize, Serialize};

/// The outcome of verifying a badge, plus an honest, non-overclaiming claim string.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VerifyResult {
    pub signature_valid: bool,
    pub timestamp_present: bool,
    pub timestamp_valid: bool,
    pub authority: Option<String>,
    pub gen_time: Option<String>,
    pub document_sha256: String,
    pub char_count: u64,
    pub large_unkeyed_insertions: u64,
    /// Human-readable claim — deliberately nuanced; never "100% human".
    pub claim: String,
}

/// Verify a badge end-to-end: signature integrity, time-anchor, and the
/// metadata-derived evidence flags, then render the honest claim.
pub fn verify_badge(badge: &Badge) -> Result<VerifyResult, CoreError> {
    let signature_valid = verify_badge_signature(badge)?;

    let (timestamp_present, timestamp_valid, authority, gen_time) = match &badge.integrity.timestamp
    {
        Some(token) => (
            true,
            verify_timestamp(token, &badge.integrity.record_sha256)?,
            Some(token.authority.clone()),
            Some(token.gen_time.clone()),
        ),
        None => (false, false, None, None),
    };

    let large_unkeyed_insertions = badge.record.evidence_flags.large_unkeyed_insertions;
    let claim = build_claim(
        signature_valid,
        timestamp_valid,
        &authority,
        &gen_time,
        large_unkeyed_insertions,
    );

    Ok(VerifyResult {
        signature_valid,
        timestamp_present,
        timestamp_valid,
        authority,
        gen_time,
        document_sha256: badge.record.document_binding.final_text_sha256.clone(),
        char_count: badge.record.document_binding.char_count,
        large_unkeyed_insertions,
        claim,
    })
}

/// Build the honest claim string. Never asserts a human authored the ideas.
fn build_claim(
    signature_valid: bool,
    timestamp_valid: bool,
    authority: &Option<String>,
    gen_time: &Option<String>,
    large_unkeyed_insertions: u64,
) -> String {
    if !signature_valid {
        return "INVALID — this badge has been altered since it was issued.".to_string();
    }

    let mut parts = vec![
        "This record was produced by this client and has not been altered since issuance."
            .to_string(),
    ];

    if timestamp_valid {
        let auth = authority.as_deref().unwrap_or("an authority");
        let when = gen_time.as_deref().unwrap_or("an unspecified time");
        parts.push(format!("It existed by {when} (time-anchored by {auth})."));
    } else {
        parts.push("It is not time-anchored.".to_string());
    }

    if large_unkeyed_insertions > 0 {
        parts.push(format!(
            "WARNING: {large_unkeyed_insertions} large insertion(s) appeared without typing — possible paste of AI-generated text."
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
