# Cross-session continuity & accumulation (Slice 1) — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Persist the browser extension's writing capture per-document across sessions so a credential covers the *whole* witnessed document, not just the current page-load — and move op-replay into the Rust core so it's a single source of truth.

**Architecture:** A new serializable `CaptureLog` in core (per-document, append-only `CaptureSession`s of normalized `CapturedOp`s) owns replay + cross-session accumulation; `build_record` reconstructs the text and aggregates content-free stats, declining when an op lands beyond the witnessed buffer. The browser adapter (`gdocs.js`) captures normalized ops + paste flags and persists the log to sharded `chrome.storage.local` keyed by the Docs URL id; at issue, the popup hands the whole log to the WASM core, which builds the record and signs in the existing worker.

**Tech Stack:** Rust (core), `serde`/`serde_json`, `wasm-bindgen` (web-verify), MV3 extension JS, Playwright.

Spec: `docs/superpowers/specs/2026-06-09-continuity-accumulation-design.md`. Out of scope: native identity (Slice 2), `/revisions/load` backfill (Slice 3).

---

## File structure

- **Create** `core/src/capture_log.rs` — `DocumentIdentity`, `CapturedOp`, `CaptureSession`, `CaptureLog`, `LogError`, replay + `build_record`.
- **Modify** `core/src/session.rs` — make `pauses_and_bursts`, `build_spans`, `build_timeline` `pub(crate)`; add a multi-session timeline merge helper.
- **Modify** `core/src/record.rs` — add `session_count`, `first_capture_at_ms`, `last_capture_at_ms`; bump `SCHEMA` to `@0.5`.
- **Modify** `core/src/lib.rs` — register module + re-exports.
- **Modify** `core/tests/common/mod.rs` — update `sample_record()` for the new fields.
- **Modify** `web-verify/src/lib.rs` — add `issue_from_capture_log(log_json) -> Result<Vec<u8>, JsValue>`.
- **Modify** `web-verify/verify.html` + `web-verify/tests/verify.spec.js` — expose + test it.
- **Modify** `extension/gdocs.js` — capture normalized ops, persist sharded log, resume, return the log on `getSession`.
- **Modify** `extension/popup.js` — issue from the log; handle decline.
- **Modify** `extension/manifest.json` — add `storage` + `unlimitedStorage`.
- **Create** `extension/tests/continuity.spec.js` — capture → shimmed storage → reload → resume → accumulate.

---

## Task 1: Core — `CaptureLog` types + serde

**Files:**
- Create: `core/src/capture_log.rs`
- Modify: `core/src/lib.rs`
- Test: `core/tests/capture_log.rs` (create)

- [ ] **Step 1: Write the failing test**

Create `core/tests/capture_log.rs`:

```rust
use humanshipd_core::capture_log::{CaptureLog, CaptureSession, CapturedOp, DocumentIdentity, LOG_SCHEMA};

fn session(id: &str, started: u64, ops: Vec<CapturedOp>) -> CaptureSession {
    CaptureSession {
        session_id: id.into(),
        surface_kind: "gdocs".into(),
        surface_app: "docs.google.com".into(),
        started_at_ms: started,
        ops,
    }
}

#[test]
fn capture_log_round_trips_through_json() {
    let mut log = CaptureLog::new(DocumentIdentity { kind: "gdocs".into(), id: "doc-1".into() });
    log.append(session("s1", 1000, vec![
        CapturedOp::Insert { at_ms: 0, pos: 0, text: "Hello".into(), pasted: false },
        CapturedOp::Delete { at_ms: 50, pos: 4, len: 1 },
    ]));
    assert_eq!(log.schema, LOG_SCHEMA);
    let json = serde_json::to_string(&log).unwrap();
    let back: CaptureLog = serde_json::from_str(&json).unwrap();
    assert_eq!(back, log);
    assert_eq!(back.sessions.len(), 1);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p humanshipd-core --test capture_log`
Expected: FAIL — `unresolved import humanshipd_core::capture_log`.

- [ ] **Step 3: Write minimal implementation**

Create `core/src/capture_log.rs`:

```rust
//! Per-document, append-only capture log (Slice 1 of cross-session continuity).
//!
//! A `CaptureLog` accumulates a document's writing across sessions as normalized,
//! surface-agnostic ops. Core owns replay + accumulation so there is one source of
//! truth (native + WASM); adapters only capture ops and persist the serialized log.
//! The log holds the inserted *text* (needed to reconstruct the document) and is
//! local-only working state — it never enters the content-free record/credential.

use serde::{Deserialize, Serialize};

/// Versioned log schema identifier.
pub const LOG_SCHEMA: &str = "authorshipped/log@1";

/// How "this document" is keyed across sessions. Slice 1: a Google Docs URL id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DocumentIdentity {
    /// Surface family, e.g. `"gdocs"` (Slice 2 adds `"native"`).
    pub kind: String,
    /// Stable per-surface id (the Docs URL document id).
    pub id: String,
}

/// One normalized edit. `at_ms` is relative to its session's start.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum CapturedOp {
    /// Insert `text` at character offset `pos`. `pasted` ⇒ arrived without typing.
    Insert { at_ms: u64, pos: u64, text: String, pasted: bool },
    /// Delete `len` characters starting at offset `pos`.
    Delete { at_ms: u64, pos: u64, len: u64 },
}

/// One contiguous writing session (one page-load / app run).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaptureSession {
    pub session_id: String,
    pub surface_kind: String,
    pub surface_app: String,
    /// Absolute epoch milliseconds at session start (for cross-session ordering).
    pub started_at_ms: u64,
    pub ops: Vec<CapturedOp>,
}

/// The accumulated, persisted capture for one document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaptureLog {
    pub schema: String,
    pub identity: DocumentIdentity,
    pub sessions: Vec<CaptureSession>,
}

impl CaptureLog {
    /// A fresh log for `identity`, with no sessions yet.
    pub fn new(identity: DocumentIdentity) -> Self {
        CaptureLog { schema: LOG_SCHEMA.to_string(), identity, sessions: Vec::new() }
    }

    /// Append a captured session.
    pub fn append(&mut self, session: CaptureSession) {
        self.sessions.push(session);
    }
}
```

