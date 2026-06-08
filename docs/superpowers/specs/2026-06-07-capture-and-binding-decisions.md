# Design: Capture & Binding — Locked Implementation Decisions

- **Date:** 2026-06-07
- **Status:** decisions locked (awaiting user review)
- **Purpose:** Close the five technical gaps surfaced before any refactor, with research-backed, committed answers. This is a *decisions* document — it locks *what* we do, not the v1 architecture (deliberately out of scope; see end).
- **Companion specs:** [`2026-06-06-content-binding-and-capture-fidelity-design.md`](./2026-06-06-content-binding-and-capture-fidelity-design.md), [`2026-06-06-authorship-signals-and-reporting-design.md`](./2026-06-06-authorship-signals-and-reporting-design.md), base [`2026-06-05-human-authorship-attestation-design.md`](./2026-06-05-human-authorship-attestation-design.md).
- **Research basis:** [`docs/research/2026-06-06-google-docs-writing-capture.md`](../../research/2026-06-06-google-docs-writing-capture.md), [`…-edit-stream-models-capture-pipeline.md`](../../research/2026-06-06-edit-stream-models-capture-pipeline.md), [`…-document-binding-and-content-identity.md`](../../research/2026-06-06-document-binding-and-content-identity.md), [`…-software-architecture-practices.md`](../../research/2026-06-06-software-architecture-practices.md).

## Cross-cutting principle

Every decision below resolves uncertainty the same way the project resolves everything else: **honest gradients over binary verdicts.** When sources disagree or coverage is partial, we *downgrade confidence and disclose it* — we never silently trust one source or overclaim completeness.

## Decision 1 — Paste detection in Google Docs

**Gap:** Docs exposes no raw keystrokes, and its saved edit log coalesces a burst of typing into a single insert op, so typing and pasting are indistinguishable from the edits alone.

**Locked (option C):**
- The **`paste` clipboard event** (captured at `document` level, as Grammarly does) is the **primary, real paste signal** — available only during *live* capture.
- A single `/save` bundle carrying one large insert is **weak corroboration**, not a standalone claim.
- The **historical-import path (`/revisions/load`) makes no paste claim.** A credential reconstructed from an already-written doc attests process shape, timeline, and content, but carries **no AI-dump flag**, and says so.

**Consequence accepted:** a credential for an already-written doc is weaker than a live-captured one. Honest, and matches the data.

**✅ Validated live (2026-06-07):** on a real Google Doc, the `paste` event fired in the editor iframe with the pasted text readable (136 chars). A fast-typed sentence produced **no** paste event yet coalesced into a 27-char insert — above the size threshold — so size alone would have false-flagged typing as a paste, while the paste event separated them cleanly. Confirms the design. (Note: paste listeners must be attached in the editor *iframe*, not just the top document.)

## Decision 2 — Live Google Docs capture mechanism

**Gap:** Live capture (now required by Decision 1) needs to read Docs' edits as they happen, but an isolated-world content script can't see the page's network calls or internals.

**Locked: combine all three sources by role, with reconciliation.**
- **(A) Main-world `fetch`/`XHR` patch on `/save` + `bind`** → **primary capture.** Injected into Docs' own JS context at `document_start`, forwards each mutation (positions + timestamps) to us. Paired with the `paste` event (Decision 1).
- **(B) `/revisions/load` history** → **reconciliation + backfill.** At seal time, pull Docs' server-committed history to (i) cross-check the live stream and (ii) backfill edits from before capture / prior sessions.
- **(C) Final rendered text** → **consistency gate.** Replay captured ops, reconstruct text, confirm it matches the document's actual text before hashing/binding.
- **On disagreement** (live ≠ committed, or replay ≠ actual): **downgrade the credential's confidence and disclose** ("captured live; partially reconciled") rather than refuse to issue.

**✅ Validated live (2026-06-07):** an XHR-prototype patch caught every `/save` on a real doc — even installed after page load (prototype patching is retroactive; production should still inject at `document_start` in the MAIN world for full coverage). The save body matches our `gdocs` parser (`rev` + `bundles[].commands[]`, `ty:is`/`ibi`/`s`). Typing coalesced into medium inserts (`is(27)`, `is(7)`); the 136-char paste landed as one clean `is(136)` exactly matching the paste event's length — paste event and save insert corroborate by size.

