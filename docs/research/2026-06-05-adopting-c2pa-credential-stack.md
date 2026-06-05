# Research: Adopting C2PA + Standards for the Credential Stack

- **Slug:** `2026-06-05-adopting-c2pa-credential-stack`
- **Date:** 2026-06-05
- **Status:** complete
- **Triggered by:** Founder directive to adopt industry standards/OSS rather than reinvent — after a bespoke `Badge`/`TimestampToken`/Ed25519 envelope was built despite earlier research recommending C2PA + CAWG + VC.
- **Informed:** Refactor of `core` to emit C2PA manifests via c2pa-rs; design spec §6/§7/§9. Companion (capture tooling, not cataloged here per skill): decision recorded in spec §9. Builds on [`2026-06-05-proof-of-human-authorship-feasibility.md`](./2026-06-05-proof-of-human-authorship-feasibility.md).

## Question

Which concrete standards and maintained Rust SDKs should humanshipd build the credential on, and should the bespoke `Badge` envelope (custom JSON + Ed25519 signature + custom timestamp token) be replaced by a C2PA manifest? What stays bespoke?

## Sources

### [c2pa-rs (official CAI Rust SDK)](https://github.com/contentauth/c2pa-rs) · [crate docs](https://docs.rs/c2pa/latest/c2pa/) · [crates.io `c2pa` 0.85.2](https://crates.io/crates/c2pa)
- **Authors / Org:** Content Authenticity Initiative (Adobe)
- **Type:** crate / repo / docs (primary)
- **Published:** 0.85.2, 2026-06-03 (actively maintained, ~9.1M downloads) · **Accessed:** 2026-06-05
- **Relevance:** high (the envelope/signing/verify to adopt)
- **What this contributed:** The official SDK that replaces the bespoke envelope. `Builder.add_assertion("org.humanshipd.process.v1", &record)` carries our process metadata as a **custom assertion**; the COSE/X.509 claim signature (ed25519 supported) **is** the integrity layer; `Reader` validates signature + hashes + timestamp + identity in one call. Implements both C2PA and the CAWG identity assertion.

### [c2pa-rs supported-formats](https://github.com/contentauth/c2pa-rs/blob/main/docs/supported-formats.md) · [data_hash example](https://github.com/contentauth/c2pa-rs/blob/main/sdk/examples/data_hash.rs)
- **Authors / Org:** CAI
- **Type:** repo docs / source (primary)
- **Accessed:** 2026-06-05 · **Relevance:** high (the text gap + the workaround)
- **What this contributed:** Critical gap — c2pa-rs has **no native `text/plain` handler**. The adoption path for arbitrary text is a **detached `.c2pa` sidecar** via the DataHash workflow or `set_no_embed(true)` + remote URL, hashing the final text (SHA-256). This matches humanshipd's "store only the hash, never content" model exactly.

### [CAWG Identity Assertion 1.0](https://cawg.io/identity/1.0/) · [CAI CAWG docs](https://opensource.contentauthenticity.org/docs/rust-sdk/docs/cawg-id/) · [cawg signing settings fixture](https://github.com/contentauth/c2pa-rs/blob/main/sdk/tests/fixtures/test_settings_with_cawg_signing.toml)
- **Authors / Org:** Creator Assertions Working Group; CAI
- **Type:** spec / docs / config (primary)
- **Accessed:** 2026-06-05 · **Relevance:** high (the human-identity binding)
- **What this contributed:** The standalone `cawg-identity` crate is **discontinued — folded into `c2pa`** (`c2pa::identity`). A `[cawg_x509_signer.local]` settings block auto-emits a `cawg.identity` assertion binding a named human actor (`cawg.creator`) to our custom assertion via `referenced_assertions`. **No human-vs-AI role exists**; "human-authored" semantics map to `digitalSourceType=digitalCapture` + a `cawg.training-mining` assertion + our custom record as evidence — so the *claim semantics* stay bespoke.

### [ssi (SpruceID) 0.16.0](https://crates.io/crates/ssi)
- **Authors / Org:** SpruceID
- **Type:** crate (primary)
- **Published:** 0.16.0, 2026-04-16 · **Accessed:** 2026-06-05 · **Relevance:** medium (defer)
- **What this contributed:** The Rust W3C Verifiable Credentials library. Integrates into C2PA via CAWG `identity_claims_aggregation` — but the CAWG VC-integration section is explicitly *"undergoing significant exploration"* and **only `cawg.x509.cose` is operational today.** → W3C VC is **deferred / optional**, not core.

