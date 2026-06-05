//! humanshipd credential core.
//!
//! All credential logic lives here so every frontend (native messaging host,
//! WASM verify page, future capture adapters) reuses byte-identical behavior.
//! See `docs/superpowers/specs/2026-06-05-human-authorship-attestation-design.md`.

pub mod canonical;
pub mod error;
pub mod record;

pub use error::CoreError;
pub use record::{
    BurstStats, DocumentBinding, EvidenceFlags, PauseStats, ProcessStats, Replay, RevisionStats,
    Surface, UnkeyedInsertion, WritingSessionRecord, SCHEMA,
};