Add to `core/src/lib.rs` after `pub mod canonical;`:

```rust
pub mod capture_log;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p humanshipd-core --test capture_log`
Expected: PASS (1 test).

- [ ] **Step 5: Commit**

```bash
git add core/src/capture_log.rs core/src/lib.rs core/tests/capture_log.rs
git commit -m "✨ feat(core): CaptureLog types for cross-session accumulation"
```

---

## Task 2: Core — replay + `Unwitnessed` decline

**Files:**
- Modify: `core/src/capture_log.rs`
- Test: `core/tests/capture_log.rs`

- [ ] **Step 1: Write the failing test**

Append to `core/tests/capture_log.rs`:

```rust
use humanshipd_core::capture_log::LogError;

#[test]
fn reconstructs_text_across_two_sessions() {
    let mut log = CaptureLog::new(DocumentIdentity { kind: "gdocs".into(), id: "d".into() });
    log.append(session("s1", 1000, vec![
        CapturedOp::Insert { at_ms: 0, pos: 0, text: "Hello".into(), pasted: false },
    ]));
    log.append(session("s2", 90_000, vec![
        CapturedOp::Insert { at_ms: 0, pos: 5, text: " world".into(), pasted: false },
    ]));
    assert_eq!(log.reconstruct_text().unwrap(), "Hello world");
}

#[test]
fn declines_when_an_op_lands_beyond_the_witnessed_buffer() {
    let mut log = CaptureLog::new(DocumentIdentity { kind: "gdocs".into(), id: "d".into() });
    // First op inserts at pos 5 of an empty doc → pre-existing content we never saw.
    log.append(session("s1", 1000, vec![
        CapturedOp::Insert { at_ms: 0, pos: 5, text: "x".into(), pasted: false },
    ]));
    assert!(matches!(log.reconstruct_text(), Err(LogError::Unwitnessed { .. })));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p humanshipd-core --test capture_log`
Expected: FAIL — no method `reconstruct_text`, no `LogError`.

- [ ] **Step 3: Write minimal implementation**

Append to `core/src/capture_log.rs`:

```rust
/// Why a log could not be turned into a credential.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LogError {
    /// An op references a position the log never witnessed being written — the
    /// document had pre-existing content, or was edited outside our capture.
    Unwitnessed { pos: u64, buffer_len: u64 },
}

impl std::fmt::Display for LogError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogError::Unwitnessed { pos, buffer_len } => write!(
                f,
                "edit at position {pos} is beyond the {buffer_len} characters humanshipd witnessed — this document wasn't captured from the start"
            ),
        }
    }
}
impl std::error::Error for LogError {}

impl CaptureLog {
    /// Replay every session's ops into the reconstructed document text. Errors with
    /// [`LogError::Unwitnessed`] when an op falls beyond the witnessed buffer.
    pub fn reconstruct_text(&self) -> Result<String, LogError> {
        Ok(self.reconstruct_buffer()?.into_iter().collect())
    }

    fn reconstruct_buffer(&self) -> Result<Vec<char>, LogError> {
        let mut buf: Vec<char> = Vec::new();
        for session in &self.sessions {
            for op in &session.ops {
                match op {
                    CapturedOp::Insert { pos, text, .. } => {
                        let pos = *pos as usize;
                        if pos > buf.len() {
                            return Err(LogError::Unwitnessed {
                                pos: *pos,
                                buffer_len: buf.len() as u64,
                            });
                        }
                        buf.splice(pos..pos, text.chars());
                    }
                    CapturedOp::Delete { pos, len, .. } => {
                        let start = *pos as usize;
                        let end = start + *len as usize;
                        if end > buf.len() {
                            return Err(LogError::Unwitnessed {
                                pos: *pos,
                                buffer_len: buf.len() as u64,
                            });
                        }
                        buf.drain(start..end);
                    }
                }
            }
        }
        Ok(buf)
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p humanshipd-core --test capture_log`
Expected: PASS (3 tests).

- [ ] **Step 5: Commit**

```bash
git add core/src/capture_log.rs core/tests/capture_log.rs
git commit -m "✨ feat(core): CaptureLog replay with unwitnessed-content decline"
```

---

## Task 3: Core — record schema gains session summary

**Files:**
- Modify: `core/src/record.rs`
- Modify: `core/tests/common/mod.rs`
- Test: existing suites must still pass.

