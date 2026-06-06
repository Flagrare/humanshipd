# Design: Human-Authorship Attestation (synthy-free / "Authorshipped")

- **Date:** 2026-06-05
- **Status:** draft (awaiting user review)
- **Companion specs:** [`2026-06-06-authorship-signals-and-reporting-design.md`](./2026-06-06-authorship-signals-and-reporting-design.md) — the signal taxonomy (provenance vs. inference), per-span banded report, content-free replay/visualization, competitive landscape, and the borrowed-feature roadmap. [`2026-06-06-content-binding-and-capture-fidelity-design.md`](./2026-06-06-content-binding-and-capture-fidelity-design.md) — reading real web editors and verifying real document formats (not just exact plain text).
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

#### 7.2.1 Tracked-apps picker (user agency)

The desktop control app surfaces an explicit **list of apps the user can choose to track** — ideally pre-populated from writing apps detected on the machine (e.g. Word, Scrivener, Final Draft), with everything **off by default**. The user opts each app in.

This reframes the scary permission ("this tool can read input") into visible agency ("you chose exactly these apps, and you can see and change the list anytime"). It also enforces the capture allow-list technically: only apps the user enabled are ever observed. A defining feature of the desktop control app, not an afterthought.

## 8. POC scope (thin vertical slice)

**Build:** capture adapter → Rust core → build record → sign + RFC 3161 timestamp → verify (badge validated, claim shown).

Because capture is a Strategy (adapters emit a common `EditEvent` stream the core consumes), the *first* capture adapter is interchangeable. **Decision (2026-06-05): build the native macOS Accessibility adapter first** (it's the reusable adapter that also serves Scrivener/Final Draft, keeps the stack Rust-native, and avoids the browser/two-codebase concern). The **Google Docs MV3 extension moves to last.**

**Deferred:** OCR fallback; full C2PA/CAWG/VC emission; transparency log; zero-knowledge process attestation (research track).

**Build sequence (✓ = done):**
1. ✓ Rust core: record schema + canonicalization + hashing + verify.
2. ✓ Rust core: signing + time-anchoring (pluggable authority; POC local TSA).
3. ✓ Native Messaging host wrapping the core + headless `verify_badge` CLI.
4. **Native macOS capture adapter** (`AXUIElement` text diffs + `CGEventTap` keystroke timing): de-risk on **TextEdit** first, then **Word**. Emits the `EditEvent` stream → core.
5. WASM verify page (in-browser version of the CLI verifier).
6. Google Docs MV3 extension (capture via in-page edit events → native messaging).
7. End-to-end test in each target app.

## 9. Tech stack — adopt standards, don't reinvent

Principle: engineering effort goes to the value proposition (human-authorship attestation). Everything underneath builds on the canonical standard + its reference OSS. See research [`adopting-c2pa-credential-stack`](../../research/2026-06-05-adopting-c2pa-credential-stack.md) and [`app-agnostic-capture-architecture`](../../research/2026-06-05-app-agnostic-capture-architecture.md).

**Credential / provenance — adopt C2PA, do not hand-roll an envelope:**
- **`c2pa` (c2pa-rs 0.85)** — the credential is a **C2PA manifest**: our process record is a custom assertion `org.humanshipd.process.v1`; COSE/X.509 claim signature (ed25519); **built-in RFC 3161** timestamp via `tsa_url`; emitted as a **detached `.c2pa` sidecar** over the final-text SHA-256 (no native text handler, and matches our hash-only model). `Reader` does verification.
- **CAWG identity assertion** (`c2pa::identity`) — binds the named human author (`cawg.creator`); "human-authored" semantics via `digitalSourceType` + our custom assertion (claim semantics stay ours).
- **Deferred / opt-in:** W3C VC (`ssi`) — CAWG VC integration spec not yet stable; Sigstore **Rekor** (`sigstore-rekor`) — optional public anchoring (network, off by default).
- **Stays bespoke (no standard exists):** the process-metadata schema and the honest claim/threshold semantics.
- **Tradeoff accepted:** c2pa-rs is heavy, but interop with the C2PA / Authors-Guild ecosystem *is* the value — the one place a heavy standard is justified.

**Capture — adopt established AX libraries, keep deps lean (auditable attack surface):**
- **macOS:** `accessibility-sys` + `accessibility` crates with **event-driven `AXObserver` + CFRunLoop** (not raw FFI + polling). Resolve focused text via `AXFocusedUIElement`, descending containers (Word's `AXSplitGroup`/`AXScrollArea`) to `AXTextArea`. **Do not** vendor screenpipe (its capture is a committed binary and the approach is deprecated upstream).
- **Windows (later):** `uiautomation`. **Linux (later):** `atspi`. Both are the mature standard adoption targets.

**Frontend:** Extension in TypeScript, MV3, thin tap over Native Messaging. Desktop app (Phase 2) in Tauri (wraps the same core).

**License:** permissive (MIT/Apache).

## 10. Open questions (to resolve during build)

- Final brand/name (POC codename: "Authorshipped"/"Humanship"; claim wording locked: **"Human Authored"**).
- Which public RFC 3161 TSA(s) to default to; whether to ship a transparency-log option in v1.
- Exact bucketing for burst/pause/revision stats (privacy vs. signal).
- (Resolved) Local replay is now a defined optional feature — see §7.1.

## 11. References

See the three research catalog files linked at the top and indexed in [`docs/research/README.md`](../../research/README.md).
