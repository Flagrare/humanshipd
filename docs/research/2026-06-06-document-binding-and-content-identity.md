# Research: Binding a Credential to a Formatted Document, and Content Identity Across Formats

- **Slug:** `2026-06-06-document-binding-and-content-identity`
- **Date:** 2026-06-06
- **Status:** complete
- **Triggered by:** Pre-refactor decision cluster — the "document shape / retain file formats" question (how a credential binds to a real `.docx`/PDF/Google Doc, and survives cross-format), and "what does a reader verify against."
- **Informed:** The binding/artifact-model decisions for the v1 architecture brainstorm; content-binding & capture-fidelity spec.

## Question

How do world-class systems bind a signature/credential to a *formatted* document while retaining its format and detecting tampering — and how is "the same writing" identified when it moves across formats/editions?

## Sources

### Document digital-signature standards (byte-range binding inside the file)
- **[ETSI EN 319 142-1 (PAdES)](https://www.etsi.org/deliver/etsi_en/319100_319199/31914201/01.02.01_60/en_31914201v010201p.pdf)**, **[MS-OFFCRYPTO XAdES](https://learn.microsoft.com/en-us/openspecs/office_file_formats/ms-offcrypto/341ca65a-95b8-439b-a5a0-d6868b828854)**, **[ODF 1.3 signatures](https://blog.documentfoundation.org/blog/2025/08/01/whats-new-in-odf-1-3-and-1-4/)** — specs/vendor — **primary** — accessed 2026-06-06 — high
- **What they contributed:** PDF signatures hash a `/ByteRange` covering the whole file except the signature placeholder, embedded as an *incremental update* (original bytes never move); `DocMDP` controls which later edits are allowed. OOXML/.docx and ODF are ZIP/OPC packages signed with XML-DSig+XAdES over *canonicalized package parts* (more repackage-resilient, still container-bound). Common pattern: hash a precisely-defined region, embed the signature inside the file, append-only.

### C2PA in documents + Durable Content Credentials
- **[C2PA Spec 2.2](https://spec.c2pa.org/specifications/specifications/2.2/specs/_attachments/C2PA_Specification.pdf)** / **[2.4](https://spec.c2pa.org/specifications/specifications/2.4/specs/C2PA_Specification.html)**, **[Durable Content Credentials](https://opensource.contentauthenticity.org/docs/durable-cr/)**, **[C2PA Soft Binding API](https://spec.c2pa.org/specifications/specifications/2.2/softbinding/Decoupled.html)** — **primary** — accessed 2026-06-06 — high
- **What they contributed:** C2PA hard-binds via a **data-hash assertion** (byte ranges + exclusions for the manifest), and since spec **1.4 can embed a manifest into ZIP-based formats (EPUB, OOXML, ODF)** and PDF (format-specific appendices; `brob` box). **Durable CC** = hard binding + **soft bindings** (perceptual fingerprint and/or watermark) + a **manifest repository** reachable via the Soft Binding Resolution API — so a stripped/re-exported doc resolves back to its credential. This is the model to mirror.

### Content identity that survives format/edition change
- **[ISCC / ISO 24138:2024](https://iscc.codes/)** ([Text Content-Code algorithm](https://core.iscc.codes/units/content/code_content_text/)) — **primary** — accessed 2026-06-06 — high
- **What it contributed:** A *standardized, content-derived* code in four UNITs: **Instance-Code** (byte-exact checksum), **Data-Code** (raw-byte similarity), **Content-Code** (perceptual/content similarity), **Meta-Code**. The **Text Content-Code** normalizes away exactly what changes across formats (whitespace, punctuation, case, encoding via NFD/case-fold/NFKC), then 13-char n-grams → XXH32 → MinHash256 → 64-bit soft hash; similarity = Hamming distance / NPHD. The only standard answer to "same writing, different format."
- **[Winnowing (Schleimer/Wilkerson/Aiken, SIGMOD 2003)](https://theory.stanford.edu/~aiken/publications/papers/sigmod03.pdf)** — **primary** — accessed 2026-06-06 — high
- **What it contributed:** The canonical document-fingerprinting algorithm (behind MOSS/Turnitin-lineage): a **format-specific normalizer front-end** feeding a format-agnostic hashing engine (k-grams → window-min selection); guarantees detection of shared substrings ≥ `t=w+k−1`; two thresholds (noise `k`, guarantee `t`); positional fingerprints enable *partial/containment* matching. Lesson: separate the format front-end from the hashing core.
- **[SWHID](https://docs.softwareheritage.org/devel/swh-model/persistent-identifiers.html)** / **[IPFS CIDs](https://docs.ipfs.tech/concepts/content-addressing/)** / **[FRBR-WEMI (IFLA/LoC)](https://www.loc.gov/catdir/cpso/frbreng.pdf)** — **primary** — accessed 2026-06-06 — medium
- **What they contributed:** Byte-intrinsic IDs (Git/SWHID/IPFS) prove the *negative* — any byte change → different ID, so they can't anchor cross-format. FRBR/WEMI gives the conceptual frame: one **Work** → **Expressions** (revisions) → **Manifestations** (formats/editions). `.txt`→`.docx`→`.pdf` are manifestations of one expression — the credential should anchor at the expression/content level.
- **MinHash vs SimHash / fuzzy hashing (ssdeep, TLSH, Nilsimsa)** — secondary — accessed 2026-06-06 — medium — MinHash-over-shingles (what ISCC uses) is the strongest fit for reformatted prose; forensic fuzzy hashes are weaker for this.

## Synthesis

The world's best systems **bind to bytes inside the file** (PAdES ByteRange, OOXML/ODF canonicalized parts, C2PA data-hash) — crisp and tamper-evident, but they **break on any format change**, so the universal practice is **sign/seal as a discrete final step on the finished export, and re-sign per format**. The *only* thing that survives a format change is a **soft/content binding** (perceptual fingerprint + repository), which is deliberately fuzzy.

For humanshipd this resolves the "retain format" question into a **layered, two-phase model**:
1. **Capture during authoring** → a content-free writing-process record (the attestation).
2. **Seal at export** → emit a **C2PA manifest** carrying that record as an assertion, **hard-bound** to the exported artifact (embed into PDF / ZIP-based `.docx`/`.epub`/`.odf`; sidecar for plain text / Google Docs), **plus** content identifiers in the assertion: an **ISCC Instance-Code** (byte-exact "this file") *and* an **ISCC Content-Code** (similarity, "same writing across formats"). Optionally a winnowing fingerprint set for partial/containment matches.
3. **Verify in three honest tiers:** *Exact file* (Instance-Code/hard binding) → *Same writing* (Content-Code within a stated Hamming threshold) → *needs review* (borderline) → *no match*. Never false precision; state the threshold.

Net: don't reinvent embedding (emit C2PA), adopt ISCC for content identity (it's the ISO standard), and accept the crisp↔fuzzy gradient as a feature presented honestly.

## Downstream uses

- v1 architecture brainstorm — the binding/artifact model (two-phase capture-then-seal; layered hard+content binding; per-format embedding).
- Content-binding & capture-fidelity spec — concretizes its tiered-verification proposal with ISCC + winnowing + C2PA embedding.
