//! humanshipd credential core.
//!
//! All credential logic lives here so every frontend (native messaging host,
//! WASM verify page, future capture adapters) reuses byte-identical behavior.
//! See `docs/superpowers/specs/2026-06-05-human-authorship-attestation-design.md`.

pub mod badge;
pub mod canonical;
pub mod error;
pub mod record;
pub mod session;
pub mod signing;
pub mod timestamp;
pub mod verify;

pub use badge::{anchor_badge, sign_record, verify_badge_signature, Badge, Integrity};
pub use error::CoreError;
pub use record::{
    BurstStats, DocumentBinding, EvidenceFlags, PauseStats, ProcessStats, Replay, RevisionStats,
    Surface, UnkeyedInsertion, WritingSessionRecord, SCHEMA,
};
pub use session::{build_record, EditEvent, SessionInput, LARGE_UNKEYED_THRESHOLD};
pub use signing::{verify_signature, KeyPair};
pub use timestamp::{verify_timestamp, LocalTsa, TimestampAuthority, TimestampToken};
pub use verify::{verify_badge, VerifyResult};