- [ ] **Step 1: Write the failing test**

Append to `core/tests/capture_log.rs`:

```rust
#[test]
fn build_record_reports_two_sessions_and_aggregates_active_time() {
    let mut log = CaptureLog::new(DocumentIdentity { kind: "gdocs".into(), id: "d".into() });
    // Session 1: 2s of writing.
    log.append(session("s1", 1_000, vec![
        CapturedOp::Insert { at_ms: 0,    pos: 0, text: "Hello".into(), pasted: false },
        CapturedOp::Insert { at_ms: 2_000, pos: 5, text: "!".into(),     pasted: false },
    ]));
    // Session 2 a day later: 1s of writing.
    log.append(session("s2", 90_000_000, vec![
        CapturedOp::Insert { at_ms: 0,    pos: 6, text: " more".into(), pasted: false },
        CapturedOp::Insert { at_ms: 1_000, pos: 11, text: ".".into(),    pasted: false },
    ]));
    let record = log.build_record().unwrap();
    assert_eq!(record.session_count, 2);
    // Active time is the SUM of per-session spans (2s + 1s), not wall-clock (a day).
    assert_eq!(record.process.active_ms, 3_000);
    assert_eq!(record.document_binding.char_count, "Hello! more.".chars().count() as u64);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p humanshipd-core --test capture_log`
Expected: FAIL — no `session_count` field, no `build_record` method.

- [ ] **Step 3a: Add the schema fields**

In `core/src/record.rs`, bump the schema and add fields to `WritingSessionRecord`:

```rust
/// The current record schema identifier (spec §5).
pub const SCHEMA: &str = "authorshipped/record@0.5";
```

Add to the `WritingSessionRecord` struct (after `pub replay: Replay,`):

```rust
    /// How many distinct writing sessions this record accumulates (≥ 1).
    #[serde(default = "one")]
    pub session_count: u64,
    /// Absolute epoch ms of the first and last captured session start (0 when
    /// single-session / unknown). Lets the report say "written over D days."
    #[serde(default)]
    pub first_capture_at_ms: u64,
    #[serde(default)]
    pub last_capture_at_ms: u64,
```

Add this free function at the bottom of `core/src/record.rs`:

```rust
fn one() -> u64 {
    1
}
```

- [ ] **Step 3b: Update single-session `build_record` to set the new fields**

In `core/src/session.rs`, inside `build_record`, in the returned `WritingSessionRecord { … }` literal, add after `replay: Replay { … },`:

```rust
        session_count: 1,
        first_capture_at_ms: 0,
        last_capture_at_ms: 0,
```

- [ ] **Step 3c: Update the test helper**

In `core/tests/common/mod.rs`, in `sample_record()`, add after `replay: Replay { … },`:

```rust
        session_count: 1,
        first_capture_at_ms: 0,
        last_capture_at_ms: 0,
```

- [ ] **Step 3d: Make session helpers reusable + add `CaptureLog::build_record`**

In `core/src/session.rs`, change these three fn signatures from `fn` to `pub(crate) fn`:
`pub(crate) fn pauses_and_bursts`, `pub(crate) fn build_spans`, `pub(crate) fn build_timeline`. Also add, in `session.rs`:

```rust
/// Convert a session's normalized ops into the content-free `EditEvent`s the
/// record aggregation consumes.
pub(crate) fn ops_to_events(ops: &[crate::capture_log::CapturedOp]) -> Vec<EditEvent> {
    use crate::capture_log::CapturedOp;
    ops.iter()
        .map(|op| match op {
            CapturedOp::Insert { at_ms, pos, text, pasted } => {
                let chars = text.chars().count() as u64;
                EditEvent {
                    at_ms: *at_ms,
                    inserted_chars: chars,
                    deleted_chars: 0,
                    keystrokes: if *pasted { 0 } else { chars },
                    at_offset: Some(*pos),
                }
            }
            CapturedOp::Delete { at_ms, pos, len } => EditEvent {
                at_ms: *at_ms,
                inserted_chars: 0,
                deleted_chars: *len,
                keystrokes: *len,
                at_offset: Some(*pos),
            },
        })
        .collect()
}
```

In `core/src/capture_log.rs`, add:

```rust
use crate::canonical::sha256_hex;
use crate::record::*;
use crate::session::{build_spans, build_timeline, ops_to_events, pauses_and_bursts};

impl CaptureLog {
    /// Build a content-free record over **all** sessions. Aggregates counts; active
    /// time is the sum of per-session writing spans; cross-session gaps are session
    /// boundaries, not pauses. Declines with [`LogError`] on unwitnessed content.
    pub fn build_record(&self) -> Result<WritingSessionRecord, LogError> {
        let text: String = self.reconstruct_buffer()?.into_iter().collect();

        let per_session: Vec<Vec<EditEvent>> =
            self.sessions.iter().map(|s| ops_to_events(&s.ops)).collect();
        let all: Vec<EditEvent> = per_session.iter().flatten().cloned().collect();

        let keystrokes: u64 = all.iter().map(|e| e.keystrokes).sum();
        let total_inserted: u64 = all.iter().map(|e| e.inserted_chars).sum();
        let keyed_inserted: u64 =
            all.iter().filter(|e| e.keystrokes > 0).map(|e| e.inserted_chars).sum();
        let keyed_fraction = if total_inserted > 0 {
            keyed_inserted as f64 / total_inserted as f64
        } else {
            1.0
        };
        let insertions_without_keystrokes: Vec<UnkeyedInsertion> = all
            .iter()
            .filter(|e| e.inserted_chars > 0 && e.keystrokes == 0)
            .map(|e| UnkeyedInsertion { size: e.inserted_chars })
            .collect();
        let large_unkeyed_insertions = insertions_without_keystrokes
            .iter()
            .filter(|u| u.size >= crate::session::LARGE_UNKEYED_THRESHOLD)
            .count() as u64;
        let revisions = RevisionStats {
            insertions: all.iter().filter(|e| e.inserted_chars > 0).count() as u64,
            deletions: all.iter().filter(|e| e.deleted_chars > 0).count() as u64,
            reformulations: 0,
        };

        // Per-session active span + pauses/bursts, combined; cross-session gaps excluded.
        let mut active_ms = 0u64;
        let mut gt_2s = 0u64;
        let mut burst_count = 0u64;
        let mut burst_total = 0f64;
        let mut spans: Vec<ProvenanceSpan> = Vec::new();
        let mut timeline: Vec<TimelinePoint> = Vec::new();
        let mut clock = 0u64; // continuous writing-time offset across sessions
        for events in &per_session {
            if let (Some(first), Some(last)) = (events.first(), events.last()) {
                active_ms += last.at_ms.saturating_sub(first.at_ms);
            }
            let (p, b) = pauses_and_bursts(events);
            gt_2s += p.gt_2s;
            burst_count += b.count;
            burst_total += b.mean_len * b.count as f64;
            spans.extend(build_spans(events));
            for mut pt in build_timeline(events) {
                pt.at_ms += clock; // lay sessions on a continuous axis (no day-gaps)
                timeline.push(pt);
            }
            if let Some(last) = events.last() {
                clock += last.at_ms + 1; // +1 marks the session boundary
            }
        }
        let pauses = PauseStats { gt_2s, buckets: Vec::new() };
        let bursts = BurstStats {
            count: burst_count,
            mean_len: if burst_count > 0 { burst_total / burst_count as f64 } else { 0.0 },
            buckets: Vec::new(),
        };

        let surface = self
            .sessions
            .last()
            .map(|s| Surface { kind: s.surface_kind.clone(), app: s.surface_app.clone() })
            .unwrap_or(Surface { kind: self.identity.kind.clone(), app: String::new() });
        let session_id = self
            .sessions
            .last()
            .map(|s| s.session_id.clone())
            .unwrap_or_default();

        Ok(WritingSessionRecord {
            schema: SCHEMA.to_string(),
            session_id,
            surface,
            document_binding: DocumentBinding {
                final_text_sha256: sha256_hex(text.as_bytes()),
                char_count: text.chars().count() as u64,
            },
            process: ProcessStats {
                active_ms,
                keystrokes,
                bursts,
                pauses,
                revisions,
                insertions_without_keystrokes,
                keyed_fraction,
                spans,
                timeline,
            },
            evidence_flags: EvidenceFlags { large_unkeyed_insertions },
            replay: Replay { available: false, log_sha256: None },
            session_count: self.sessions.len() as u64,
            first_capture_at_ms: self.sessions.first().map(|s| s.started_at_ms).unwrap_or(0),
            last_capture_at_ms: self.sessions.last().map(|s| s.started_at_ms).unwrap_or(0),
        })
    }
}
```

Make `LARGE_UNKEYED_THRESHOLD` reachable: it is already `pub const` in `session.rs`, so `crate::session::LARGE_UNKEYED_THRESHOLD` resolves.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p humanshipd-core`
Expected: PASS — `capture_log` (4 tests) plus all existing suites green (record/credential/etc. compile with the new fields).

- [ ] **Step 5: Commit**

```bash
git add core/src/capture_log.rs core/src/session.rs core/src/record.rs core/tests/common/mod.rs core/tests/capture_log.rs
git commit -m "✨ feat(core): build_record over a multi-session CaptureLog"
```

---

## Task 4: Core — re-exports

**Files:**
- Modify: `core/src/lib.rs`

- [ ] **Step 1: Add re-exports**

In `core/src/lib.rs`, after the `pub use credential::{…}` block, add:

```rust
pub use capture_log::{CaptureLog, CaptureSession, CapturedOp, DocumentIdentity, LogError, LOG_SCHEMA};
```

- [ ] **Step 2: Run the gate**

Run: `cargo test -p humanshipd-core && cargo clippy -p humanshipd-core --all-targets -- -D warnings`
Expected: PASS, no warnings.

- [ ] **Step 3: Commit**

```bash
git add core/src/lib.rs
git commit -m "✨ feat(core): re-export CaptureLog API"
```

---

## Task 5: WASM — issue from a capture log

**Files:**
- Modify: `web-verify/src/lib.rs`
- Modify: `web-verify/verify.html`
- Test: `web-verify/tests/verify.spec.js`

- [ ] **Step 1: Write the failing test**

Append to `web-verify/tests/verify.spec.js` (after the existing in-browser issuance test):

```javascript
test("issues a credential from a multi-session capture log (WASM)", async ({ page }) => {
  await page.goto("/verify.html");
  await expect(page.getByRole("button", { name: "Verify" })).toBeEnabled();
  const out = await page.evaluate(async () => {
    const log = {
      schema: "authorshipped/log@1",
      identity: { kind: "gdocs", id: "doc-x" },
      sessions: [
        { session_id: "s1", surface_kind: "gdocs", surface_app: "docs.google.com", started_at_ms: 1000,
          ops: [{ op: "insert", at_ms: 0, pos: 0, text: "Hello", pasted: false }] },
        { session_id: "s2", surface_kind: "gdocs", surface_app: "docs.google.com", started_at_ms: 90000000,
          ops: [{ op: "insert", at_ms: 0, pos: 5, text: " world", pasted: false }] },
      ],
    };
    const manifest = window.issue_from_capture_log(JSON.stringify(log));
    const doc = new TextEncoder().encode("Hello world");
    const r = window.verify_credential_named(manifest, doc, "doc.txt");
    return { valid: r.valid, tier: r.verdict && r.verdict.tier };
  });
  expect(out.valid).toBe(true);
  expect(out.tier).toBe("exact_file");
});

