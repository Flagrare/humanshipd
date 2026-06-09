# Research log

External research conducted for this project. Each entry credits the sources it leaned on and links forward to where the findings landed (design specs, code, decisions). See the `/flagrare:research-catalog` skill for the workflow, or just read an entry to see the shape.

## Sessions

| Date | Topic | Triggered by | Informed |
|------|-------|--------------|----------|
| 2026-06-08 | [Identifying the Same Document Across Sessions (Native Files)](./2026-06-08-native-document-identity.md) | Open Question B: keying the whole-document credential for native files | Decision 7 (native-file document identity) |
| 2026-06-08 | [Credential Signing, Trust Model & the Fork-and-Forge Problem](./2026-06-08-credential-signing-trust-and-fork-and-forge.md) | Open decision A: how to sign + the fork-and-forge threat for an OSS local-only tool | Decision 6; base spec §4 threat model |
| 2026-06-08 | [Binding a Credential to an Identity Without a CA (Individual Signers)](./2026-06-08-identity-binding-without-a-ca.md) | How an individual signer with no CA-issued cert binds a credential to an identity, standard-aligned, without a central authority | Decision 6 pluggable-identity slot; verified-author-identity roadmap |
| 2026-06-06 | [Binding a Credential to a Formatted Document & Content Identity Across Formats](./2026-06-06-document-binding-and-content-identity.md) | Pre-refactor "retain file formats" / verify-against-what decision | v1 architecture brainstrm; binding/artifact model |
| 2026-06-06 | [World-Class Architecture Practices for the v1 Refactor](./2026-06-06-software-architecture-practices.md) | Ground the v1 refactor in ports-&-adapters, shared-core, schema discipline | v1 architecture brainstorm; crate boundaries |
| 2026-06-06 | [Edit-Stream Models & Capture Pipeline (OT/CRDT/event-sourcing)](./2026-06-06-edit-stream-models-capture-pipeline.md) | Ground humanshipd's EditEvent/op model in battle-tested edit-stream models | `EditEvent` op model upstream of `ProcessStats`; capture-fidelity spec |
| 2026-06-06 | [Capturing the Google Docs Writing Process in the Browser](./2026-06-06-google-docs-writing-capture.md) | Make Google Docs the "smoking gun" browser-capture target | Capture-fidelity spec §4; Docs adapter spike |
| 2026-06-06 | [Competitive Landscape of Authorship / Provenance / Detection Tools](./2026-06-06-competitive-landscape-authorship-tools.md) | "Research what other tools do; integrate the best parts" | Signals/reporting spec §8/§9/§10 |
| 2026-06-06 | [AI-vs-Human Authorship Signals and Honest Probability Bands](./2026-06-06-ai-authorship-signals-and-probability-bands.md) | Grammarly-style banded report; "what signals estimate AI probability" | Signal taxonomy; provenance-vs-inference; spec §4/§5 |
| 2026-06-05 | [Adopting C2PA + Standards for the Credential Stack](./2026-06-05-adopting-c2pa-credential-stack.md) | Directive to adopt standards, not reinvent | Refactor core to c2pa-rs; spec §6/§7/§9 |
| 2026-06-05 | [App-Agnostic, On-Device Capture Architecture](./2026-06-05-app-agnostic-capture-architecture.md) | "Blend into any app" capture requirement (Cluely-class) | Capture-layer design; forthcoming spec |
| 2026-06-05 | [Positive vs. Negative Framing of a Human-Authorship Label](./2026-06-05-human-authorship-label-framing.md) | Naming/badge framing decision (AI-Free vs Human Authored) | Naming recommendation; forthcoming design spec |
| 2026-06-05 | [Feasibility of Proving Human Authorship of Text](./2026-06-05-proof-of-human-authorship-feasibility.md) | Scoping the "synthy-free" / "Authorshipped" concept | Feasibility verdict; forthcoming design spec |

## Adding a session

When you research something external (vendor docs, papers, blog posts, open-source repos), the `/flagrare:research-catalog` skill produces a file in this directory and a row in this table. Run it before the synthesis goes back to the requester.
