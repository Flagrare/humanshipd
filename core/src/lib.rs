//! humanshipd credential core.
//!
//! All credential logic lives here so every frontend (native messaging host,
//! capture adapters, verify page) reuses byte-identical behavior. The credential
//! is a C2PA manifest (via c2pa-rs); the process record is a custom assertion.
//! See `docs/superpowers/specs/2026-06-05-human-authorship-attestation-design.md`.

pub mod canonical;
pub mod credential;
pub mod error;
pub mod fingerprint;
pub mod record;
pub mod report;
pub mod session;
pub mod text_embed;

pub use credential::{
    issue_ephemeral, issue_sidecar, issue_sidecar_with_author, read, read_sidecar,
    CredentialReadout, PROCESS_ASSERTION,
};
pub use error::CoreError;
pub use record::{
    BurstStats, DocumentBinding, EvidenceFlags, PauseStats, ProcessStats, Provenance,
    ProvenanceSpan, Replay, RevisionStats, Surface, TimelinePoint, UnkeyedInsertion,
    WritingSessionRecord, SCHEMA,
};
pub use report::{
    render_process_shape, render_report, NuanceSummary, ProcessAssessment, ProcessShape,
    ProvenanceReport, ReportBand,
};
pub use fingerprint::{text_iscc, text_soft_binding, SoftBinding, ISCC_ALG};
pub use session::{build_record, EditEvent, SessionInput, LARGE_UNKEYED_THRESHOLD};
pub use text_embed::{embed, extract, strip};