test("declines issuance for an unwitnessed (pre-existing) document", async ({ page }) => {
  await page.goto("/verify.html");
  await expect(page.getByRole("button", { name: "Verify" })).toBeEnabled();
  const err = await page.evaluate(async () => {
    const log = {
      schema: "authorshipped/log@1",
      identity: { kind: "gdocs", id: "doc-x" },
      sessions: [{ session_id: "s1", surface_kind: "gdocs", surface_app: "docs.google.com", started_at_ms: 1000,
        ops: [{ op: "insert", at_ms: 0, pos: 5, text: "x", pasted: false }] }],
    };
    try { window.issue_from_capture_log(JSON.stringify(log)); return null; }
    catch (e) { return String(e.message || e); }
  });
  expect(err).toContain("wasn't captured from the start");
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd web-verify/tests && npx playwright test -g "capture log"`
Expected: FAIL — `window.issue_from_capture_log is not a function`.

- [ ] **Step 3: Implement the WASM function**

In `web-verify/src/lib.rs`, add the import to the `humanshipd_core::credential` use line:
`issue_sidecar_with_author` is already imported. Add a new import line:

```rust
use humanshipd_core::capture_log::CaptureLog;
```

Add both functions (near `issue_credential`). `reconstruct_text_from_log` exists so
the popup can get the bound document text for the zip **without** re-implementing
replay in JS (that would reintroduce the JS↔Rust drift this whole change removes):

```rust
/// Issue a credential from an accumulated capture log (cross-session continuity).
/// Replays + aggregates in core, then signs. Throws if the log references content
/// humanshipd never witnessed (pre-existing/externally-edited document).
#[wasm_bindgen]
pub fn issue_from_capture_log(log_json: &str, author: Option<String>) -> Result<Vec<u8>, JsValue> {
    let log: CaptureLog = serde_json::from_str(log_json)
        .map_err(|e| JsValue::from_str(&format!("bad capture log: {e}")))?;
    let record = log
        .build_record()
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let text = log
        .reconstruct_text()
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    issue_sidecar_with_author(&record, text.as_bytes(), author.as_deref())
        .map_err(|e| JsValue::from_str(&format!("issue failed: {e}")))
}

/// Reconstruct the bound document text from a capture log (single source of truth
/// for replay — used to build the zip's `document.txt`, never to re-derive in JS).
#[wasm_bindgen]
pub fn reconstruct_text_from_log(log_json: &str) -> Result<String, JsValue> {
    let log: CaptureLog = serde_json::from_str(log_json)
        .map_err(|e| JsValue::from_str(&format!("bad capture log: {e}")))?;
    log.reconstruct_text().map_err(|e| JsValue::from_str(&e.to_string()))
}
```

In `web-verify/verify.html`, extend the import and the window exposure:

```html
import init, { verify_credential, verify_credential_named, issue_credential, issue_from_capture_log, reconstruct_text_from_log } from "./pkg/humanshipd_web_verify.js";
```

and after `window.issue_credential = issue_credential;`:

```javascript
      window.issue_from_capture_log = issue_from_capture_log;
      window.reconstruct_text_from_log = reconstruct_text_from_log;
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cd web-verify/tests && npx playwright test -g "capture log"`
Expected: PASS (config rebuilds the wasm; both tests green).

- [ ] **Step 5: Commit**

```bash
git add web-verify/src/lib.rs web-verify/verify.html web-verify/tests/verify.spec.js
git commit -m "✨ feat(web-verify): issue_from_capture_log (multi-session issuance)"
```

---

## Task 6: Extension — capture normalized ops + persist sharded log

**Files:**
- Modify: `extension/gdocs.js`
- Test: covered by Task 8's spec.

This rewires `gdocs.js`: it stops reconstructing text (no `buf`), instead recording normalized `CapturedOp`s, persisting them per-session to `chrome.storage.local`, and resuming prior sessions on load.

- [ ] **Step 1: Replace the reconstruction state with op recording**

In `extension/gdocs.js`, replace the state block and `applyOp` with op-recording. Replace everything from `let startTime = null;` down to the end of `applyOp` (the `switch` function) with:

```javascript
  const isTop = window.top === window;
  const docId = (location.pathname.match(/\/d\/([^/]+)/) || [])[1] || "unknown";
  const KEY_PREFIX = `humanshipd:log:gdocs:${docId}`;

  let priorSessions = []; // sessions loaded from storage (earlier page-loads)
  let sessionIndex = 0; // this page-load's session number
  let startedAtMs = Date.now();
  const ops = []; // this session's normalized CapturedOps
  let pendingPaste = null; // { len, at } awaiting the next insert

  const charsOf = (s) => Array.from(s || "");

  function consumePasteFor(len, at) {
    if (!pendingPaste) return false;
    const recent = at - pendingPaste.at < 4000;
    const sized = pendingPaste.len === 0 || pendingPaste.len === len;
    if (recent && sized) { pendingPaste = null; return true; }
    return false;
  }

  // Translate a Google Docs save op (is/ds/mlti) into normalized CapturedOps and
  // record them. Text reconstruction now lives in core (build_record), not here.
  function recordOp(op, at) {
    switch (op && op.ty) {
      case "is": {
        const ibi = Number.isInteger(op.ibi) ? op.ibi : 1;
        const chars = charsOf(op.s);
        if (chars.length === 0) return;
        const pasted = consumePasteFor(chars.length, at);
        ops.push({ op: "insert", at_ms: Math.max(at - startedAtMs, 0), pos: Math.max(ibi - 1, 0), text: chars.join(""), pasted });
        break;
      }
      case "ds": {
        const si = Number.isInteger(op.si) ? op.si : 1;
        const ei = Number.isInteger(op.ei) ? op.ei : si;
        const len = Math.max(ei - si + 1, 0);
        if (len === 0) return;
        ops.push({ op: "delete", at_ms: Math.max(at - startedAtMs, 0), pos: Math.max(si - 1, 0), len });
        break;
      }
      case "mlti": {
        for (const sub of (op.mts || [])) recordOp(sub, at);
        break;
      }
      default:
        break;
    }
    saveSoon();
  }
```

- [ ] **Step 2: Add persistence (load + debounced sharded save)**

Immediately after the block above, add:

```javascript
  function currentSession() {
    return {
      session_id: `gdocs-${docId}-${sessionIndex}`,
      surface_kind: "gdocs",
      surface_app: "docs.google.com",
      started_at_ms: startedAtMs,
      ops,
    };
  }

  let saveTimer = null;
  function saveSoon() {
    if (saveTimer) return;
    saveTimer = setTimeout(saveNow, 1500);
  }
  function saveNow() {
    saveTimer = null;
    if (ops.length === 0) return;
    chrome.storage.local.set({ [`${KEY_PREFIX}:s${sessionIndex}`]: currentSession() });
  }
  window.addEventListener("pagehide", saveNow, true);
  document.addEventListener("visibilitychange", () => { if (document.visibilityState === "hidden") saveNow(); }, true);

  // On load (top frame only): pull prior sessions, start a new session after them.
  if (isTop) {
    chrome.storage.local.get(null, (all) => {
      const keys = Object.keys(all || {})
        .filter((k) => k.startsWith(`${KEY_PREFIX}:s`))
        .sort((a, b) => Number(a.split(":s")[1]) - Number(b.split(":s")[1]));
      priorSessions = keys.map((k) => all[k]);
      sessionIndex = priorSessions.length; // this page-load is the next session
    });
  }
```

- [ ] **Step 3: Update the message + getSession handlers**

In `extension/gdocs.js`, in the `window.addEventListener("message", …)` handler, replace `if (d.kind === "op" && isTop) applyOp(d.op, d.at);` with:

```javascript
    if (d.kind === "op" && isTop) recordOp(d.op, d.at);
```

Replace the entire `chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => { … })` block with a handler that returns the **accumulated log**:

```javascript
  chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
    if (message?.type !== "getSession") return false;
    const sessions = ops.length > 0 ? [...priorSessions, currentSession()] : [...priorSessions];
    if (sessions.length === 0 || sessions.every((s) => s.ops.length === 0)) return false;
    sendResponse({
      log: { schema: "authorshipped/log@1", identity: { kind: "gdocs", id: docId }, sessions },
      surface_kind: "gdocs",
    });
    return true;
  });
