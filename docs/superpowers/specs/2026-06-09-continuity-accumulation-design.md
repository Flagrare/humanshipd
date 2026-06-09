# Cross-session continuity & whole-document accumulation — Slice 1 design

- **Date:** 2026-06-09
- **Status:** design (approved in brainstorming; pending implementation plan)
- **Decisions advanced:** Decision 5 (what one credential covers / accumulation), foundation for Decision 7 (native identity) and Decision 2B/2C (backfill + consistency).

## Context

The browser extension reconstructs a document in an **empty buffer that resets on
every page load**. Open or refresh a Doc that already has content and type, and the
first `/save` op ("insert at position 12") lands at position 0 — you capture only
this session's edits, mis-positioned, and the credential binds to a partial,
incorrect document. The user's plain framing: *"I write today, turn off the PC, come
back tomorrow — it's as if I'm writing from scratch."*

The fix is **continuity**: a per-document, append-only capture log that **persists
across sessions**, keyed by a stable document identity. The same capability a
years-old local `.docx` needs — so it must be a **shared core** capability, not a
Docs-only extension hack (building it twice is the drift we're avoiding).

This is large, so it's decomposed into three independently shippable slices:

- **Slice 1 (this spec)** — core accumulation log + Google Docs persistence,
  **witnessed-only**. Fixes the bug; lays the shared foundation.
- **Slice 2** — native-file identity (Decision 7: layered embedded-GUID + OS-identity
  + content-fingerprint) + macOS persistence.
- **Slice 3** — `/revisions/load` backfill of unwitnessed history + witnessed-vs-
  reconstructed labeling (the `Unknown` provenance band).

Slice 1 is designed so 2 and 3 plug in without rework.

## Architecture (Approach A: core owns the model + logic; adapters own I/O)

The op-replay and accumulation live in **one place — the Rust core**, compiled
native *and* WASM. Adapters do only I/O (capture, storage, identity). This keeps a
single source of truth (no JS↔Rust drift) and fits WASM, where storage *must* live
JS-side anyway.

### Core: the capture log

A new serializable, versioned type:

```
CaptureLog ("authorshipped/log@1")
  identity: DocumentIdentity        // keys "this document"
  sessions: [ CaptureSession ]      // in order, one per writing session
CaptureSession
  session_id, surface_kind, surface_app
  started_at_ms: u64                // absolute epoch ms
  ops: [ CapturedOp ]               // raw insert(pos, text) / delete(range), at_ms (rel. to session), pasted: bool
DocumentIdentity                    // Slice 1: { kind: "gdocs", id: <URL doc id> }
```

`CapturedOp` holds the inserted **text** — necessary to reconstruct the document and
position later edits. **Privacy boundary:** the log (persisted locally) therefore
contains document content; it stays on the machine and **never enters the
record/credential**, which remains content-free.

Core functions (the only place replay/accumulation exists):

- `append(session)` — add a session's ops.
- `build_record() -> Result<Record, Declined>` — replay **all** sessions' ops into a
  reconstructed text, then a content-free `WritingSessionRecord` aggregated across
  sessions. **Declines** (errors) when an op can't be positioned against the
  witnessed buffer (see Consistency).
- serde serialize / deserialize for the adapter to persist.

**This replaces the op-replay currently hand-ported in `extension/gdocs.js`** (it
mirrors `core/src/gdocs.rs::apply_op` today) — deleting that duplication. `gdocs.js`
keeps only what's inherently JS-side: capturing ops and **attaching the paste flag**
(the `paste` browser event).

### Cross-session aggregation (the correctness-critical part)

`build_record` over multiple sessions is **not** naïve event concatenation:

- **Counts** (keystrokes, inserted, spans, revisions) — summed across sessions.
- **Active time** — the **sum of each session's writing span** (`last.at_ms -
  first.at_ms`), *not* wall-clock first-to-last (so a 3-day gap doesn't read as
  "active 3 days").
- **Pauses / bursts** — computed per session, then combined. **Cross-session gaps are
  session boundaries, not pauses.**
- **Timeline** (fingerprint graph) — sessions laid on a continuous *writing-time*
  axis (concatenated active spans), with **session boundaries marked**, then stride-
  sampled to `MAX_TIMELINE_POINTS` as today.
- **Document binding** — `sha256(reconstructed text)` + char count.

The record schema gains a small amount: `sessions: count` and first/last timestamps,
so the report can say *"written across N sessions over D days."*

### Document identity (Slice 1)

`DocumentIdentity = { kind: "gdocs", id: <URL doc id> }` — the doc id is stable in
the URL (`/d/<id>/`). The interface is shaped so native (Slice 2) adds
`{ kind: "native", … }` without touching the log or accumulation.

### Browser adapter: persistence & lifecycle

**Storage:** `chrome.storage.local`, **sharded by session** — each past session is
its own immutable key (`humanshipd:log:gdocs:<docId>:s<n>`); only the *active*
session's key is rewritten during capture, so writes are O(today's edits), not O(all
history). Isolated from Google for free (the extension's own storage, not the page
origin — unlike a content-script IndexedDB, which would be `docs.google.com`'s).
Behind the adapter's I/O boundary, so swappable to extension-context IndexedDB later
without touching core. New permissions: `storage` + `unlimitedStorage` (the only new
ones; still no host, no network).

**Lifecycle:**
1. **Page load** (`gdocs.js`): read doc id from the URL → load saved session keys for
   it. Found → resumed doc; not → fresh.
2. **Capture:** collect raw ops live, attach the paste flag; persist the active
   session's key **debounced** (every few seconds + on `pagehide`/`visibilitychange`)
   so turning off the PC mid-session loses nothing.
3. **Resume (next day):** prior sessions already loaded; today's edits open a **new**
   `CaptureSession`.
4. **Issue:** popup → full accumulated log → WASM `build_record()` → content-free
   record + reconstructed text → existing in-browser worker signs → zip + download.
   On decline, the popup shows the clear "didn't witness from the start" message.

Net effect: `gdocs.js` shrinks to **capture + paste-flag + store + forward** — the
replay buffer is gone (it's in core now).

## Edge cases & error handling

- **Gap / external edits** — one unified check: if an op lands beyond the witnessed
  buffer, `build_record` **declines**. Covers both "doc had pre-existing content" and
  "edited on another device between sessions," without `/revisions/load`. Honest
  message; backfill (Slice 3) is the real fix for pre-existing docs.
- **Multiple tabs of one doc** — each tab is its own `CaptureSession` (separate
  keys), merged at issue. No locking, no races.
- **Schema version mismatch** — log is versioned; an unreadable version is discarded
  and restarted (migrations later).
- **Storage quota failure** — surfaced as a clear error, never a silent loss.
- **No log + Issue** — existing "nothing captured yet" message.

## Testing

- **Core (Rust)** — the meat: append two sessions → `build_record` reconstructs the
  correct accumulated text + aggregated stats (active_ms = sum of spans, session
  count, cross-session gaps not counted as pauses); the position-consistency decline;
  serde round-trip.
- **WASM (Playwright)** — issue from a multi-session log in-browser → verifies
  exact-file; decline returns the right error.
- **Extension (Playwright)** — extend the gdocs spec: capture → persist to a shimmed
  `chrome.storage` → reload → resume → assert the accumulated log includes the prior
  session.
- **Manual** — write in a Doc, close, reopen, write more, Issue → one credential
  covering both sessions.

## Out of scope (later slices)

- **Native-file identity + macOS persistence** (Slice 2 / Decision 7).
- **`/revisions/load` backfill + witnessed-vs-reconstructed labeling** (Slice 3 /
  Decision 2B/2C).
- **Reading the canvas to cross-check the reconstructed text** — impossible for Docs;
  the witnessed log is authoritative, and divergence is handled by the decline check.
