# Research: AI-vs-Human Authorship Signals and Honest Probability Bands

- **Slug:** `2026-06-06-ai-authorship-signals-and-probability-bands`
- **Date:** 2026-06-06
- **Status:** complete
- **Triggered by:** User question — "research the statistics, events and signals we should use to estimate the probability of content being AI generated," prompted by Grammarly Authorship's "Writing activity report" banded percentages.
- **Informed:** Signal taxonomy and the provenance-vs-inference distinction for the design spec (§4 threat model, §5 record schema, a forthcoming "reporting / bands" section). Reinforces the existing decision to reject text-intrinsic AI detection as a verdict mechanism.

## Question

What statistics, events, and signals can a writing-process recorder use to characterize how a document was authored (human vs. AI), and how should those be combined and presented as percentage "bands" (à la Grammarly Authorship) **honestly** — without overclaiming a calibrated "probability this is AI"?

## Sources

### [Grammarly Authorship — product page](https://www.grammarly.com/authorship)
- **Authors / Org:** Grammarly Inc.
- **Type:** vendor doc — **primary**
- **Published:** ongoing · **Accessed:** 2026-06-06
- **Relevance:** high
- **What this contributed:** Establishes that Authorship is *process-provenance*, not text analysis. The reported bands are word-count proportions of captured events, not detector confidence scores.

### [About Grammarly Authorship — Support](https://support.grammarly.com/hc/en-us/articles/29548735595405-About-Grammarly-Authorship)
- **Authors / Org:** Grammarly Support
- **Type:** vendor doc — **primary**
- **Published:** ongoing · **Accessed:** 2026-06-06
- **Relevance:** high
- **What this contributed:** Documents the captured signals (keystroke presence/timing, paste/clipboard events, paste-source discrimination, accepted AI suggestions, edit history, 10-min session timing), the explicit "no stylometry" stance, the "Unknown" band, account/cloud requirement (~24h retention; 12 months if shared), and self-stated limitations (fast typing, desktop pastes, unrecognized AI tools, cross-device → Unknown).
- **Quoted:**
  > Authorship "does not analyse the contents of the text, words or structure."

### [How Grammarly Launders AI-Generated Content](https://www.plagiarismtoday.com/2025/11/06/how-grammarly-launders-ai-generated-content/) and [the Dec 2025 relabel](https://www.plagiarismtoday.com/2025/12/11/grammarly-updates-authorship-improves-labeling/)
- **Authors / Org:** Jonathan Bailey, Plagiarism Today
- **Type:** engineering/industry blog — **secondary (critical)**
- **Published:** 2025-11-06 / 2025-12-11 · **Accessed:** 2026-06-06
- **Relevance:** high
- **What this contributed:** Demonstrates the "trust boundary = capture surface" failure: AI text run through Grammarly's own Humanizer reported "75% Typed by a Human" until the Dec 2025 update reclassified Humanizer/paraphraser output as "Copied from a source." Text written outside the tracked editor shows "100% Unknown." Confirms the laundering and intermediate-editor evasion paths.

### [Plagiarism Detection Using Keystroke Logs (EDM 2024)](https://files.eric.ed.gov/fulltext/ED675615.pdf)
- **Authors / Org:** Crossley, Tian, Choi, Holmes, Morris
- **Type:** peer-reviewed paper — **primary**
- **Published:** 2024-07 · **Accessed:** 2026-06-06
- **Relevance:** high
- **What this contributed:** ~99% accuracy distinguishing genuine composition from transcription — **but with copy-paste disabled**, so the signal is process *shape* (pre-word/clause pauses, more insertions/deletions/revisions vs. long uninterrupted bursts), not paste interception. The honest ceiling for "drafting signature" signals.

### [Keystroke Dynamics Against Academic Dishonesty in the Age of LLMs (arXiv:2406.15335)](https://arxiv.org/html/2406.15335v1)
- **Authors / Org:** Kundu, Mehta, Kumar, Lal, Anand, Singh, Shah
- **Type:** peer-reviewed paper — **primary**
- **Published:** 2024-06-21 · **Accessed:** 2026-06-06
- **Relevance:** high
- **What this contributed:** The realistic reliability band for inferring *humanness* from typing process: **52–86% accuracy, FAR 17.6–47.5%, FRR 23.9–48.5%**. Also notes no defense against copy-then-retype. The number to quote for honesty.