### [RFC 3161](https://www.rfc-editor.org/rfc/rfc3161.html) · [RustCrypto `cms`](https://docs.rs/cms/latest/cms/)
- **Authors / Org:** IETF; RustCrypto
- **Type:** spec / crate
- **Accessed:** 2026-06-05 · **Relevance:** high (timestamp — use C2PA's built-in)
- **What this contributed:** **No `rfc3161-client` Rust crate exists** (that's PyPI). But c2pa-rs has **built-in RFC 3161 timestamping** via `tsa_url` in signer settings, embedding the token in the COSE signature and verifying it in `Reader`. → **Drop the bespoke timestamp token; use c2pa-rs `tsa_url`.** Standalone fallback if ever needed: `cms` + `der` + `x509-cert`.

### [Sigstore Rekor — `sigstore-rekor` 0.8.0](https://crates.io/crates/sigstore-rekor) · [Rekor overview](https://docs.sigstore.dev/logging/overview/) · [Rekor v2 GA](https://blog.sigstore.dev/rekor-v2-ga/)
- **Authors / Org:** Sigstore / OpenSSF
- **Type:** crate / docs / blog
- **Accessed:** 2026-06-05 · **Relevance:** medium (optional)
- **What this contributed:** Append-only transparency log; `HashedRekordV2` fits anchoring the final-text hash + signature. Pre-1.0. **Keep strictly opt-in** — it requires a network call, against the local-only/zero-telemetry default.

### ISCC — the durable fingerprint standard (Layer 3)

When implementing the durable (soft) binding, we confirmed the standard and its Rust implementation.

#### [ISO 24138:2024 — International Standard Content Code (ISCC)](https://www.iso.org/standard/77899.html)
- **Authors / Org:** ISO / ISCC Foundation
- **Type:** spec (primary)
- **Published:** 2024 · **Accessed:** 2026-06-05
- **Relevance:** high (the durable-binding standard)
- **What this contributed:** Defines ISCC — similarity-preserving, content-derived identifiers across media types. This is *why* the durable binding survives reformatting/light edits where a SHA-256 hard binding can't: a near-identical document yields the same or a near ISCC code. It's the standard behind the `io.iscc.v0` soft-binding entry.

#### [iscc-core — reference implementation](https://github.com/iscc/iscc-core)
- **Authors / Org:** ISCC Foundation
- **Type:** open-source project (reference, Python)
- **Published:** ongoing · **Accessed:** 2026-06-05
- **Relevance:** medium
- **What this contributed:** The 100%-test-covered Python reference for the ISCC algorithms; the conformance baseline that the Rust implementation is tested against. Establishes the `gen_text_code_v0(text, bits)` API shape we use.

#### [iscc-lib — Rust implementation](https://github.com/iscc/iscc-lib) ([docs](https://lib.iscc.codes/))
- **Authors / Org:** ISCC Foundation
- **Type:** open-source project (Rust)
- **Published:** v0.4.0 (crates.io) · **Accessed:** 2026-06-05
- **Relevance:** high (adopted)
- **What this contributed:** The Rust implementation of ISO 24138 we adopted (rather than reimplementing the algorithm). `gen_text_code_v0` produces the Text-Code we attach as the C2PA soft binding. Conformance-tested against the Python reference, so its codes match the broader ISCC ecosystem.

## Synthesis

**Replace the bespoke `Badge` with a C2PA manifest emitted by c2pa-rs. Keep bespoke only what has no standard: the process-metadata schema and the "Human Authored" claim semantics.**

| Bespoke today | Adopt (standard) | Crate |
|---|---|---|
| `Badge` JSON envelope | C2PA manifest store, detached **`.c2pa` sidecar** | `c2pa` 0.85 |
| `record` (process metadata) | **custom assertion** `org.humanshipd.process.v1` — *schema stays ours* | `c2pa` |
| `record_sha256` + JCS canonicalization | C2PA hard-binding hash (final-text SHA-256 as asset hash) | `c2pa` |
| `public_key` + Ed25519 signature | **COSE / X.509** claim signature (ed25519 supported) | `c2pa` |
| custom timestamp token | **built-in RFC 3161** via `tsa_url` | `c2pa` |
| "human authored" identity | **CAWG identity assertion** (`cawg.creator`) + `digitalSourceType` — *semantics ours* | `c2pa::identity` |
| — | optional public anchoring | `sigstore-rekor` (opt-in) |
| — | optional portable VC | `ssi` (deferred; CAWG VC spec unstable) |

**Migration:** add `c2pa = "0.85"`; keep `session::build_record` (it produces the `ProcessRecord`); replace `badge.rs` / `signing.rs` / `timestamp.rs` with a c2pa-rs Builder that adds the record as a custom assertion, signs via a Settings TOML (`[signer.local]` / `[cawg_x509_signer.local]` with `alg`, cert chain, `tsa_url`, `referenced_assertions`), and emits a detached `.c2pa` sidecar over the final-text hash; replace `verify.rs`'s crypto with `Reader`, keeping only the honest-claim rendering.

**Accepted tradeoffs:** (1) c2pa-rs signing needs an **X.509 cert chain**, not a bare Ed25519 key — for local trust we generate a self-signed chain; the fork-and-forge honesty (open-source local issuance) is unchanged. (2) c2pa-rs is a **heavy dependency** vs the ~200-line bespoke core — accepted because **interoperability with the C2PA/Content-Credentials ecosystem (Authors Guild, CAI tooling) IS the value proposition** for a human-authorship credential; this is the one place where the standard's weight is justified (unlike the capture layer, where we keep deps lean).

## Implementation findings (2026-06-05)

- **C2PA adoption proven:** `core::credential` issues + verifies via c2pa-rs — process record as the `org.humanshipd.process` custom assertion, signed with `EphemeralSigner` (self-signed, valid-but-untrusted; fits the honest local model), verified by `Reader`. Tests: round-trip + asset-tamper detection. (Note: c2pa strips a trailing `.vN` from assertion labels, so the label is `org.humanshipd.process`, not `…​.v1`.)
- **Detached-over-text is blocked by a real c2pa gap:** `Builder::data_hashed_placeholder("text/plain")` returns **`"type is unsupported"`** — c2pa's data-hash binding is format-specific and has **no text handler**. So a standalone `.c2pa` over `sha256(text)` can't be produced via the Builder data-hash API today. **Next-task options:** (a) register a custom c2pa format handler for our text type; (b) use the **BoxHash** path; or (c) interim — bind the text via the record's `document_binding.final_text_sha256` (set by `build_record`) and have the verifier recompute `sha256(text)` and compare, while the C2PA manifest provides the signed envelope. The embed-in-carrier path is proven; the format-agnostic detached path is the open item.

## First-class / SOTA binding for written work (2026-06-05)

Researched the genuinely first-class architecture (not a hack) for binding a C2PA credential to text/documents. Three layers:

1. **Hard binding (do now, c2pa-rs first-class):** a standalone **`.c2pa` sidecar manifest** (`application/x-c2pa-manifest-store`, writable in c2pa-rs) with a **data-hash over the exported file's exact bytes** — **format-agnostic** (PDF/DOCX/EPUB/txt), sidestepping the text-handler gap (hash the *file*, not `text/plain`). Carries our custom process assertion + **CAWG X.509 identity assertion** (`cawg.identity`, write-supported in c2pa-rs) + `c2pa.created` action.
2. **In-text travel (spec, not in SDK):** C2PA 2.4 **Appendix A.8** embeds the manifest as non-rendering **Unicode variation selectors** (invisible, survives copy-paste). Add via the Encypher `c2pa-text` codec around a c2pa-rs manifest.
3. **Durable Content Credentials (robustness frontier):** hard binding + **soft binding** + registry. For prose the robust leg is a **content fingerprint — ISCC `io.iscc.v0`** (no capacity needed, survives reformatting) resolved via a manifest registry / Soft Binding Resolution API. A ZWC text watermark (`com.writerslogic.zwc-watermark.1`) exists but is fragile.

**Hard practical constraint:** c2pa-rs **cannot embed** into PDF (read-only), DOCX/EPUB/ZIP (unsupported), or text (unsupported). So native-embed-in-document is NOT first-class yet; the **sidecar `.c2pa` + data-hash over the file IS** the first-class substitute, and is the recommended Layer-1.

Sources (this round): [C2PA 2.4 spec — A.4 PDF / A.6 ZIP-EPUB-OOXML / A.7 HTML / A.8 unstructured text / §9.2.4 / §9.3](https://spec.c2pa.org/specifications/specifications/2.4/specs/C2PA_Specification.html); [c2pa-rs supported-formats](https://github.com/contentauth/c2pa-rs/blob/main/docs/supported-formats.md) (PDF read-only; DOCX/EPUB/text unsupported; `.c2pa` writable); [c2pa-rs `sdk/src/identity/`](https://github.com/contentauth/c2pa-rs) (CAWG X.509 identity write-supported); [Encypher c2pa-text](https://github.com/encypherai/c2pa-text) (A.8 reference codec); [C2PA Soft Binding Algorithm List](https://github.com/c2pa-org/softbinding-algorithm-list) (text-capable: `io.iscc.v0`, `com.digimarc.validate.1`, `com.writerslogic.zwc-watermark.1`); [CAI TrustMark+C2PA](https://opensource.contentauthenticity.org/docs/trustmark/c2pa/) & [Soft Binding Resolution API](https://developer.adobe.com/cai-soft-binding-api/) (Durable CC, image-led); [CAWG identity framework](https://cawg.io/about/identity-framework/). All accessed 2026-06-05.

## Downstream uses

- `core/src/credential.rs` — C2PA credential via c2pa-rs (Layers 1 & 2); bespoke badge/signing/timestamp removed.
- `core/src/fingerprint.rs` — ISCC durable fingerprint (Layer 3a) via `iscc-lib`, attached as a `c2pa.soft-binding` assertion.
- `registry/` — opt-in fingerprint → credential lookup service (Layer 3b); `registry/tests/durable_recovery.rs` proves end-to-end recovery.
- Design spec §6/§7/§9; README ("how the certificate travels").
