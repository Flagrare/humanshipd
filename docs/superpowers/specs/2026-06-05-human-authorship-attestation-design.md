# Design: Human-Authorship Attestation (synthy-free / "Authorshipped")

- **Date:** 2026-06-05
- **Status:** draft (awaiting user review)
- **Research basis:** [`docs/research/2026-06-05-proof-of-human-authorship-feasibility.md`](../../research/2026-06-05-proof-of-human-authorship-feasibility.md), [`…-app-agnostic-capture-architecture.md`](../../research/2026-06-05-app-agnostic-capture-architecture.md), [`…-human-authorship-label-framing.md`](../../research/2026-06-05-human-authorship-label-framing.md)

## 1. Summary

An open-source, local-only, zero-telemetry tool that records *how* a piece of text was composed and issues a verifiable **"Human Authored"** credential backed by that record. It is the inverse of DeepMind SynthID: rather than watermarking AI output, it attests a human writing process.

It is deliberately framed as a **tamper-evident attestation and deterrent — not proof.** This honesty is a feature: because the project is open-source, its threat model is published in full (§4), which the commercial players do not do.

## 2. Goals / Non-goals

**Goals**
- Capture a writing-process record across the apps people already use (blend in; no forced editor switch).
- Keep all content on-device; only hashes ever leave the machine. Zero telemetry.
- Produce a credential a third party can verify for **integrity** and **time-anchoring** without trusting the author's machine.
- Be extensible to new writing surfaces (mainstream docs first; scriptwriting/film later) behind one shared core.
- Be honest and auditable: the limits are documented and the code is inspectable.

**Non-goals**
- Proving a human originated the *ideas* (information-theoretically impossible on user-controlled hardware — §4).
- Post-hoc "is this AI?" classifier detection (unreliable; explicitly out of scope).
- Monetization, accounts, user tracking, or any cloud processing of content.
- Defeating a motivated adversary who retypes AI output (the copy-type attack — undefeatable; disclosed).

## 3. Architecture

Four layers; **all credential logic lives in one shared Rust core**, so capture adapters stay thin and cannot drift (1Password/Bitwarden/Tailscale pattern).

```
 CAPTURE adapters (thin, per-surface) ──────────────────────────────
   • Browser extension (MV3)  → Google Docs + web editors
        thin event tap → Native Messaging (stdio JSON) ──┐
   • OS Accessibility adapter → native apps (Word/Scrivener/Final Draft)
        AXUIElement / UIA / AT-SPI ───────────────────────┤
   • OCR fallback             → last resort only ─────────┘
                                                          │ raw events
                                                          ▼
 RUST CORE (one implementation of everything below) ────────────────
   RECORD   build metadata-only writing-session record (no content)
   ANCHOR   hash → sign → RFC 3161 timestamp (sends only a hash)
   VERIFY   validate a record/badge offline
                                                          │ hash only
                                                          ▼
 TRUST ANCHOR (network touch #1, optional/stateless) ───────────────
   • RFC 3161 Timestamping Authority (trusted time)
   • (optional) public transparency log (append-only inclusion)
                                                          ▼
 CREDENTIAL / VERIFY  badge + verify page; schema shaped to map onto
   C2PA manifest + CAWG identity assertion + W3C VC (interop later)
```

