# Changelog

All notable changes to humanshipd are recorded here. The project is a
research-grade preview; versions are milestones, not releases on crates.io.

## [0.2.0] — 2026-06-08 — Cross-format verification

Verification stops being all-or-nothing. A credential sealed to your writing now
verifies even when the document has been reformatted, lightly edited, or saved to a
different format — and says, honestly, how close the match is.

### Added

- **Tiered match verdict.** Checking a credential against a document now reports
  one of *exact file*, *same content* (reformatted / lightly edited), *same
  writing* (heavily edited), *borderline — needs review*, or *no match*, with the
  measured content distance and the published threshold. A genuine credential
  checked against a reformatted copy reads as a content match instead of a flat
  "invalid"; only a broken or forged signature reads as invalid.
- **Cross-format content matching.** Identity rides a 256-bit ISCC content code
  (ISO 24138) scored by normalized Hamming distance, calibrated on a real corpus —
  genuine multi-revision edits stay within ~0.12, unrelated writing sits above
  ~0.44, a wide gap rather than a knife-edge.
- **Reads text out of common formats before matching.** `.txt` and `.docx` work
  everywhere — the in-browser validator unzips Word files in-page — and `.pdf`
  works with the command-line tool. So a `.docx` export of the same writing now
  verifies, and a PDF in the browser is pointed to the CLI rather than failing
  opaquely.
- **Google Docs capture.** A parser turns a Google Docs revision log into a
  credential, extending capture beyond native apps and ordinary web editors.

### Changed

- ISCC content code raised 64 → 256 bits, so edits grade smoothly instead of
  saturating the distance.
- The verify page shows the matched tier and distance, with a distinct amber
  "borderline — needs review" state.

### Fixed

- A document-extraction failure (for example a PDF in the browser) is no longer
  mislabeled "Could not read credential" — the message now names its real cause.

### Public API (`humanshipd-core`)

- `read_sidecar_with_text(manifest, file_bytes, text)` — verify against text
  extracted from a container while keeping the byte-exact hard binding on the raw
  bytes.
- New `formats` module: `DocFormat`, `extract_text`, `extract_named`.
- `Verdict` enum, plus `iscc_distance` / `classify` and the locked band thresholds
  (`BAND_SAME_CONTENT_MAX` ≤ 0.05, `BAND_SAME_WRITING_MAX` ≤ 0.20,
  `BAND_BORDERLINE_MAX` ≤ 0.35).
- `CoreError::Format` for extraction failures.

### Still out of scope

- Precise extraction of headers/footers and tracked changes; in-browser PDF; and
  `.rtf`. Verified author identity remains future work (see 0.1.0).

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