```

- [ ] **Step 4: Verify it loads (syntax)**

Run: `node --check extension/gdocs.js`
Expected: no output (valid syntax).

- [ ] **Step 5: Commit**

```bash
git add extension/gdocs.js
git commit -m "✨ feat(extension): record normalized ops + persist sharded gdocs log"
```

---

## Task 7: Extension — issue from the log in the popup

**Files:**
- Modify: `extension/popup.js`
- Modify: `extension/issue-worker.js`

- [ ] **Step 1: Teach the worker to issue from a log (and return the bound text)**

In `extension/issue-worker.js`, change the import and handler to support both shapes.
For the log path it also returns the WASM-reconstructed text (so the popup never
re-implements replay in JS):

```javascript
import init, { issue_credential, issue_from_capture_log, reconstruct_text_from_log } from "./pkg/humanshipd_web_verify.js";

let ready = null;
const ensureReady = () => (ready ||= init());

self.onmessage = async (event) => {
  try {
    await ensureReady();
    const { log, session, author } = event.data;
    if (log) {
      const logJson = JSON.stringify(log);
      const manifest = issue_from_capture_log(logJson, author || undefined);
      const text = reconstruct_text_from_log(logJson);
      self.postMessage({ ok: true, manifest, text }, [manifest.buffer]);
    } else {
      const manifest = issue_credential(JSON.stringify(session));
      self.postMessage({ ok: true, manifest }, [manifest.buffer]);
    }
  } catch (e) {
    self.postMessage({ ok: false, error: String(e && e.message ? e.message : e) });
  }
};
```

- [ ] **Step 2: Update the popup to send the log; use the worker's reconstructed text**

First, update `issueViaWorker` to resolve the **whole** worker result (it now also
carries `text` for the log path). Change its `worker.onmessage` line
`if (event.data?.ok) resolve(event.data.manifest);` to:

```javascript
      if (event.data?.ok) resolve(event.data);
