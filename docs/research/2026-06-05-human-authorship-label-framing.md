# Research: Positive vs. Negative Framing of a Human-Authorship Label

- **Slug:** `2026-06-05-human-authorship-label-framing`
- **Date:** 2026-06-05
- **Status:** complete
- **Triggered by:** Founder deciding how to name the claim/badge — negative/exclusionary ("AI-Free", "No AI") vs positive/affirmative ("Human Authored", "Human Generated", "Human Written") — and the product name ("Authorshipped"/"Authorshipd").
- **Informed:** Naming/positioning recommendation in chat (2026-06-05) and the forthcoming design spec. Companion feasibility research: [`2026-06-05-proof-of-human-authorship-feasibility.md`](./2026-06-05-proof-of-human-authorship-feasibility.md).

## Question

For a tool that substantiates human authorship of text, which label framing carries more weight, is more defensible, and performs better: a negative/exclusionary claim ("AI-Free") or a positive/affirmative one ("Human Authored")? Which specific term is strongest? And do product-naming choices (burying "human", dropped-vowel wordmarks) help or hurt a *trust* product specifically?

## Sources

### Existing human-authorship initiatives (terminology already converging)
#### [Authors Guild "Human Authored" Certification](https://authorsguild.org/human-authored/) ([FAQ](https://authorsguild.org/human-authored/faq/))
- **Authors / Org:** The Authors Guild
- **Type:** industry/vendor doc (primary)
- **Published:** 2025 (expanded to all US authors Mar 2026) · **Accessed:** 2026-06-05
- **Relevance:** high (the incumbent term)
- **What this contributed:** The de-facto category standard is the *affirmative* "Human Authored" (3,000+ authors, 5,000+ titles; public registry; tagline "Certifying Human Creativity in an AI World"). The FAQ explicitly explains choosing an affirmative frame over labeling AI content, with a de-minimis allowance (grammar/research/brainstorming). The most credible incumbent deliberately rejected a negative frame.

#### [Not By AI badge — About](https://notbyai.fyi/about)
- **Authors / Org:** Not By AI
- **Type:** vendor doc
- **Published:** 2023 · **Accessed:** 2026-06-05
- **Relevance:** high (even the negative-named project pivots positive)
- **What this contributed:** The most negatively-named initiative is self-certified, uses a fuzzy "90% human" rule, has no legal effect, and explicitly repositions as "pro-human, not anti-AI." Evidence that even negative-named projects gravitate to affirmative framing and lack defensibility.

#### [C2PA / Content Credentials — what is it](https://c2pa.ai/what-is-c2pa) & [Explainer 2.4](https://spec.c2pa.org/specifications/specifications/2.4/explainer/Explainer.html)
- **Authors / Org:** C2PA / Content Authenticity Initiative
- **Type:** standards/vendor doc
- **Published:** 2025–2026 · **Accessed:** 2026-06-05
- **Relevance:** medium-high
- **What this contributed:** The serious provenance infrastructure uses neutral "how it was authored" assertions and flags AI via `digitalSourceType` — not an absolute "AI-Free" verdict. Shows where an affirmative attestation should sit relative to the disclosure regime.

### Precedent labels — which framing won each market
#### [Cruelty-Free / Not Tested on Animals (FDA)](https://www.fda.gov/cosmetics/cosmetics-labeling-claims/cruelty-freenot-tested-animals) & [Ethical Consumer guide](https://www.ethicalconsumer.org/health-beauty/guide-cruelty-free-animal-testing-certification)
- **Authors / Org:** US FDA; Ethical Consumer
- **Type:** regulator; industry analysis
- **Published:** ongoing · **Accessed:** 2026-06-05
- **Relevance:** high
- **What this contributed:** "Cruelty-Free"/"not tested on animals" are unregulated and definable at will, so the *positive certification* (Leaping Bunny) — not the bare negative phrase — carries trust. Positive certification beat the negative descriptor.

