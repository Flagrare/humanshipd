# Changelog

All notable changes to humanshipd are recorded here. The project is a
research-grade preview; versions are milestones, not releases on crates.io.

## [0.1.0] — 2026-06-06 — MVP

The first end-to-end slice: capture a writing process, issue a signed credential,
and verify it in the browser with an honest, content-free report.

### Added

- **Per-span provenance + banded report.** The record now carries an ordered list
  of how text entered (typed vs. pasted), and the verify page renders word-count
  proportions plus a four-tier summary (fully typed / typed-with-pastes /
  mostly-pasted / unverified). An "unknown" band accounts for any final-document
  text the tool never saw typed or pasted, so writing outside coverage is shown
  honestly rather than counted as human.
- **Writing fingerprint.** A content-free timeline (edit position over time, or
  cumulative length where position is unknown) rendered as an SVG where a paste is
  a vertical jump and a revisit dips back, with a scrubbable replay (play/pause,
  speed) and jump-to-paste markers.
- **Per-edit caret offsets** captured through the pipeline — `selectionStart` in
  the extension, text prefix-diff in the macOS capturer — enabling the true
  position-over-time fingerprint.
- **Process-shape corroboration (positive-only).** A weak, secondary signal that
  can affirm a human-like drafting rhythm but never claims text "is AI"; its
  absence is explicitly not evidence of AI. Carries its own error band.
- **Self-asserted author name.** An optional, signed (tamper-evident) but
  unverified name, captured in the extension popup and shown on the verify page as
  "not independently verified."
- **Shareable report.** One-click "Save as PDF / print" on a valid verification.
- The verify report, fingerprint, process-shape, and author all appear only when
  the credential is valid for the document on screen.

### Changed

- C2PA `digitalSourceType` corrected from `digitalCapture` (a camera term) to
  `digitalCreation` (human, non-generative) for text.
- Record schema advanced to `@0.4` (spans, timeline, per-edit offsets).

### Security

- The verify page renders all credential-derived values with `textContent` only,
  never `innerHTML`.

### Deliberately out of scope (and why)

- **Verified author identity** — a local-only tool cannot attest who someone is;
  real verification needs an external identity authority (CAWG identity assertion).
- **Per-contributor attribution** and a **paste-source citation helper** — both
  need capture-layer data we don't record yet (per-editor identity; paste origin,
  which browsers don't expose).
