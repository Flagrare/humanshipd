# Research: Binding a Credential to an Identity Without a CA (Individual Signers)

- **Slug:** `2026-06-08-identity-binding-without-a-ca`
- **Date:** 2026-06-08
- **Status:** complete
- **Triggered by:** The "how does an individual signer with no CA-issued certificate bind a credential to an identity/persistent key, in a standard way, without a centralized authority" question for humanshipd (local-only, zero-telemetry).
- **Informed:** The identity/signing-model decision for v1 (what baseline to ship and what to leave as an upgrade path).

## Question

For an open-source, local-only, zero-telemetry tool issuing C2PA credentials that attest a human writing process, how do you bind a credential to an identity (or persistent key) for an **individual** signer with no CA-issued cert — standard-aligned and without a centralized authority? What is the industry standard, and what are the credible lightweight alternatives, with honest maturity assessments?

## Sources

### CAWG identity assertion (the standard for naming a creator on C2PA)
- **[CAWG Identity Assertion 1.2](https://cawg.io/identity/1.2/)** — Creator Assertions Working Group / DIF — **primary** — accessed 2026-06-08 — high — **DIF Ratified 2025-12-15.** Defines exactly two credential types: `cawg.x509.cose` (X.509 + COSE, traditional PKI, S/MIME-style certs for orgs) and `cawg.identity_claims_aggregation` (a specialized W3C VC issued by an aggregator). v1.2 added §8.2.4 "Trust model for X.509 certificates." Individuals lacking a CA relationship use the **identity claims aggregation** path.
- **[CAWG identity framework](https://cawg.io/about/identity-framework/)** — CAWG — **primary** — accessed 2026-06-08 — high — Aggregator = "trusted platform vendor [that gathers] information about a user, typically an individual content creator, and replay[s] those signals on their behalf," signing the CAWG assertion with its own key. **did:web/DIDs are NOT a currently supported option** (only X.509 + ICA). "Full W3C Verifiable Credentials" is named only as a "possible future credential type" with "no guarantee regarding when or whether." (page dated Jan 2026)
- **[CAWG identity assertions — CAI open-source docs](https://opensource.contentauthenticity.org/docs/manifest/cawg-id/)** — Adobe CAI — **primary/tooling** — accessed 2026-06-08 — high — X.509 path is for enterprises/news orgs; aggregator path is "for individuals." c2pa-rs SDK can build/sign/validate both.

### C2PA core trust model (why self-signed isn't "in the trust list")
- **[C2PA Conformance / Trust List](https://c2pa.org/conformance/)** + **[CAI: getting a signing certificate](https://opensource.contentauthenticity.org/docs/signing/get-cert/)** — C2PA / Adobe CAI — **primary** — accessed 2026-06-08 — high — The official **C2PA Trust List launched mid-2025**; only conformance-approved generator products get certs from listed CAs. Core C2PA "does not support attribution to individuals/organizations" (privacy-preserving by design); identity is an **extension** (CAWG). A self-signed cert produces a cryptographically valid manifest that simply **does not chain to the C2PA trust anchor** — verifiers show it as untrusted/unverified, not invalid.
- **[C2PA Spec 2.4 — timestamps](https://spec.c2pa.org/specifications/specifications/2.4/specs/C2PA_Specification.html)** — C2PA — **primary** — accessed 2026-06-08 — high — Strongly recommends an **RFC 3161 TSA** timestamp so signatures can be validated indefinitely (proves the sig existed while the cert was valid). C2PA maintains a **TSA trust list** too. **C2PA does not reference Sigstore/Rekor-style transparency logs** as part of its model.

### Sigstore keyless (Fulcio + Rekor) — the SW-supply-chain pattern
- **[Sigstore cosign signing overview](https://docs.sigstore.dev/cosign/signing/overview/)** + **[Sigstore security model](https://docs.sigstore.dev/about/security/)** + **[Rekor overview](https://docs.sigstore.dev/logging/overview/)** — Sigstore — **primary** — accessed 2026-06-08 — high — Flow: ephemeral in-memory keypair → OIDC token (Google/GitHub/Microsoft) → **Fulcio** (free CA) issues a ~minutes-long X.509 cert binding the key to the OIDC identity → sign → record cert+sig+digest in **Rekor** (append-only Merkle transparency log, public inclusion + consistency proofs). Shifts trust from "trust this key" to "trust this identity at this timestamp." Private key never hits disk.
- **[A Longitudinal Study of Usability in Identity-Based Software Signing (arXiv 2603.17133)](https://arxiv.org/pdf/2603.17133)** — academic — **secondary** — accessed 2026-06-08 — medium — Confirms identity-based signing (Sigstore-style) is the maturing direction; append-only issuance/signing logs are now common in identity-based ecosystems.

### did:web / DIDs + VCs maturity
- **[W3C DID Core / DID v1.1](https://www.w3.org/TR/did-1.1/)** + **[W3C VC Data Model]** — W3C — **primary** — accessed 2026-06-08 — high — DID Core and VC Data Model are W3C Recommendations (stable). `did:web` resolves an identity to a domain you control (`https://example.com/.well-known/did.json`) — self-sovereign, no central CA, but trust == "do you trust this domain," and a domain owner can silently rotate keys (no transparency log unless added).
- **[GS1: VCs & DIDs Technical Landscape (2025)](https://ref.gs1.org/docs/2025/VCs-and-DIDs-tech-landscape)** — GS1 — **secondary** — accessed 2026-06-08 — medium — Core standards stable; 150+ DID methods → interop friction; did:web is among the closest-to-standard, lowest-infra methods. Real tooling exists (edge/cloud wallets) but ecosystem still consolidating.

### Individual-creator practice + Sigstore↔C2PA bridge
- **[World Privacy Forum: Privacy, Identity and Trust in C2PA](https://worldprivacyforum.org/posts/privacy-identity-and-trust-in-c2pa/)** — WPF — **secondary** — accessed 2026-06-08 — medium — Independent analysis of the CAWG aggregator model and its privacy/centralization tradeoffs (aggregator becomes a trusted intermediary).
- **Sigstore↔C2PA bridge:** searches found **no production project** bridging Sigstore keyless signing into C2PA manifest signing. C2PA uses COSE signatures and its own CA trust list; the patterns are conceptually compatible (both bind identity→key→artifact, both can use transparency/timestamping) but **not wired together today**. Treat as a design analogy, not an off-the-shelf path.

## Synthesis

**The industry standard for naming an individual on a C2PA credential is the CAWG identity assertion's *identity claims aggregation* path** — a third-party "aggregator" verifies your signals (verified social accounts, websites, accreditations) and issues a W3C-VC-style credential bound to the specific asset, signing on your behalf. It is the most mature, ratified (DIF, 2025-12-15) answer — **but it is a centralized authority by another name** (Adobe Connected Identities is the main production aggregator), requires a network round-trip, and is therefore **incompatible with local-only + zero-telemetry**. The X.509/CAWG path is for orgs and likewise needs a CA. **did:web/DIDs are not yet a supported CAWG credential type** (named only as possible-future).

**Credible lightweight alternatives, by maturity:**
1. **Self-signed key, identity unverified, disclosed as such** — fully local, zero-telemetry, standard-aligned (produces a valid C2PA manifest that just doesn't chain to the trust list). The honest baseline. C2PA itself is privacy-preserving without identity, so this is *in-model*, not a hack.
2. **Sigstore-style keyless (Fulcio + Rekor)** — elegant for "no long-lived key," but requires OIDC (telemetry) + a public transparency log + a CA (Fulcio). **Disqualified by local-only/zero-telemetry** in its hosted form; the *transparency-log pattern* is borrowable offline (a local append-only Merkle log the user can optionally publish) but that's bespoke, not a standard.
3. **did:web as the persistent identity** — lets a user bind a credential to a domain they already control, self-sovereign, no CA, no telemetry at sign time (resolution is verifier-side). Standards are stable; it is **not yet a CAWG-recognized credential type**, so it'd be a forward-compatible extension, not conformance.

**Honest baseline answer:** For humanshipd, **"self-signed key (persistent, user-held), identity unverified by default, with an explicit upgrade path to bind to a user-supplied identity later"** *is* the standard-aligned local-only baseline — there is no lightweight standard that gives verified identity without a network authority. The right architecture is: persistent local keypair → sign the C2PA manifest + always include an **RFC 3161 timestamp** (the one piece of external trust that is cheap, standard, and doesn't leak content) → leave a clean seam for (a) CAWG aggregator binding and (b) did:web, as opt-in upgrades. Disclose the trust level plainly (self-signed / timestamped / identity-bound) rather than implying verification that isn't there.