#### [Non-GMO Project](https://www.nongmoproject.org/) & [critique](https://geneticliteracyproject.org/2019/08/16/viewpoint-why-the-non-gmo-project-label-is-little-more-than-a-marketing-tool-that-deceives-consumers/)
- **Authors / Org:** Non-GMO Project; Genetic Literacy Project
- **Type:** vendor/certifier; industry/opinion analysis
- **Published:** ongoing / 2019 · **Accessed:** 2026-06-05
- **Relevance:** high (closest analog to AI-Free vs Human-Authored)
- **What this contributed:** The market chose "Non-GMO **Verified**" (affirmative verification of a process) over "GMO-Free" — deliberately, because "GMO-free" is an unprovable absolute (the standard is actually a <1% threshold, so products are not literally "GMO-free"). The direct precedent for choosing verifiable affirmation over an indefensible absolute negative.

#### [Made in USA Labeling Rule (Federal Register)](https://www.federalregister.gov/documents/2021/07/14/2021-14610/made-in-usa-labeling-rule) & [MSU consumer-confusion study](https://msutoday.msu.edu/news/2025/08/msu-study-reveals-consumer-confusion-over-made-in-usa-labels)
- **Authors / Org:** US FTC; Michigan State University
- **Type:** regulator/legal; academic/news
- **Published:** 2021 / 2025 · **Accessed:** 2026-06-05
- **Relevance:** medium-high (cautionary tale)
- **What this contributed:** Even a *positive* claim becomes a liability when it implies an absolute: "Made in USA" is FTC-regulated under an "all or virtually all" standard with penalties up to ~$43,280/violation, and is widely misunderstood. Argues for a *process-attestation* framing over any absolute.

### Consumer psychology — positive vs. absence framing
#### [Making sense of the "clean label" trends](https://www.flandersfood.com/sites/default/files/) (Asioli et al., *Food Research International*)
- **Authors / Org:** Asioli et al.
- **Type:** academic
- **Published:** 2017 · **Accessed:** 2026-06-05
- **Relevance:** high
- **What this contributed:** "Free-from"/clean-label demand is driven by avoidance/prevention motivation, not gain-seeking — a weaker, fear-based footing for a trust brand than affirmative identity.

#### [Clean labeling: presence of benefits or absence of detriments?](https://www.sciencedirect.com/science/article/abs/pii/S0969698921004598) (*J. Retailing and Consumer Services*)
- **Type:** academic · **Published:** 2022 · **Accessed:** 2026-06-05 · **Relevance:** high (the nuance / dissent)
- **What this contributed:** The honest counterpoint — absence-framed claims can *outperform* in health/environment contexts. Absence framing has situational pull; the recommendation still favors affirmation on credibility/defensibility grounds, but the dissent is recorded.

#### [Opposing effects of sugar-free claims on willingness to pay](https://www.ncbi.nlm.nih.gov/pmc/articles/PMC12616639/) & [Framing Effect in Marketing](https://www.leadalchemists.com/marketing-psychology/framing-effect/)
- **Type:** academic; industry/marketing analysis · **Published:** 2025 / unknown · **Accessed:** 2026-06-05 · **Relevance:** medium-high
- **What this contributed:** A "free-from" claim can depress perceived quality and willingness-to-pay — a credibility penalty a trust product cannot absorb. Positive framing of the same fact ("95% fat-free") is generally preferred and engages reward rather than threat processing.

### Legal / advertising-standards — the decisive argument
#### [FTC: Are your "all natural" claims accurate?](https://www.ftc.gov/business-guidance/blog/2016/04/are-your-all-natural-claims-all-accurate) & [FTC Advertising Substantiation Policy](https://www.ftc.gov/legal-library/browse/ftc-policy-statement-regarding-advertising-substantiation)
- **Authors / Org:** US FTC
- **Type:** regulator/legal (primary)
- **Published:** 2016 / 1984 · **Accessed:** 2026-06-05
- **Relevance:** high
- **What this contributed:** The FTC reads absolute claims ("all natural", "100%") at face value even when qualified and requires a reasonable basis before dissemination — so "AI-Free" (an absolute negative about the entire universe of how text came to be) carries the heaviest substantiation burden and is effectively unfalsifiable/indefensible if challenged.