```

The `getSession` response now carries `{ log, ... }` (gdocs) or the old session shape
(other surfaces). Replace the block from `show("Signing credential in your browser…")`
through the `manifestBytes = await issueViaWorker(...)` with:

```javascript
  show("Signing credential in your browser…");
  const author = document.getElementById("author").value.trim();
  let manifestBytes;
  let documentText;
  try {
    const payload = session.log
      ? { log: session.log, author }
      : { session: { ...session, author }, author };
    const result = await issueViaWorker(payload);
    manifestBytes = result.manifest;
    documentText = session.log ? result.text : session.final_text;
  } catch (e) {
    show(`Could not issue: ${e.message}`, "err");
    return;
  }
  const docBytes = new TextEncoder().encode(documentText);
```

Then change the `getSession` guards earlier in `popup.js`: the gdocs path has no `final_text`/`code_editor`. Replace the two guards (`if (session.code_editor)` and `if (!session.final_text …)`) so they only apply to the non-log path:

```javascript
  if (!session.log) {
    if (session.code_editor) {
      show("This looks like a code editor (Ace / CodeMirror / Monaco). humanshipd can't read its text yet — its content isn't in the page. Try a plain text box, or the macOS app for desktop editors.", "err");
      return;
    }
    if (!session.final_text || !session.final_text.trim()) {
      show("Couldn't read any text from this editor — it may be canvas- or model-based. Try a plain text box, or the macOS app.", "err");
      return;
    }
  }
