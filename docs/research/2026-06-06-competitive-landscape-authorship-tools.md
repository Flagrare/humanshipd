# Research: Competitive Landscape of Authorship / Provenance / Detection Tools

- **Slug:** `2026-06-06-competitive-landscape-authorship-tools`
- **Date:** 2026-06-06
- **Status:** complete
- **Triggered by:** User directive — "research what other tools are doing and integrate the best parts of them into our spec roadmap."
- **Informed:** [`docs/superpowers/specs/2026-06-06-authorship-signals-and-reporting-design.md`](../superpowers/specs/2026-06-06-authorship-signals-and-reporting-design.md) §8 (feature matrix + borrowed features), §9 (standards alignment), §10 (roadmap).

## Question

Who else occupies the human-authorship attestation / AI-detection / writing-process-provenance space, what features do they offer, and which of those should humanshipd adopt — within its open-source, local-only, content-free, standards-based constraints?

## Sources

### AI-detection & integrity vendors

#### [Turnitin Clarity — UCLA DTS](https://dts.ucla.edu/initiatives/bruin-learn-center-excellence/academic-tech-tools/turnitin-clarity) + [AI detection FAQs](https://guides.turnitin.com/hc/en-us/articles/28477544839821-Turnitin-s-AI-writing-detection-capabilities-FAQs)
- **Org:** Turnitin · **Type:** vendor doc / institutional — **primary** · **Accessed:** 2026-06-06 · **Relevance:** high
- **What this contributed:** Clarity (2026 pilot) adds genuine process capture (paste detection, writing time, draft playback, AI-usage policy tags) and explicitly frames it as "insight into how your writing develops," not a verdict. Relies on revision history, cloud, institutional license. Borrow: assignment-level policy profiles; the process-insight-not-verdict framing.

#### [Announcing GPTZero Docs](https://gptzero.me/news/announcing-gptzero-docs-the-future-of-transparent-writing/) + [Best Writing Replay Tools](https://gptzero.me/news/best-writing-replay-tools/) + [GPTZero Authorship](https://gptzero.me/authorship)
- **Org:** GPTZero · **Type:** vendor — **primary** · **Accessed:** 2026-06-06 · **Relevance:** high
- **What this contributed:** Closest philosophical sibling; CEO reframing toward "verifying the process rather than the output." Writing Replay logs keystrokes/edits, paste detection, bursts, multi-editor contribution %, badges — but layers AI detection on large pastes and is cloud/Docs-bound. Borrow: per-editor attribution, activity insights, replay-as-trust-signal, shareable badge.

#### [Originality.ai Chrome Extension](https://originality.ai/chrome-extension) + ["watch a writer write" / "can be tricked" admission](https://originality.ai/blog/chatgpt-detection-chrome-extension-watch-writer)
- **Org:** Originality.ai · **Type:** vendor — **primary** · **Accessed:** 2026-06-06 · **Relevance:** high
- **What this contributed:** Char-by-char replay + paste-location overlay + "Human Typing Score." **Critically self-admits** the replay uses Google Docs *revision history, not keystrokes*, "cannot replay documents written before installation," and the approach "could be tricked" by typing AI output. This is the concrete justification for humanshipd's keystroke-true, signed-log differentiator.