### [Keystroke Dynamics: Concepts, Techniques, and Applications (arXiv:2303.04605)](https://arxiv.org/html/2303.04605v3)
- **Authors / Org:** Shadman, Wahab, Manno, Lukaszewski, Hou, Hussain
- **Type:** survey paper — **primary**
- **Published:** 2023 · **Accessed:** 2026-06-06
- **Relevance:** medium
- **What this contributed:** Feature robustness (dwell+flight, digraph/trigraph latencies robust; single features noisy) and EER bands — fixed-text ~0.75–10%, free-text ~5–12% — plus the degradation factors (keyboard change, cross-language ~14% loss, fatigue/stress) that make per-user biometric identity an overclaim for a real writing tool.

### [Spoofing keystroke dynamics from screen-recorded video (Journal of Big Data, 2022)](https://link.springer.com/article/10.1186/s40537-022-00662-8)
- **Authors / Org:** Siahaan & Chowanda
- **Type:** peer-reviewed paper — **primary**
- **Published:** 2022 · **Accessed:** 2026-06-06
- **Relevance:** medium
- **What this contributed:** Concrete spoof evidence — ~15% FAR and up to 64% evasion reconstructing timing from a screen recording, no malware. Supports treating timing as forgeable.

### [GPT detectors are biased against non-native English writers (arXiv:2304.02819)](https://arxiv.org/abs/2304.02819)
- **Authors / Org:** Liang, Yuksekgonul, Mao, Wu, Zou (Stanford)
- **Type:** peer-reviewed paper — **primary**
- **Published:** 2023-04-06 · **Accessed:** 2026-06-06
- **Relevance:** high
- **What this contributed:** The strongest case against text-intrinsic detection: avg **61.22% false-positive rate** on non-native TOEFL essays; 97.8% flagged by ≥1 detector; 19.78% by all seven; vs. 5.19% on US 8th-grade essays. Detectors target low perplexity / constrained vocabulary, not AI authorship.

### [Can AI-Generated Text be Reliably Detected? (arXiv:2303.11156, TMLR)](https://arxiv.org/abs/2303.11156)
- **Authors / Org:** Sadasivan, Kumar, Balasubramanian, Wang, Feizi
- **Type:** peer-reviewed paper — **primary**
- **Published:** 2023-03-17 (rev. 2025) · **Accessed:** 2026-06-06
- **Relevance:** high
- **What this contributed:** The information-theoretic ceiling: best-possible detector **AUROC ≤ ½ + TV(M,H) − TV(M,H)²/2**. As models approach human text distribution, TV → 0 and detection → coin flip. Detectability is fundamentally bounded, not an engineering gap. The cleanest public justification for attesting from process, not words.