```

Also relax the empty-session guard near the top so a log counts as captured. Replace `if (!session || !session.events?.length)` with:

```javascript
  if (!session || (!session.log && !session.events?.length)) {
```

(No JS replay helper — the document text comes from `reconstruct_text_from_log` in
the worker, keeping replay single-sourced in core.)

- [ ] **Step 3: Verify syntax**

Run: `node --check extension/popup.js && node --check extension/issue-worker.js`
Expected: no output.

- [ ] **Step 4: Commit**

```bash
git add extension/popup.js extension/issue-worker.js
git commit -m "✨ feat(extension): issue credential from the accumulated capture log"
```

---

## Task 8: Extension — permissions + continuity Playwright test

**Files:**
- Modify: `extension/manifest.json`
- Create: `extension/tests/continuity.spec.js`

- [ ] **Step 1: Add storage permissions**

In `extension/manifest.json`, change the `permissions` array to include storage:

```json
  "permissions": ["activeTab", "scripting", "downloads", "storage", "unlimitedStorage"],
```

- [ ] **Step 2: Write the failing test**

Create `extension/tests/continuity.spec.js`:

```javascript
// Regression for cross-session continuity: gdocs.js records normalized ops, persists
// them to a (shimmed) chrome.storage, and on a reload resumes — so getSession returns
// a log accumulating BOTH the prior session and the new one.

const { test, expect } = require("@playwright/test");
const fs = require("fs");
const path = require("path");

const GDOCS = fs.readFileSync(path.join(__dirname, "..", "gdocs.js"), "utf8");

// Load gdocs.js with a chrome shim backed by a JS object that survives a "reload".
async function boot(page, store) {
  await page.goto("data:text/html,<body></body>");
  await page.evaluate(({ gdocs, store }) => {
    window.__store = store;
    let getSession = null;
    window.chrome = {
      runtime: { onMessage: { addListener: (fn) => (getSession = fn) } },
      storage: {
        local: {
          set: (obj) => Object.assign(window.__store, obj),
          get: (_keys, cb) => cb({ ...window.__store }),
        },
      },
    };
    // Pretend we're on a doc URL so docId resolves.
    Object.defineProperty(window, "location", {
      value: { pathname: "/document/d/DOC123/edit" }, configurable: true,
    });
    // eslint-disable-next-line no-eval
    eval(gdocs);
    window.__post = (op, at) => window.postMessage({ source: "humanshipd-gdocs", kind: "op", op, at: at || Date.now() }, "*");
    window.__session = () => { let s = null; if (getSession) getSession({ type: "getSession" }, null, (r) => (s = r)); return s; };
  }, { gdocs: GDOCS, store });
}

const flush = (page) => page.evaluate(() => new Promise((r) => setTimeout(r, 30)));

test("resumes a prior session and accumulates the new one", async ({ page }) => {
  const store = {};
  // Session 1: type "Hello".
  await boot(page, store);
  await page.evaluate(() => window.__post({ ty: "is", ibi: 1, s: "Hello" }));
  await flush(page);
  await page.evaluate(() => new Promise((r) => setTimeout(r, 1600))); // let debounced save fire
  const after1 = await page.evaluate(() => JSON.parse(JSON.stringify(window.__store)));
  expect(Object.keys(after1).some((k) => k.includes("DOC123:s0"))).toBe(true);

  // "Reload": new page, SAME store object → prior session loads, new session appends.
  await boot(page, after1);
  await page.evaluate(() => window.__post({ ty: "is", ibi: 6, s: " world" }));
  await flush(page);
  const out = await page.evaluate(() => window.__session());
  expect(out.log.sessions.length).toBe(2);
  expect(out.log.sessions[0].ops[0].text).toBe("Hello");
  expect(out.log.sessions[1].ops[0].text).toBe(" world");
});
```

- [ ] **Step 3: Run test to verify it passes**

Run: `cd extension/tests && npx playwright test continuity.spec.js`
Expected: PASS — two sessions accumulate across the simulated reload.

(If it fails on the `location` redefinition, the gdocs.js reads `location.pathname` at IIFE start; the test defines it before `eval(gdocs)`, so it resolves.)

- [ ] **Step 4: Update `gdocs.spec.js` for the new shape**

`gdocs.js` no longer replays text or returns `final_text`/`events`; it returns a
`log`, and it now calls `chrome.storage.local` + reads `location.pathname`. Make
these exact edits to `extension/tests/gdocs.spec.js`:

(a) In its `boot` helper, extend the `chrome` shim and define `location` to match
`continuity.spec.js`:

```javascript
      window.__store = window.__store || {};
      window.chrome = {
        runtime: { onMessage: { addListener: (fn) => (getSession = fn) } },
        storage: { local: {
          set: (obj) => Object.assign(window.__store, obj),
          get: (_k, cb) => cb({ ...window.__store }),
        } },
      };
      Object.defineProperty(window, "location", { value: { pathname: "/document/d/DOC/edit" }, configurable: true });
```

(b) **Delete** the tests `replays is / ds / mlti into the final text, all
keystroke-backed` and `reconstructs the expected text from real /save bodies` —
that replay/reconstruction logic now lives in `core/tests/capture_log.rs`.

(c) **Adapt** the two remaining tests to read from the returned log instead of
`final_text`/`events`:
- `flags a paste …`: after posting a typed op, a paste event, and a matching insert,
  assert `out.log.sessions.at(-1).ops.some(o => o.pasted === true)` and that exactly
  one op has `pasted: true`.
- `the inject extracts ops from a /save body …`: after the `fetch` of the synthetic
  `/save` body, assert `out.log.sessions.at(-1).ops` contains the expected inserts
  (e.g. `ops[0].text === "From save "`).

Then run: `cd extension/tests && npx playwright test`
Expected: PASS — `capture.spec.js`, `gdocs.spec.js` (updated), `no-host.spec.js`,
`continuity.spec.js`.

- [ ] **Step 5: Commit**

```bash
git add extension/manifest.json extension/tests/continuity.spec.js extension/tests/gdocs.spec.js
git commit -m "✨ feat(extension): storage permission + cross-session continuity test"
```

---

## Task 9: Full gate + manual verification

- [ ] **Step 1: Run all automated gates**

```bash
cargo test --workspace
cargo clippy --workspace --exclude humanshipd-macos-capture --all-targets -- -D warnings
(cd web-verify && cargo check --target wasm32-unknown-unknown)
(cd web-verify/tests && npx playwright test)
(cd extension/tests && npx playwright test)
```
Expected: all green.

- [ ] **Step 2: Rebuild the extension bundle**

```bash
bash extension/build-wasm.sh
```
Expected: `extension/pkg` rebuilt with `issue_from_capture_log` in the glue
(`grep -c issue_from_capture_log extension/pkg/humanshipd_web_verify.js` ≥ 1).

- [ ] **Step 3: Manual end-to-end (user)**

Reload the extension at `chrome://extensions`. In a Google Doc: type a sentence, **close the tab**, reopen the same doc, type more, click Issue → one `.zip` → drop into the verify page → **exact-file**, and the report shows "2 sessions". Then refresh a doc that already had content → Issue should **decline** with the "wasn't captured from the start" message.

- [ ] **Step 4: Commit (if any doc/staleness fixes)**

Run `/flagrare:staleness-audit`; update `extension/README.md` (capture now persists across sessions) and the root README "Working today" line. Commit.

---

## Notes for the implementer

- **TDD throughout:** every core behavior has a Rust test before code; the WASM + extension behaviors have Playwright tests.
- **No text in the record:** `build_record` hashes the reconstructed text but never stores it. The text lives only in the local log + the zip's `document.txt`.
- **Decline path is load-bearing:** it's how we stay honest about unwitnessed content until Slice 3's backfill — keep the message user-facing.
- **gdocs-inject.js is unchanged:** it still forwards `/save` ops; only `gdocs.js`'s handling of them changed.
- **Load vs save ordering:** the storage `get(null)` on page load resolves in well
  under the 1500ms save debounce, so `sessionIndex` is set before the first save —
  early-typed ops stay in `ops[]` and land in the correct session. If you ever see a
  prior session getting overwritten, gate `saveNow()` behind a `loaded` flag set in
  the `get` callback.