#### [Copyleaks AI Content Detector](https://copyleaks.com/ai-content-detector), [Pangram third-party evals](https://www.pangram.com/blog/third-party-pangram-evals), [Winston AI review](https://www.tryleap.ai/review/winston-ai)
- **Type:** vendor (Copyleaks page 403'd; corroborated via secondary) / secondary reviews · **Accessed:** 2026-06-06 · **Relevance:** medium
- **What this contributed:** Inference-only detectors. Borrow honestly: Copyleaks' per-sentence color-coding that *explains the why*; Pangram's four-tier nuance (inverted to provenance: fully-typed/typed-with-pastes/mostly-pasted/unverified); Winston's downloadable PDF report artifact. All accuracy claims are vendor-controlled; treat skeptically.

### Writing-process & replay tools

#### [Draftback](https://draftback.com/) + [Somers, "How I reverse-engineered Google Docs"](https://features.jsomers.net/how-i-reverse-engineered-google-docs/)
- **Author:** James Somers · **Type:** tool + engineering blog — **primary** · **Accessed:** 2026-06-06 · **Relevance:** high
- **What this contributed:** The reference replay implementation. Reconstructs from a single mutation array with persistent character IDs; windowed rendering scales to tens of thousands of revisions; **position-vs-time revision graph** ("a visual fingerprint of a writer"). Runs fully in-browser. The fingerprint graph needs only position+timestamp → content-free, highest-value borrowed visual.

#### [Revision History](https://revisionhistory.com/) + [Brisk Teaching Inspect Writing](https://www.briskteaching.com/inspect-writing) + Grammarly Authorship replay
- **Type:** vendor — **primary** · **Accessed:** 2026-06-06 · **Relevance:** medium
- **What this contributed:** Revision History — paste/deletion moments highlighted with **jump-to icons**. Brisk — playback up to **60×**, typed-vs-pasted separation. Grammarly — chronological replay with **stopping points at each paste**, source names, excludable-on-share. Borrow: jump-to-paste markers, speed control, paste stopping-points, exclude-replay-on-share UX.

#### [Inputlog](https://www.inputlog.net/) + ScriptLog
- **Authors:** Leijten & Van Waes (Inputlog); Lund (ScriptLog) · **Type:** academic tools — **primary** · **Accessed:** 2026-06-06 · **Relevance:** medium
- **What this contributed:** The rigorous vocabulary — **P-bursts/R-bursts**, configurable pause thresholds (0/1/2/5 s) classified within/between words/sentences. Data-based (not video) replay with adjustable speed. All run on timing alone → content-free-compatible.

### Content-provenance ecosystem

#### [C2PA Specification 2.3](https://spec.c2pa.org/specifications/specifications/2.3/specs/_attachments/C2PA_Specification.pdf) + [2.4](https://spec.c2pa.org/specifications/specifications/2.4/specs/C2PA_Specification.html)
- **Org:** C2PA · **Type:** spec — **primary** · **Published:** 2.3 dated 2026-01-05 · **Accessed:** 2026-06-06 · **Relevance:** high
- **What this contributed:** C2PA now specifies **embedding manifests into unstructured text** (appendix; section letter shifted across 2.2→2.4 — confirm against targeted version). Align in-text embedding to the standard, don't hand-roll.

#### [CAWG specs](https://cawg.io/specs/) + [Identity Framework](https://cawg.io/about/identity-framework/)
- **Org:** Creator Assertions Working Group / DIF · **Type:** spec — **primary** · **Accessed:** 2026-06-06 · **Relevance:** high
- **What this contributed:** Identity Assertion v1.2 (X.509 + identity-claims-aggregator) binds *who*, not human-vs-AI. There is **no dedicated "created by a human" CAWG assertion**. Training & Data Mining Assertion v1.1 can declare AI-usage stance. Use for named-author identity (roadmap Phase 4).

#### [IPTC Digital Source Type vocabulary](https://cv.iptc.org/newscodes/digitalsourcetype/)
- **Org:** IPTC · **Type:** vocabulary spec — **primary** · **Accessed:** 2026-06-06 · **Relevance:** high
- **What this contributed:** **The actual "human, non-generative" signal is `digitalCreation`**, carried through C2PA — not a CAWG assertion. There is no standalone "no generative AI used" term; `digitalCreation` is the de-facto positive baseline. Emit it (spec §9).

#### [Durable Content Credentials](https://opensource.contentauthenticity.org/docs/durable-cr/) + [ISO 24138 ISCC](https://www.iso.org/standard/77899.html) + [Adobe Content Authenticity beta](https://blog.adobe.com/en/publish/2025/04/24/adobe-content-authenticity-now-public-beta-helps-creators-secure-attribution)
- **Type:** spec / vendor — **primary** · **Accessed:** 2026-06-06 · **Relevance:** medium
- **What this contributed:** Confirms ISCC (similarity-preserving, text-capable) as authoritative C2PA soft binding and the hard+soft+registry recovery model — already in the base spec. Adobe's app is the UX benchmark for "inspect credential." Truepic/Numbers Protocol/Starling Lab validate certificate-trust-list and registry/recovery patterns (Starling explicitly handles documents).

### Adjacent & open-source competitors

#### [OKhuman](https://okhuman.com/) + [Generative AI in the Newsroom profile](https://generative-ai-newsroom.com/this-tool-listens-to-you-type-to-prove-your-writing-is-human-da3350fe02e2)
- **Author:** OKhuman; Clare Spencer (profile) · **Type:** vendor + journalism — **primary/secondary** · **Accessed:** 2026-06-06 · **Relevance:** high
- **What this contributed:** Closest framing match — "understands your effort... **not your words**," content-free, portable stamp with public detail page, newsroom/academia wedge (post Chicago Public Media freelancer-ChatGPT incident). But proprietary, server-side, early-access. Borrow: "effort not words" framing, public per-stamp verification page.

#### [TypeOS Authorship](https://typeos.com/authorship) + [Authors Guild "Human Authored" cert](https://authorsguild.org/news/human-authored-certification-expands-to-all-authors/) + ZK-PoP (arXiv:2603.00179)
- **Type:** vendor / org / preprint — **primary** · **Accessed:** 2026-06-06 · **Relevance:** medium
- **What this contributed:** TypeOS — "certificate of authorship" framing (but Docs-only, stylometry, no named standard). Authors Guild mark — honor-system self-attestation + public database; humanshipd is the *evidence layer* it lacks (partnership, not rival). ZK-PoP paper describes humanshipd's exact thesis (content-free ZK process attestation, Rust) but **its cited repo 404s** — concept, not shipping competitor.

#### GitHub niche search (verified via `gh search repos`)
- **Type:** primary (negative result) · **Accessed:** 2026-06-06 · **Relevance:** high
- **What this contributed:** Zero repositories for "proof of human writing," "human authorship attestation," or "writing provenance keystroke." The open-source corner is empty — humanshipd's whitespace.

## Synthesis

The market splits into **inference-only detectors** (Copyleaks, Pangram, Winston — guess AI-ness from words, all subject to §2's bias/ceiling problems) and a growing **provenance wing** (Grammarly, GPTZero, Turnitin Clarity, Originality.ai, OKhuman, TypeOS — capture how text was written). humanshipd is in the provenance wing but is the **only** entrant that is simultaneously open-source, local-only, content-free, and standards-based. Two concrete differentiators emerged: (1) competitors' replay relies on Google Docs *revision history*, which Originality.ai admits is trickable — a **keystroke-true signed log** is an architectural advantage; (2) the open-source niche is literally empty.

Borrowed features, mapped to the roadmap: evidence-colored per-span report (§5); keystroke-true + structural replay and the Draftback fingerprint graph and paste-jump markers (§7); four-tier provenance nuance (§5); per-contributor attribution, citation helper, shareable signed PDF (Phase 4); `digitalCreation` + CAWG identity + C2PA text embedding (§9). Positioning: the auditable evidence layer that the Authors Guild's honor-system mark and newsrooms can plug into.

## Downstream uses

- Companion spec §8 (feature matrix + borrowed features), §9 (standards corrections: `digitalCreation`, C2PA text embedding, CAWG v1.2), §10 (phased roadmap).
- Reinforces base-spec §9 standards-adoption stance and the "attests, doesn't prove" framing.