### [OpenAI — New AI classifier for indicating AI-written text](https://openai.com/index/new-ai-classifier-for-indicating-ai-written-text/)
- **Authors / Org:** OpenAI
- **Type:** vendor announcement — **primary** (page 403'd to fetcher; figures corroborated across secondary coverage incl. [TechCrunch](https://techcrunch.com/2023/07/25/openai-scuttles-ai-written-text-detector-over-low-rate-of-accuracy/))
- **Published:** 2023-01-31 (retired 2023-07-20) · **Accessed:** 2026-06-06
- **Relevance:** high
- **What this contributed:** OpenAI's own classifier caught only **26% of AI text** and false-flagged **9% of human text**, and was retired "due to its low rate of accuracy." A vendor admitting its own detector doesn't work.

### [Scalable watermarking for identifying LLM outputs — SynthID-Text (Nature, 2024)](https://www.nature.com/articles/s41586-024-08025-4.pdf)
- **Authors / Org:** Dathathri et al., Google DeepMind
- **Type:** peer-reviewed paper — **primary**
- **Published:** 2024-10-24 · **Accessed:** 2026-06-06
- **Relevance:** medium
- **What this contributed:** The one defensible text-intrinsic technology — opt-in watermarking via Tournament Sampling — but authors concede vulnerability to meaning-preserving edits (paraphrase, back-translation). Complementary to process attestation, not competitive.

### [On Calibration of Modern Neural Networks (arXiv:1706.04599, ICML 2017)](https://arxiv.org/abs/1706.04599) + [scikit-learn: Probability calibration](https://scikit-learn.org/stable/modules/calibration.html)
- **Authors / Org:** Guo, Pleiss, Sun, Weinberger; scikit-learn
- **Type:** peer-reviewed paper + library doc — **primary**
- **Published:** 2017 / ongoing · **Accessed:** 2026-06-06
- **Relevance:** high
- **What this contributed:** Raw scores are overconfident and not probabilities; fixes are Platt/temperature scaling (small data) and isotonic regression (>~1000 samples). Measure with reliability diagrams, Brier score, ECE. Basis for "never display a raw score as a percentage."

### [Score-Level Multibiometric Fusion via Dempster–Shafer (IEEE 6932423)](https://ieeexplore.ieee.org/document/6932423/) + [biometric fusion benchmark (arXiv:2111.08703)](https://arxiv.org/pdf/2111.08703)
- **Type:** peer-reviewed — **primary**
- **Accessed:** 2026-06-06 · **Relevance:** medium
- **What this contributed:** Standard ways to fuse weak signals — naive-Bayes log-LR sum, logistic-regression stacking (workhorse), Dempster–Shafer (models "ignorance" explicitly, enabling a principled "insufficient evidence" state).

### [C2PA Technical Specification 2.1](https://spec.c2pa.org/specifications/specifications/2.1/index.html) + [CAI limits explainer](https://www.softwareseni.com/how-c2pa-content-credentials-work-and-what-their-limits-are/)
- **Type:** spec + secondary explainer
- **Accessed:** 2026-06-06 · **Relevance:** high
- **What this contributed:** Provenance is *notarization, not truth-verification* — a valid signature proves origin + integrity, not that content is real or non-AI. Directly shapes honest UX: attest the recorded process, never the writer's honesty.

## Synthesis

**The reframe.** "Probability the content is AI-generated" is the wrong target — it inherits every unfixable flaw of AI detectors (bias, evasion, no calibration, an information-theoretic ceiling). The right target is **provenance composition**: a word-count breakdown of *how each span verifiably entered the document*. Grammarly proved the model is commercially legible, and proved its honesty ceiling is exactly its capture surface (anything entering outside the tracked editor → "Unknown"; same-tool "humanizing" was a laundering hole until relabeled).

**Three signal tiers, by confidence:**

1. **Provenance / event signals (high confidence, signable).** Keystroke presence vs. absence per span; paste/clipboard events and paste-source; text accepted from a known AI tool; edit history (typed-then-deleted, AI-rephrase); focus changes; timestamps. These are *facts about what happened*, and are what a signed credential can honestly assert. This is what humanshipd already captures (`evidence_flags.large_unkeyed_insertions`).
2. **Process-shape / behavioral signals (medium confidence, corroborative only).** Pause distribution and *location* (planning pauses at clause/sentence boundaries), burst structure (P-bursts/R-bursts; pause ≥ 2 s), revision/backspace ratio, inter-key cadence plausibility. Strong at the *aggregate* (Crossley ~99% genuine-vs-transcribed, paste disabled); weak per-keystroke; for *humanness* specifically the honest band is FAR/FRR ~18–48% (Kundu). Spoofable by retyping and by video timing reconstruction. Use as soft corroboration, never as identity or an anti-AI guarantee.
3. **Text-intrinsic / stylometric signals (low confidence — reject as a verdict).** Perplexity, burstiness, log-rank/GLTR, DetectGPT curvature, stylometry. Uncalibrated, biased against non-native writers (61% FPR), trivially evaded by paraphrase, and provably capped (Sadasivan). OpenAI retired its own (26%/9%). The only defensible text-intrinsic tech is **opt-in watermarking** (SynthID-Text) — provenance-at-generation, complementary, not a detector.

**Combining + presenting honestly:**
- Boundaries between spans are *observed events* (a paste, a focus change) → high confidence; only the per-span *class* is uncertain. Frame as change-point detection (high confidence) + span labeling (CRF/BIO) for the uncertain class.
- If you ever emit a score, **calibrate it** (Platt/temperature/isotonic) and publish a reliability diagram; consider Dempster–Shafer so low-evidence spans can say "insufficient evidence" instead of forcing a number.
- **Bands = word-count proportions of provenance, not confidence scores** (mirror Grammarly). Make **"Unknown" a first-class, non-penalizing band**. Use coarse buckets, ranges over false precision, no spurious decimals.
- C2PA framing throughout: the credential *notarizes the recorded process*; it never certifies the writer's honesty or that text "is human."

**Bottom line for humanshipd:** Lead with provenance composition (high-confidence, verifiable, already partly captured), present process-shape as corroboration with stated error bands, and publicly reject text-intrinsic detection as a verdict mechanism — citing the bias study, OpenAI's retirement, and the Sadasivan bound as the justification.

## Downstream uses

- Design spec — extend §5 record schema toward a banded provenance report; reinforce §4 threat model with the retype/video-spoof/laundering attacks and the "Unknown" band.
- The existing "attests, it doesn't prove" framing in `README.md` and the spec is directly supported by the Sadasivan bound and the Liang bias study.