**✅ B-reconcile + C-consistency validated (2026-06-07):** pulled the real full committed `/revisions/load` changelog and ran it through our actual `gdocs::session_from_changelog` parser. It reconstructed text that is **ISCC-identical (Hamming 0/64)** to the document's real `.txt` export — so the committed history holds the live-captured ops (reconcile) and replay equals reality (consistency). Lands on the same content code (`EAAXLIOV6OJBYPGY`) as the `.docx` and `.txt` of the same doc. (Observed earlier: the committed log *lags* live edits by a minute or two before consolidating — exactly the gap the downgrade-and-disclose rule covers.) Spike: `cargo run --example gdocs_consistency`.

## Decision 3 — What the credential binds to, and how it travels

**Gap:** we hash plain-text bytes, but the artifact a person keeps/shares is a formatted file (`.docx`, PDF, Google Doc); the reader has the file, not our text.

**Locked: two-phase, embed-where-supported.**
- **Capture during writing, seal at export.** The credential is finalized and bound to the *finished file's* bytes at export time (you can only hash final bytes).
- **Travel:** **embed the C2PA manifest inside the file** where the format allows it (PDF, `.docx`, EPUB, ODF); **sidecar `.c2pa` fallback** for formats without an embed slot (plain text, Google Docs). The credential rides inside the document where possible — the standard "Content Credentials" model.

**⚠️ Validated (2026-06-07) — embedding is blocked by our library today.** `c2pa-rs` 0.85 implements **no** embedding for our document formats: the PDF handler's write path returns `WRITE_NOT_IMPLEMENTED` (read-only), and there is **no OOXML/`.docx`/EPUB/ODF handler at all** (only images, PDF-read, and media). The C2PA *spec* supports ZIP-based embedding since 1.4, but the Rust library hasn't shipped it. **Consequence for v1:** the **sidecar `.c2pa` is the universal binding for every document format** — the embed branch is *deferred* until c2pa-rs adds PDF-write / ZIP-based embedding (or we implement an asset handler ourselves, which would be reinventing library work — not now). The decision's *intent* stands; only its near-term realization narrows to sidecar-only.

## Decision 4 — Cross-format content identity & the verification verdict

**Gap:** a credential sealed to a `.docx` won't byte-match a PDF export of the same writing; we need identity that survives reformatting plus an honest verdict.

**Locked: ISCC + a four-tier banded verdict.**
- **Content identity = ISCC (ISO 24138)**, carried in the credential: the **Instance-Code** (byte-exact "this file") and the **Content-Code** (similarity-preserving over normalized text → survives `.txt`→`.docx`→`.pdf`). Adopt the standard; don't invent a fingerprint.
- **Verdict tiers** (a band, never a false-precision percentage; **threshold published**):
  1. **Exact file** — Instance-Code / hard binding matches.
  2. **Same writing** — Content-Code within the published distance threshold.
  3. **Borderline — needs review** — near the threshold (the fuzzy boundary, shown honestly).
  4. **No match** — beyond threshold.
- Verification extracts text per-format before computing the Content-Code (a format front-end feeding one matching engine).

**✅ Validated (2026-06-07):** spike over our existing `iscc-lib` Text-Code. Same writing with PDF-style line wraps and `.docx`-style double-spaces/space-before-punctuation produced the **identical** code (**Hamming 0/64**); a one-word edit was **15/64**; genuinely different writing was **29/64**. Clear separation between *same / lightly-edited / different* — so a published Hamming threshold (with a borderline band) is well-founded. *Caveat:* a 64-bit code is coarse and the exact thresholds need corpus calibration, not a single example; pairing with a winnowing fingerprint for partial/containment matches remains future work.

**✅ Verify path validated on a real file (2026-06-07):** exported the live test doc from Google as both `.docx` and `.txt`. A minimal OOXML extractor (unzip → `word/document.xml` → `<w:t>` runs) pulled the text from the real `.docx`; its ISCC equalled the ISCC of Google's own `.txt` export of the same doc — **Hamming 0/64** — while different writing was 31/64. So a reader dropping in their actual `.docx` lands on the same content code as the captured writing. Confirms the full cross-format verify path end-to-end (spike: `cargo run --example docx_extract`). The extractor is crude (string-scan, no headers/footers/tracked-changes handling yet) — the deterministic "which text counts" rules from the open questions still need nailing.

