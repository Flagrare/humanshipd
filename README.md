# humanshipd

> Working codename. Public-facing claim wording is **"Human Authored"**.

An **open-source, local-only, zero-telemetry** tool that records *how* a piece of writing was composed and issues a verifiable **Human Authored** credential. It is the inverse of [SynthID](https://deepmind.google/technologies/synthid/): rather than watermarking AI output, it attests a human writing process.

## Honest framing (read this first)

This is a **tamper-evident attestation and deterrent — not proof.** We say so openly because the project is open-source and its threat model is published in full.

- It **cannot** prove a human originated the ideas. The "copy-type" attack (reading AI output and retyping it) is information-theoretically undefeatable, and a forked client could emit a fake signed record.
- What the credential **does** assert: *"this record was produced by this client and has not been altered since time T,"* plus *"the writing showed an incremental, human-like composition process with no large un-keyed insertions."*
- It **detects** wholesale AI paste/dumps and post-issuance tampering/back-dating.

See the full [threat model](docs/superpowers/specs/2026-06-05-human-authorship-attestation-design.md#4-threat-model-published-openly).

## Principles

- **Open source.** Auditable by anyone.
- **Local-only.** Content never leaves your machine; only hashes do.
- **Zero telemetry.** No tracking, no accounts, no analytics.
- **No monetization.** A public good, not a product.
- **All claims verifiable.** Every research claim is cataloged with its sources.

## Architecture (in brief)

One shared **Rust core** owns all credential logic (record format, hashing, signing, RFC 3161 time-anchoring, verification). Thin, per-surface **capture adapters** feed it:

- **Browser extension (MV3)** → Google Docs + web editors (forwards events to the core over Native Messaging; holds no credential logic)
- **OS Accessibility adapter** → native apps (Word, Scrivener, Final Draft)
- **OCR** → last-resort fallback

## Status

Early design. Nothing is built yet. Start with the design spec:

- **Design spec:** [`docs/superpowers/specs/2026-06-05-human-authorship-attestation-design.md`](docs/superpowers/specs/2026-06-05-human-authorship-attestation-design.md)
- **Research catalog:** [`docs/research/`](docs/research/) — feasibility, capture architecture, and label-framing, each with cited sources.

## License

[MIT](LICENSE)
