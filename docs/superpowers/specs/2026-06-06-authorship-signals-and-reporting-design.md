# Design: Authorship Signals, Banded Reporting & Competitive Roadmap

- **Date:** 2026-06-06
- **Status:** draft (awaiting user review)
- **Companion to:** [`2026-06-05-human-authorship-attestation-design.md`](./2026-06-05-human-authorship-attestation-design.md) — that spec defines the architecture, threat model, credential format, and POC. **This** spec defines *what we measure*, *how we report it honestly*, and *which competitor features we adopt and when*.
- **Research basis:** [`docs/research/2026-06-06-ai-authorship-signals-and-probability-bands.md`](../../research/2026-06-06-ai-authorship-signals-and-probability-bands.md) and the four competitive-landscape research threads of 2026-06-06 (catalogued in `docs/research/`).

## 1. Summary

humanshipd records *how* a document was written and issues a verifiable credential. This document specifies the **signal model** behind that credential — the statistics and events we capture — and the **banded provenance report** we render from them, modeled honestly after Grammarly's "Writing activity report" but corrected for its weaknesses.

The governing principle, established by the research, is one distinction:

> **Provenance** (a verifiable record of what happened — typed, pasted, pasted-from-an-AI-tool) is high-confidence and signable. **Inference** (guessing AI-ness from the words themselves) is uncalibrated, biased, evadable, and *provably* capped. We lead with provenance and treat inference as an optional, clearly-flagged, low-confidence add-on — never a headline.

This is not a hedge. It is the project's central technical bet, and it is where every credible competitor is migrating while we are already there.

## 2. The provenance-vs-inference principle

"Probability the content is AI-generated" is the wrong target. It inherits four problems with no engineering fix:

1. **Bias.** Seven detectors averaged a **61% false-positive rate** on non-native English essays (Liang et al., Stanford 2023) vs. 5.19% on US 8th-grade essays — they flag low perplexity, not AI.
2. **Vendor non-confidence.** OpenAI's own classifier caught **26% of AI text** while false-flagging **9% of human text**, and was **retired** in July 2023 "due to its low rate of accuracy."
3. **Trivial evasion.** Recursive paraphrasing drops detector scores below threshold; watermarks degrade under meaning-preserving edits.
4. **A theoretical ceiling.** Sadasivan et al. proved the best possible detector obeys `AUROC ≤ ½ + TV(M,H) − TV(M,H)²/2`: as models approach the human text distribution, detection → coin flip. This is a bound, not a gap to close.

Grammarly itself sidesteps all four: it states Authorship "does not analyse the contents of the text, words or structure." Its bands are **word-count proportions of verified events**, not detector confidence. We adopt that stance and harden it (see §3, §5).

## 3. The signal taxonomy (three confidence tiers)

Every signal we might use falls into one of three tiers. The design rule: **lead with Tier 1, corroborate with Tier 2, publicly reject Tier 3 as a verdict.**

### Tier 1 — Provenance / event signals (high confidence, signable)

Facts about what happened in the tracked editor. A signed credential can assert these directly.

| Signal | What it records | Source of confidence |
|---|---|---|
| **Keystroke presence vs. absence** per span | Whether characters arrived with corresponding key events | A block with no key events near-certainly wasn't typed *here* |
| **Paste / clipboard events + source** | Paste size, position, and origin (in-app browser clipboard vs. unidentifiable desktop/private source) | Observed OS/editor event |
| **AI-tool-origin events** | Text accepted from a recognized AI integration or pasted from a known AI tool | Logged integration event |
| **Edit history** | Typed-then-deleted, in-place reformulation, AI-assisted rephrase | Observed event stream |
| **Timestamps & focus/session changes** | When each span entered; session boundaries (idle expiry) | Local clock + RFC 3161 anchor |

This is the credential's backbone and is what the core already partially captures (`evidence_flags.large_unkeyed_insertions`, `process.keyed_fraction`).

**Hard limit (state it in every report):** *the trust boundary equals the capture surface.* Anything entering outside a tracked editor is **Unknown** — a first-class, non-penalizing band, never a failure. Grammarly's December-2025 "humanizer laundering" fix is the cautionary tale: same-tool paraphrasing was reported as "75% Typed by a Human" until relabeled.