### 3.1 Capture adapters
- **Browser extension (MV3):** a thin content-script event tap for Google Docs (live edit events; Docs' fine-grained data is in-page only) and generic web editors. Forwards raw events over **Native Messaging** to the Rust core companion. Holds **no** credential logic.
- **OS Accessibility adapter:** reads the focused text element + change events from an allow-listed native writing app via AXUIElement (macOS) / UIA (Windows) / AT-SPI (Linux/GNOME). This is the scriptwriting/film extensibility path.
- **Input-timing correlation:** keystroke *timing only* (never content) to flag text that appeared *without* keystrokes — the AI paste/dump detector.
- **OCR fallback:** on-device OCR (Vision / Windows.Media.Ocr / Tesseract) only where no extension/AX coverage exists.

### 3.2 What leaves the device
Only a **content hash** and a **record hash** (for the timestamp request), plus the returned signed timestamp token. Never: text content, keystrokes, or telemetry. Raw events may be kept **locally** for the author's own replay, but never transmitted.

## 4. Threat model (published openly)

| Attack | Outcome | Our stance |
|---|---|---|
| **Copy-type** (human reads AI text, retypes it) | **Undefeatable.** Genuine human motor signals; I(timing; provenance \| content) = 0. | Disclosed prominently. Challenge-response + revision-trajectory analysis raise *cost*, never close it. |
| **Fork-and-forge** (open-source client modified to emit a fake signed record) | Possible. | The badge therefore attests **integrity after issuance**, not authorship authenticity. Stated plainly. |
| **Wholesale AI paste / dump** | **Detected** (large insertion without correlated keystrokes). | A defensible positive signal. |
| **Post-issuance tampering / back-dating** | **Detected** via signature + RFC 3161 timestamp (+ optional transparency log). | Core guarantee. |
| **Synthetic keystroke injection / replay** | Partially detectable (machine-vs-human motor signal), but not vs. a real human. | Documented; not over-claimed. |

**The honest one-line claim:** *"This record was produced by this client and has not been altered since time T"* — plus *"the writing showed an incremental human-like composition process with no large un-keyed insertions."* Never *"a human definitely wrote this."*

## 5. Record schema (metadata-only)

The `WritingSessionRecord` contains **no document content** — only counts, timing, and hashes. Illustrative shape (final field set during implementation):

```jsonc
{
  "schema": "authorshipped/record@0.1",
  "session_id": "<random, unlinkable>",
  "surface": { "kind": "gdocs|native-ax|web|ocr", "app": "<id>" },
  "document_binding": { "final_text_sha256": "<hex>", "char_count": 1234 },
  "process": {
    "active_ms": 0,
    "keystrokes": 0,
    "bursts": { "count": 0, "mean_len": 0, "buckets": [] },
    "pauses": { "gt_2s": 0, "buckets": [] },
    "revisions": { "insertions": 0, "deletions": 0, "reformulations": 0 },
    "insertions_without_keystrokes": [ { "size": 0 } ],   // AI-dump signal
    "keyed_fraction": 0.0                                   // typed vs appeared
  },
  "evidence_flags": { "large_unkeyed_insertions": 0 },
  "replay": { "available": false, "log_sha256": null },    // optional, local-only; hash binds a shared replay to this session (§7.1)
  "integrity": {
    "record_sha256": "<hex>",
    "client_signature": "<sig over record_sha256>",        // forgeable; binds to a key
    "rfc3161_token": "<base64 TSA token over record_sha256>",
    "transparency_log": null                                 // optional inclusion proof
  }
}
```

Design constraints:
- Canonicalize before hashing (stable field order) so the same logical record hashes identically across adapters.
- Schema versioned from day one.
- Field set kept C2PA/CAWG/VC-mappable (so standards interop is an extension, not a rewrite).

## 6. Trust anchor & tamper-check server

- **Signing:** the core signs `record_sha256` with a client key. (Acknowledged forgeable in an open-source/local context — this binds the record to a key and gives integrity, not authorship.)
- **Trusted time:** request an **RFC 3161** timestamp token from a public Timestamping Authority, sending only the hash. Proves the record existed by time T and is tamper-evident.
- **(Optional) transparency log:** append the hash to a public append-only log for independent back-dating detection.
- **The "at most a server":** a **stateless** verifier that, given a badge, checks the signature + timestamp token (+ inclusion proof). It never sees content and stores no PII — consistent with zero-telemetry.

## 7. Credential & verification

- Output a **badge** + a **verify page/CLI** that validates a record offline (signature, timestamp, document-hash match) and renders the honest claim text (§4).
- Schema shaped to map onto a **C2PA manifest + CAWG identity assertion + W3C VC** so the credential can become standards-interoperable later (the gap: CAWG has no "human-authored" role today — a slot this project can help define).

### 7.1 Optional feature: writing replay

The capture layer already produces an ordered edit-event stream, so a Draftback-style **replay** (watch the document being written) is nearly free as a presentation layer. It is included, but deliberately **quarantined from the default credential** because it requires retaining content, which conflicts with the metadata-only posture.

Rules:
- **Local-only and off by default.** An author may replay *their own* session on their machine; nothing leaves the device.
- **Opt-in sharing only.** Attaching a replay as supplementary evidence (e.g., student→teacher, author→publisher) is the author's explicit, informed choice to expose content. Never automatic.
- **Hash-bound.** The replay log's hash (`replay.log_sha256`) is part of the signed record, so the default badge stays content-free, but a *shared* replay can be verified as the genuine, un-doctored session.
- **No security uplift.** Replay adds persuasiveness to a human viewer, not cryptographic strength: copy-typed text replays as smooth human writing, and "human auto-typers" can manufacture fake replays. Documented as such; never presented as proof.

### 7.2 UI surfaces

Four surfaces; the POC needs only the first two. The UI is part of the trust story — it must make the privacy guarantees visible and must never overclaim.

| Surface | Role | Phase |
|---|---|---|
| **Extension popup** | Capture status (recording/idle, current doc), an always-visible capture indicator + one-click pause, and an explicit **"Issue Human Authored credential"** button | POC |
| **Verify page** | Static page; drop in a badge/record → render the honest claim + pass/fail. Verification runs **in-browser via the Rust core compiled to WASM** — no server, content never leaves | POC |
| **Desktop control app** | Menu-bar/tray: status, pause, permissions + allow-list for the OS-Accessibility adapter. Built in **Tauri** (Rust-native, wraps the same core) | Phase 2 |
| **Replay viewer** | Local Draftback-style player (§7.1) | Phase 2 |

UI non-negotiables (from the project values):
- Always-visible capture indicator + one-click pause (not a covert keylogger).
- "Metadata only — nothing leaves your device" stated in-context.
- The verify page renders the **nuanced** claim ("incremental human-like process; not altered since T"), **never** a "✓ 100% Human" badge that would overclaim.

## 8. POC scope (thin vertical slice)

**Build:** Browser extension (Google Docs) → Native Messaging → Rust core → build record → sign + RFC 3161 timestamp → local verify page that validates the badge and shows the claim.

**Deferred:** OS Accessibility adapter; OCR; full C2PA/CAWG/VC emission; transparency log; zero-knowledge process attestation (research track).

**Build sequence:**
1. Rust core: record schema + canonicalization + hashing + verify (test-first).
2. Rust core: signing + RFC 3161 client.
3. Native Messaging host wrapping the core.
4. MV3 extension: capture Google Docs edit events + paste detection; forward to host.
5. Verify page/CLI: validate a badge, render the honest claim.
6. End-to-end test on a real Google Doc.

## 9. Tech stack

- **Core:** Rust (single reusable implementation; strong crypto + RFC 3161 ecosystem; the official C2PA lib is Rust; reusable by future native adapters).
- **Extension:** TypeScript, MV3, thin tap over Native Messaging. (Type drift avoided via generated types if/when the extension needs core types.)
- **License:** permissive (MIT/Apache) — reuse screenpipe *patterns*, not GPL code.

## 10. Open questions (to resolve during build)

- Final brand/name (POC codename: "Authorshipped"/"Humanship"; claim wording locked: **"Human Authored"**).
- Which public RFC 3161 TSA(s) to default to; whether to ship a transparency-log option in v1.
- Exact bucketing for burst/pause/revision stats (privacy vs. signal).
- (Resolved) Local replay is now a defined optional feature — see §7.1.

## 11. References

See the three research catalog files linked at the top and indexed in [`docs/research/README.md`](../../research/README.md).