#### [ASA/CAP Substantiation](https://www.asa.org.uk/advice-online/substantiation.html) & [ASA ruling on 'free-from' claims](https://www.ctpa.org.uk/news/asa-ruling-on-free-from-claims-7003) & [Debevoise: "natural"/"free-of" claims](https://www.debevoise.com/-/media/files/insights/publications/2021/07/20210721-the-nature-of-natural-advertising-claims.pdf)
- **Authors / Org:** UK ASA/CAP; CTPA (reporting ASA); Debevoise & Plimpton
- **Type:** regulator/legal (CTPA page returned 403 — substance via ASA/CAP pages)
- **Published:** ongoing / 2021 · **Accessed:** 2026-06-05
- **Relevance:** high
- **What this contributed:** UK standard: objective claims must be evidenced or are deemed misleading, judged by consumer interpretation; "free-from" claims are often read as *comparative/denigrating* ("our rivals use AI"), raising the bar further and adding class-action/regulatory exposure — exactly the risk "AI-Free" would carry. Affirmative act-claims ("Human Authored") are about what the author *did* and are far easier to stand behind.

#### [New 'Human Authored' authenticity mark (Transparency Coalition)](https://www.transparencycoalition.ai/news/new-human-authored-authenticity-mark-launched-by-authors-guild) & ["It was 80% me, 20% AI" (arXiv)](https://arxiv.org/pdf/2411.13032)
- **Authors / Org:** Transparency Coalition; arXiv (academic)
- **Type:** industry/advocacy news; academic preprint
- **Published:** 2025 / 2024 · **Accessed:** 2026-06-05
- **Relevance:** medium
- **What this contributed:** Frames "Human Authored" within the AI-transparency movement; and shows authorship is a *spectrum*, not binary — supporting an affirmative, de-minimis-tolerant claim over an absolute "AI-free."

## Synthesis

**Recommendation: use a POSITIVE/affirmative frame anchored on the word "human." Strongest term: "Human Authored" (with "Human Written" a close plain-language second). Avoid "AI-Free"/"No AI" as the primary claim, and avoid "Human Generated" entirely — "generated" is the signature verb of AI ("AI-generated", "GenAI") and semantically collides with the very thing being distinguished.** Four converging lines of evidence:

1. **Precedent:** In every mature trust-label market the durable, ownable brand is the positive one (Cruelty-Free, Non-GMO Verified, Organic, Fair Trade); the negative phrasing survives only as a regulated, narrowly-defined, litigated descriptor. The Non-GMO Project is the closest analog and deliberately chose verifiable affirmation over the unprovable "GMO-Free."
2. **Psychology:** Absence/"free-from" framing rests on avoidance/fear, can depress perceived value, and tethers brand salience to the enemy (AI). Positive framing foregrounds the value being sold (human craft) and engages reward processing.
3. **Legal (decisive):** "AI-Free" is the exact class of absolute negative claim regulators treat most harshly (FTC face-value reading; ASA misleading/denigration risk) and is structurally unprovable. "Human Authored" is an affirmative attestation about what the author did — certifiable and defensible. This mirrors why both the Non-GMO Project and the Authors Guild chose affirmation.
4. **The category is already converging** on "Human Authored"/"human-made" — even the most negatively-named initiative (Not By AI) repositions to "pro-human, not anti-AI." Adopting the affirmative aligns with the emerging standard and SEO term.

**Product-name guidance:** For a *trust* product, clarity and credibility beat cleverness. Do **not** bury "human" — it is the entire brand equity, the scannable cue, and the term the category is coalescing around. **Avoid dropped-vowel / "cute" wordmarks** (the "Authorshipd" form): misspelled marks read as startup-playful and quietly erode the authority a certification needs — every credible trust mark (Leaping Bunny, Non-GMO Project, Fair Trade, USDA Organic, Authors Guild) uses plain, literal, authoritative language. "Authorshipped" is more legible than "Authorshipd" but still (a) buries "human" and (b) the "-shipped" pun leans playful for a credential. A safer pattern: a literal, human-forward name + an affirmative certification phrase ("Human Authored" / "Certified Human Authored"), with "AI" kept out of the mark and referenced only in explanatory copy.

## Downstream uses

- Naming/positioning recommendation delivered to founder (chat), 2026-06-05.
- Design spec (claim wording "Human Authored", §4/§7; naming §10): [`docs/superpowers/specs/2026-06-05-human-authorship-attestation-design.md`](../superpowers/specs/2026-06-05-human-authorship-attestation-design.md).