### Tier 2 — Process-shape / behavioral signals (medium confidence, corroborative only)

From keystroke-dynamics and writing-process research. Real, but far weaker than Tier 1, and **must be banded honestly**.

- **Pause *location* > pause length** — genuine composition clusters longer pauses at clause/sentence boundaries (planning) and shorter pauses within words (motor/spelling).
- **Burst structure** — text bounded by a pause ≥ ~2 s or a revision. **P-bursts** (pause-terminated → planning) and **R-bursts** (revision-terminated → evaluation).
- **Revision / backspace ratio, insertion & deletion counts** — authentic drafting revises and deletes more; transcription shows long uninterrupted bursts.
- **Inter-key cadence** — dwell + flight times, digraph latencies; robust *combined*, noisy individually.

**Honest reliability:** Crossley et al. (EDM 2024) hit ~99% genuine-vs-transcribed — *but with paste disabled* (process shape, not paste interception). For inferring *humanness* specifically, Kundu et al. (2024) report only **52–86% accuracy, FAR/FRR 18–48%**. Free-text keystroke biometrics sit at ~5–12% EER. Timing is spoofable: Siahaan & Chowanda (2022) reconstructed plausible timing from a screen recording (15% FAR, up to 64% evasion). → **Use as soft corroboration of a human-like process; never as identity, never as an anti-AI guarantee.**

### Tier 3 — Text-intrinsic / stylometric (low confidence — reject as a verdict)

Perplexity, burstiness, log-rank/GLTR, DetectGPT curvature, stylometry. We do **not** ship these as a verdict mechanism, for the four reasons in §2. The only defensible text-intrinsic technology is **opt-in generation-time watermarking** (e.g. SynthID-Text) — provenance the model vendor controls, complementary to us, not a detector we run. If a future "inference band" is ever offered, it lives behind §6's calibration rules and a stated error rate.

## 4. Per-span provenance model

The unit of attribution is a **span**: a contiguous run of text with one provenance. Span *boundaries* are observed events (a paste, a focus change, a typing burst) → high confidence. Only a span's *class* can be uncertain.

```jsonc
// extends WritingSessionRecord.process (base spec §5) — content-free
"spans": [
  {
    "range": { "start": 0, "len": 412 },        // character offsets into final text
    "provenance": "typed",                        // typed | pasted | ai_tool | unknown
    "source": null,                               // for pasted/ai_tool: coarse origin tag, never the content
    "keystrokes": 480,
    "active_ms": 92000,
    "bursts": { "p": 6, "r": 3 },
    "edited_after": true                          // any in-place revision touched this span later
  }
]
```

Provenance classes (descriptive, never accusatory):
- `typed` — arrived via keystrokes in a tracked editor.
- `pasted` — arrived via paste; `source` carries a coarse origin (`browser-clipboard`, `unknown-app`) — **never the pasted text**.
- `ai_tool` — accepted from a recognized AI integration / pasted from a known AI tool.
- `unknown` — entered outside any tracked surface, or pre-dates capture.

Computing spans is **change-point detection** on the event stream (boundaries) followed by labeling (class). Because boundaries are observed events, only the label is probabilistic — and for Tier-1 signals it is effectively certain.

**Shipped (Phase 1, record schema `@0.2`):** `ProcessStats.spans` is an ordered list of `{ provenance, chars, keystrokes }`, built by merging consecutive insertions of the same class. It is **order-based, not offset-based** — the schema above's `range`/`source`/`edited_after`/`bursts` fields await positional capture (offsets) and richer origin signals from the adapters. The builder currently emits only `typed` and `pasted`; `ai_tool` and `unknown` spans require adapter signals not yet present (final-document `unknown` is derived in the report — §5).

## 5. Banded provenance report (the "Writing activity report")

The user-facing deliverable, modeled on Grammarly's report but corrected.

**Bands are word-count proportions of provenance, not confidence scores.** "73% of words were typed in this editor" is a measurement; "73% likely AI" is a guess. We only ever render the former.

