# Research: Credential Signing, Trust Model, and the Fork-and-Forge Problem

- **Slug:** `2026-06-08-credential-signing-trust-and-fork-and-forge`
- **Date:** 2026-06-08
- **Status:** complete
- **Triggered by:** Open decision A — how the credential is signed and what trust/identity model applies, given an open-source, local-only, zero-telemetry tool (and the fork-and-forge threat that follows from open-source signing).
- **Informed:** Decision A (signing/trust model); base spec §4 threat model (fork-and-forge framing); RFC 3161 timestamping reaffirmation.

## Question

For an individual signer using an open-source, local-only tool — with no CA-issued certificate and no server — what is the industry-standard way to sign a C2PA credential and bind it to identity, and how do open-source projects handle the fact that a fork can forge signed credentials?

## Sources

### C2PA signing & trust model
- **[C2PA Technical Spec 2.4](https://spec.c2pa.org/specifications/specifications/2.4/specs/C2PA_Specification.html)** / **[2.2](https://spec.c2pa.org/specifications/specifications/2.2/specs/C2PA_Specification.html)**, **[CAI: get a signing cert](https://opensource.contentauthenticity.org/docs/signing/get-cert/)**, **[CAI: trust lists](https://opensource.contentauthenticity.org/docs/conformance/trust-lists/)**, **[C2PA Conformance Program](https://c2pa.org/conformance/)**, **[DigiCert/Adobe trust list](https://knowledge.digicert.com/solution/digicert-and-adobe-approved-trust-list)** — **primary** — accessed 2026-06-08 — high
- **What they contributed:** C2PA signs via COSE (`COSE_Sign1`) with **X.509 only**; signing-cert profile requires `digitalSignature` keyUsage and the `c2pa-kp-claimSigning` EKU (or doc-signing EKU). Manifest states: **well-formed → valid → trusted**. *Valid* = signature verifies + bindings match (says nothing about who). *Trusted* = cert chains to a CA on a **trust list**. Getting trusted needs a cert from a C2PA-conformant CA (DigiCert/SSL.com, ~$289/yr, org-gated via the Conformance Program). A self-signed/test cert produces a valid manifest reported as **`signingCredential.untrusted`** — a spec-anticipated, legitimate state (CAI's own samples use test certs). Adobe signs against DigiCert (on the trust list) → trusted.

### Identity binding for individuals without a CA
- **[CAWG Identity Assertion v1.2 (DIF, 2025-12-15)](https://cawg.io/identity/1.2/)**, **[CAWG identity framework](https://cawg.io/about/identity-framework/)**, **[W3C DID Core 1.1](https://www.w3.org/TR/did-1.1/)**, **[SSL.com C2PA](https://www.ssl.com/products/content-authenticity/content-credentials/c2pa/)** — **primary** — accessed 2026-06-08 — high
- **What they contributed:** CAWG's identity assertion has an **X.509 path** (org/CA) and — the answer for individuals — an **identity-claims-aggregation path** (`cawg.identity_claims_aggregation`): a trusted aggregator verifies identity *signals* (incl. **social-media account control**, `cawg.social_media`) and issues a **W3C VC** binding them, signing on the creator's behalf. **did:web/DIDs** are referenced but not mandated. Reality: there is **no free, mainstream-recognized individual path** today — the aggregation ecosystem is "a spec ahead of its ecosystem"; mainstream validators trust only the C2PA Trust List. So the practical options are self-signed (untrusted), wait for an aggregator, or did:web (weak verifier support).

### Fork-and-forge in open source / the trusted-client problem
- **[Kerckhoffs's principle](https://en.wikipedia.org/wiki/Kerckhoffs's_principle)**, **[C2PA Security Considerations 2.4](https://spec.c2pa.org/specifications/specifications/2.4/security/Security_Considerations.html)**, **[Sigstore security model](https://docs.sigstore.dev/about/security/)** / **[cosign keyless](https://docs.sigstore.dev/cosign/signing/overview/)**, **[SLSA threats](https://slsa.dev/spec/v1.0/threats)**, **[Play Integrity](https://developer.android.com/google/play/integrity/overview)**, **[Apple Secure Enclave](https://support.apple.com/guide/security/the-secure-enclave-sec59b0b31ff/web)**, **[UMBC C2PA security analysis](https://cisa.umbc.edu/verifying-provenance-of-digital-media-security-analysis-of-c2pa-and-its-implementation/)** — **primary/academic** — accessed 2026-06-08 — high
- **What they contributed:** Client-side code **cannot prevent** forgery (Kerckhoffs / "the enemy knows the system" / trusted-client problem) — the adversary controls the code and the machine, so no key can be hidden and no honest behavior enforced. C2PA itself states it provides **tamper-evidence, not forgery prevention**; UMBC's analysis confirms tamper-evidence "is insufficient to verify provenance or veracity." The achievable goals are **attribution, detection, cost-raising**. **Sigstore/SLSA's reframe:** stop preventing; bind every signature to an **OIDC identity + append-only transparency log (Rekor)** so forgery is *attributable and detectable*. **Hardware/TEE (Secure Enclave, Play Integrity, Truepic)** is the only thing that actually *stops* fork-and-forge (non-extractable keys / server-issued verdicts) — but it's fundamentally incompatible with open-source + local-only + cross-platform. Analogous domains (DRM, anti-cheat) reached the same wisdom: client-side can't prevent; use authority/attestation/attribution/cost.

## Synthesis

The three threads converge on one answer for an open-source, local-only, zero-telemetry tool:

1. **Forgery cannot be prevented** here — that's a law (Kerckhoffs), not a gap, and C2PA agrees its job is tamper-*evidence*, not prevention. Only hardware/TEE or a server authority prevent it, and both break our constraints. So we **accept and disclose** fork-and-forge (we already do in §4), and pursue **attribution + cost-raising**, not prevention.
2. **Self-signed is the legitimate, spec-anticipated baseline.** It yields a *valid* (cryptographically sound, tamper-evident) credential reported as **`signingCredential.untrusted`** — the verifier makes the trust call. "Trusted" requires a coalition-gated commercial CA (~$289/yr), unreachable per-individual/OSS — so we don't pretend to it; we frame honestly as "valid; identity unattested."
3. **Add RFC 3161 timestamping** (optional, user-supplied TSA): the one standardized, CA-free "existence proof" C2PA endorses — proves signed-before-T and keeps the credential verifiable after cert expiry. Cheap, standard, high-value.
4. **Make the identity slot pluggable** for opt-in stronger identity later — a CAWG identity-aggregation VC (incl. social-verified) or did:web — without re-architecting. Mainstream "trusted" status stays out of reach for now, but the path is open.
5. **Optional keyless + transparency-log attribution** (the Sigstore model) is the right *future opt-in* for users who want forgery to be attributable — but it's centralized + networked, so strictly opt-in, never the zero-telemetry default.

**Recommended decision:** self-signed + honest "untrusted/identity-unverified" framing as the **default**; **RFC 3161 timestamp** wired in; **pluggable identity** (CAWG aggregator / did:web) and **optional keyless+transparency-log** as opt-ins. Reject mandatory CA-trusted/CAWG-verified as a default (gatekept, costly, breaks local-only/zero-telemetry).

## Downstream uses

- Decision A (signing/trust model) — the recommendation above.
- Base spec §4 threat model — reinforces the honest fork-and-forge framing (attribution/cost, not prevention) with Kerckhoffs + Sigstore + C2PA's own self-statement.