**✅ PDF leg validated too (2026-06-07):** extracting text from the real Google-exported `.pdf` via `pdf-extract` produced *garbled* output (broken word spacing, stray newlines — PDF is the messy format), yet its ISCC still equalled the `.txt` export's — **Hamming 0/64** (`cargo run --example pdf_extract`). ISCC normalizes whitespace away before hashing, so even sloppy PDF extraction lands on the same code. **Net: all four real formats of the same writing — reconstructed-from-changelog, `.docx`, `.txt`, `.pdf` — produce the identical content code `EAAXLIOV6OJBYPGY`.** The cross-format identity is robust to extraction noise.

**◧ Threshold calibration (2026-06-07, synthetic — provisional):** measured normalized Hamming (bits-different ÷ total-bits) at 64-bit vs 256-bit (`cargo run --example iscc_calibration`). At **256-bit**: reformatted **0.00**, one-word edit **0.06**, ~5-word edit **0.17**, unrelated **0.44–0.50** (≈ the 0.5 random baseline). The 256-bit code grades edits much more smoothly than 64-bit (which is coarse — one word already cost ~0.09–0.22), with a **clean gap between lightly-edited (≤~0.17) and different (≥~0.44)**. **Decisions this firms up:** (a) compute the content code at **256-bit**, not the current 64-bit; (b) score by **normalized Hamming** so code size is decoupled from the threshold; (c) *provisional* bands — Exact/same-content ≈ 0 (≤~0.05), Same-writing ≤~0.15, Borderline ~0.15–0.30, No-match >~0.30. These synthetic bands were then confirmed on a **real corpus sourced online** (no user files needed).

**✅ Real-corpus calibration (2026-06-08):** genuine human edits = successive Wikipedia revisions of one article; "different" = unrelated articles (`cargo run --example corpus_calibration`). 256-bit normalized Hamming vs the current revision: 1 revision back **0.008**, 3 back **0.020**, 10 back **0.020**, 40 back **0.078**, **150 revisions back 0.117**; unrelated articles **0.441** and **0.527**. So *all* real same-article edits — even 150 revisions of accumulated editing — stay **≤ ~0.12**, while different writing is **≥ ~0.44**: a wide empty gap, not a knife-edge.

**Locked bands (256-bit ISCC Content-Code, normalized Hamming):**
- **Exact / same content:** ≤ 0.05 (reformatting, format conversion, light edits)
- **Same writing:** ≤ 0.20 (covers heavy multi-revision editing, observed max 0.12)
- **Borderline — needs review:** 0.20 – 0.35 (the empty zone; surface for human review)
- **No match:** > 0.35 (unrelated content, observed ≥ 0.44)

*Caveat:* one article + two unrelated docs is a modest corpus — broadening it would tighten the exact numbers, but the 0.12-vs-0.44 separation is large enough that the conclusion (real edits and different content are cleanly separable, with margin) is robust.

## Decision 5 — What one credential covers

**Gap:** documents are written across many sessions; our model captured one sitting.

**Locked: whole-document, accumulated.**
- A credential attests the document's **entire captured writing history**, accumulated across sessions, sealed at export — the claim people actually want ("a human wrote *this document*").
- Requires a **local, append-only per-document capture log** between sessions, keyed by a **stable document identity**: the **doc ID** for Google Docs (clean); for native files, a best-effort identity (path/heuristic) — **flagged as an open question** (see below).
- **Coverage honesty:** pre-capture history and inter-session gaps are **backfilled from the revision log where available, else marked unknown and the confidence downgraded-and-disclosed** (Decision 2). "Whole-document" never implies complete coverage we don't have.

## Out of scope / deferred (not decided here)

- **The v1 refactor architecture** — crate boundaries, the ports-&-adapters capture port, the append-only `EditEvent` log as source of truth, schema-versioning discipline. The research for these is done ([architecture practices](../../research/2026-06-06-software-architecture-practices.md), [edit-stream models](../../research/2026-06-06-edit-stream-models-capture-pipeline.md)); the design is a separate, later brainstorm.
- **Signing / trust model** beyond the current `EphemeralSigner` placeholder (real cert chain / CAWG verified identity).
- **Native-file document identity** (Decision 5's harder case) — needs its own resolution when desktop capture is built.