A four-tier *nuance* summary (descriptive, inverted from Pangram's detection tiers):

| Summary band | Meaning |
|---|---|
| **Fully typed** | ~all words typed in a tracked editor with a human-like process |
| **Typed with pastes** | predominantly typed; some pasted spans (sources shown) |
| **Mostly pasted** | predominantly arrived by paste |
| **Unverified** | substantial **Unknown** content (written outside coverage) |

Rendering rules (honest-UX, from calibration & risk-communication research):
- **Color by *evidence*, not suspicion** — green = typed (with timing), amber = pasted (with source), grey = **Unknown**. No red "AI!" coloring; we don't make that claim.
- **Coarse buckets, ranges over false precision.** No spurious decimals ("73.4%").
- **"Unknown" is shown prominently and framed neutrally** — it means "not captured," not "suspicious."
- The credential **notarizes the recorded process** (C2PA framing); it never certifies the writer's honesty or that text "is human."

## 6. Honest scoring & uncertainty (only if/when we emit any inferred number)

The default report uses Tier-1 proportions and needs no probabilities. *If* a future opt-in band ever surfaces an inferred score, it must obey:

- **Calibration.** Raw scores are not probabilities (Guo et al. 2017). Apply Platt / temperature scaling (small data) or isotonic regression (>~1000 samples), and publish a **reliability diagram + ECE/Brier** as the honesty receipt.
- **Fusion.** Combine weak signals via logistic-regression stacking (default) or **Dempster–Shafer** evidence theory — the latter lets a low-evidence span say *"insufficient evidence"* instead of forcing a number.
- **Sequence labeling.** Span classes via change-point detection (boundaries, high confidence) + CRF/HMM (BIO) for the uncertain class.
- **Stated error rate, always.** Any inferred band ships with its measured FAR/FRR and a plain-language caveat. Never a headline percentage, never an accusation.

## 7. Replay & visualization (content-free by default)

The capture layer already produces an ordered event stream, so replay and process visualizations are nearly free. The tension: every compelling competitor replay (Draftback, Grammarly, Originality.ai) shows the *actual evolving text*, which conflicts with our metadata-only posture. Resolution, in priority order:

1. **Structural replay (built-in default).** Replay *geometry* — caret position, insert/delete lengths, paste extents, bursts/pauses — as moving blocks/heatmap on a timeline. Conveys rhythm and typed-vs-pasted **without glyphs**. Content-free; ships in the credential's trust story.
2. **Draftback-style "fingerprint" graph (built-in).** Position-vs-time scatter of every edit (vertical = document depth, horizontal = time). Needs only *position + timestamp* — fully content-free, and the single highest-value borrowed visual.
3. **Paste timeline with jump-to-paste markers** (from Revision History) — paste events/sizes/timestamps as a navigable strip.
4. **Opt-in, locally-held encrypted text log (author convenience).** Mirroring Grammarly's on-device AES-256-GCM model: the author may *locally* retain an encrypted cleartext log that drives a rich, glyph-level replay on *their own* machine. The credential still commits only to hashes (`replay.log_sha256`, base spec §7.1). Sharing it is the author's explicit, informed choice.
5. **Verifier-supplied text.** If a verifier already holds the final document, replay the timing/hash track *against* it to confirm consistency — text comes from the verifier, never the credential.

Replay adds **persuasiveness to a human viewer, not cryptographic strength** (copy-typed text replays as smooth human writing). Documented as such; never presented as proof.

## 8. Competitive landscape & what we borrow

Full per-vendor profiles are in the 2026-06-06 research catalog. The market splits into **inference-only detectors** (guess from words) and a growing **provenance wing** (capture how text was written). We are in the provenance wing — but uniquely open-source, local-only, content-free, and standards-based.

### 8.1 Feature matrix

| | Process capture | Replay | Per-span labels | Signed verifiable credential | Local-only / content-free | Open source | Honest framing |
|---|---|---|---|---|---|---|---|
| Grammarly Authorship | ✅ keystroke+clipboard | ✅ | ✅ source bands | ❌ (report) | ❌ cloud | ❌ | ⚠️ (laundering, since relabeled) |
| GPTZero (Docs/Origin) | ✅ | ✅ | ✅ | ❌ (badge/report) | ❌ | ❌ | ⚠️ still sells detection |
| Originality.ai | ⚠️ revision-history | ✅ | ✅ contributor % | ❌ | ❌ | ❌ | ✅ admits "can be tricked" |
| Turnitin Clarity | ✅ | ✅ | ⚠️ policy tags | ❌ | ❌ | ❌ | ✅ "insight, not verdict" |
| Copyleaks / Pangram / Winston | ❌ | ❌ | ✅ (inference) | ❌ | ❌ | ❌ | ❌ accuracy inflation |
| Draftback / Revision History / Brisk | ⚠️ revision-history | ✅ | ⚠️ typed/paste | ❌ | ~ client-side | ❌ | ✅ neutral |
| OKhuman | ✅ **content-free** | ✅ stamp | ⚠️ involvement | ⚠️ portable stamp | ❌ server-side | ❌ | ✅ "effort, not words" |
| TypeOS Authorship | ✅ | ⚠️ | ⚠️ +stylometry | ✅ certificate | ❌ | ❌ | ⚠️ stores stylometry |
| **humanshipd** | ✅ **keystroke-true** | ✅ structural + opt-in | ✅ provenance | ✅ **C2PA** | ✅ **both** | ✅ | ✅ published threat model |

**The architectural differentiator:** Originality.ai and Turnitin Clarity rely on Google Docs *revision history*, which Originality.ai concedes "could be tricked." A local, keystroke-true, *signed* event log is a structural advantage, not a claim. And no shipping product is simultaneously open-source + local-only + content-free + standards-based — that corner is empty (GitHub returns zero repos for "proof of human writing" / "human authorship attestation").

### 8.2 Features worth borrowing (and how, honestly)

1. **Per-span color-coded report — colored by *evidence*, not suspicion.** (Copyleaks/GPTZero/Grammarly → §5)
2. **Keystroke-true replay** as the centerpiece, beating revision-history tools. (vs Originality.ai/Turnitin → §7)
3. **Position-vs-time "fingerprint" graph** — content-free, highest-value visual. (Draftback → §7)
4. **Paste timeline with jump-to-paste markers.** (Revision History → §7)
5. **Speed control + scrubber + data-based (not video) replay.** (ScriptLog/Brisk's up-to-60× → §7)
6. **Four-tier nuance, inverted to provenance.** (Pangram → §5)
7. **Per-contributor attribution** for collaborative docs. (GPTZero → roadmap)
8. **Citation helper for pasted-from-source spans** — when a span is `pasted` with a known source, offer a preformatted citation (APA/MLA/Chicago). (Grammarly Pro "credit your sources" → roadmap)
9. **Shareable signed artifact + downloadable PDF report** — but trust-by-cryptography, not trust-by-brand. (Winston/GPTZero → roadmap)
10. **Public per-credential verification page.** (OKhuman → already our verify page; add a shareable view)
11. **Consent / opt-out as a design value** — "the writer controls disclosure," baked into the credential. (Turnitin Clarity → already our value; make explicit)
12. **"Certificate of authorship" deliverable framing.** (TypeOS → naming/UX)

### 8.3 Positioning

humanshipd is the **trustworthy, auditable evidence layer** that bodies like the Authors Guild (whose "Human Authored" mark is honor-system self-attestation) and newsrooms (the Chicago Public Media freelancer-ChatGPT incident) can plug into. OKhuman is closest in spirit but closed; the ZK-PoP arXiv paper (arXiv:2603.00179) describes our thesis but ships no code. We are the open, standards-based implementation of an idea everyone is circling.

## 9. Standards alignment (corrections & additions)

Research refined three points for the base spec's §9:

- **The "human, non-generative" signal is IPTC `digitalCreation`**, carried *through* C2PA — not a CAWG assertion, and there is **no standalone "no generative AI used" assertion** anywhere. Emit `digitalCreation` (digitalSourceType) as our positive baseline; optionally add the **CAWG Training & Data Mining Assertion v1.1** to declare an AI-usage stance.
- **C2PA now specifies unstructured-text manifest embedding** ("Embedding Manifests into Unstructured Text," C2PA spec ≥ v2.3, 2026-01-05) — align our in-text Unicode embedding to the standardized appendix rather than a bespoke scheme. *(Confirm exact appendix letter against the targeted spec version; it has shifted across 2.2→2.4 drafts.)*
- **CAWG Identity Assertion v1.2** (incl. the identity-claims-aggregator path) is the mechanism to bind a verified author name — the "named-author identity" roadmap item.
- Unchanged and confirmed: **ISCC (ISO 24138:2024)** as an authoritative C2PA soft binding (similarity-preserving, text-capable), and **C2PA Durable Content Credentials** (hard + soft binding + registry recovery).

## 10. Roadmap (signals & reporting)

Phased so each phase ships something verifiable on its own. Phases 0–1 are the honest core; later phases add borrowed polish.

**Phase 0 — Provenance backbone (largely built).** Tier-1 capture: keyed-fraction, large un-keyed insertions, timing. Single honest claim string. → *core today.*

**Phase 1 — Per-span provenance + banded report. ✅ shipped (core).**
- ✅ `WritingSessionRecord.process.spans[]` (order-based; schema `@0.2`) built in `build_record` (§4).
- ✅ `render_report()` → word-count proportion bands + four-tier nuance summary, with final-document `unknown` derived (§5).
- ✅ Emit IPTC `digitalCreation` (corrected from `digitalCapture`) in the C2PA manifest (§9).
- ✅ Surfaced in the verify page (banded report rendered from the validated record; shown only when valid).
- *Remaining:* surface the report in the extension popup; offset-based spans + `source`/`ai_tool` once adapters emit position and origin.

**Phase 2 — Content-free visualization. ✅ shipped.**
- ✅ Writing fingerprint graph: a content-free `timeline` (schema `@0.4`) derived in `build_record`, rendered on the verify page as an SVG (§7.1–7.2).
- ✅ Per-edit caret offsets captured through the pipeline (`EditEvent.at_offset` → `TimelinePoint.offset`): extension via `selectionStart`, macOS via text prefix-diff, native-host DTO passthrough. The graph plots true **edit position over time** when offsets are present (revisits dip back down), else falls back to length.
- ✅ Paste timeline with jump-to-paste markers.
- ✅ Scrubber + speed-control structural replay (progressive reveal; content-free geometry).
- *Remaining (later):* per-contributor attribution; opt-in glyph-level local replay (§7.4).

**Phase 3 — Process-shape corroboration (Tier 2, clearly bounded). ✅ shipped (positive-only).**
- ✅ `render_process_shape()` derives three content-free signals — planning pauses, revision activity, burst segmentation — from existing record stats. The assessment is only ever `IncrementalComposition` (weak positive corroboration) or `Inconclusive`; there is **no "looks like AI" verdict**, by design. Surfaced on the verify page as a muted, secondary panel with a loud caveat (weak, spoofable, 18–48% error; absence ≠ AI).
- *Resolved open question:* Phase 3 is worth shipping **only** in this positive-corroboration-only form — it can affirm a human-like process but must never imply AI from its absence.

**Phase 4 — Identity & sharing polish.**
- CAWG Identity Assertion v1.2 (named-author identity).
- Per-contributor attribution; shareable signed report view + downloadable PDF (trust-by-crypto); citation helper for `pasted` spans with known sources.
- Opt-in, locally-held encrypted text log for glyph-level local replay (§7.4).

**Research track (unscheduled).** Zero-knowledge process attestation (prove "process features fall in human range" without revealing timing) — the ZK-PoP direction, as a privacy upgrade.

## 11. Open questions

- Span change-point thresholds (paste-size minimum, burst gap) — privacy/signal tradeoff; reuse base spec §10's burst/pause bucketing decision.
- Exact wording of the four-tier nuance labels (must stay descriptive, never accusatory).
- ~~Whether Phase-3 process-shape corroboration is worth the false-positive risk at all~~ — **resolved:** shipped positive-corroboration-only (`IncrementalComposition` / `Inconclusive`, never an AI verdict).
- Coarse `source` taxonomy for pasted spans (how much origin detail without leaking context).
- Confirm the C2PA unstructured-text appendix letter for our targeted spec version (§9).
