# Research: World-Class Architecture Practices for the v1 Refactor

- **Slug:** `2026-06-06-software-architecture-practices`
- **Date:** 2026-06-06
- **Status:** complete
- **Triggered by:** Pre-refactor decision — ground the v1 architecture (per-surface capture adapters + shared core + WASM verifier, local-only) in world-class practices rather than ad hoc reorganization.
- **Informed:** The v1 architecture brainstorm (crate boundaries, capture port, schema-versioning discipline, thin adapters).

## Question

What are the canonical, battle-tested architecture practices for a system shaped like humanshipd — a pure domain core with many thin platform/capture adapters, local-only, with an evolving append-only record format?

## Sources

### Ports & Adapters / Clean Architecture
- **[Cockburn, Hexagonal Architecture (2005)](https://alistair.cockburn.us/hexagonal-architecture)** (corroborated via [Wikipedia](https://en.wikipedia.org/wiki/Hexagonal_architecture_(software))); **[Martin, The Clean Architecture (2012)](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)** — **primary** — accessed 2026-06-06 — high
- **What they contributed:** A **port** is an abstract API the domain owns; an **adapter** is glue to the outside, split into *primary/driving* (actors invoking the app) and *secondary/driven* (infra the app invokes). The **Dependency Rule**: source dependencies point only inward; inner layers know nothing of outer ones; only plain data structures cross boundaries. → Every capture surface becomes a *primary adapter* feeding one `CaptureSink` port; the core never imports `chrome`/`AXObserver`/DOM types.

### Shared-core + thin-platform-adapters in practice
- **[1Password (Rust core → WASM, typeshare)](https://corrode.dev/podcast/s04e06-1password/)**, **[Bitwarden SDK architecture](https://contributing.bitwarden.com/architecture/sdk/)**, **[Tailscale platform layers](https://tailscale.com/blog/how-tailscale-works)** — **primary/secondary** — accessed 2026-06-06 — high
- **What they contributed:** The dominant pattern for privacy/security tooling: maximize a shared core, keep clients thin, expose via a typed FFI/WASM boundary, and **generate cross-language types** (typeshare / UniFFI / wasm-bindgen) so adapters can't drift. Hard rule: only presentational/platform-integration code lives outside the core. Validates our "one Rust core, thin shells" instinct with concrete precedent.

### Local-first software
- **[Ink & Switch, "Local-first software" (2019)](https://www.inkandswitch.com/essay/local-first/)** — **primary** — accessed 2026-06-06 — high
- **What it contributed:** Seven ideals (fast/local-primary, multi-device, network-optional, collaboration, longevity/"the Long Now", security & privacy by default, user ownership). We already satisfy several by being local-only; the lesson is to treat **sync as a driven port left unimplemented** — never bake in device-bound IDs or plaintext-on-disk assumptions that would violate longevity/portability later.

### Schema evolution discipline
- **[Protobuf proto3 update rules](https://protobuf.dev/programming-guides/proto3/)**, **[Greg Young, event versioning](https://leanpub.com/esversioning/read)** / **[Dudycz](https://event-driven.io/en/how_to_do_event_versioning/)**, **[Confluent schema evolution](https://docs.confluent.io/platform/current/schema-registry/fundamentals/schema-evolution.html)** — **primary** — accessed 2026-06-06 — high
- **What they contributed:** The discipline that prevents our `0.1→0.4-in-a-day` churn: additive-only fields with defaults; never rename or repurpose a field (field *numbers*, not names, define identity); reserve retired fields; "a new event version must be convertible from the old — if not, it's a *new event*, not a version"; evolve via **upcasters** that read every historical version. → A stable record `version` + additive changes + an upcaster registry.

### Browser extension (MV3) as a thin adapter
- **[Chrome content scripts](https://developer.chrome.com/docs/extensions/develop/concepts/content-scripts)**, **[native messaging](https://developer.chrome.com/docs/extensions/develop/concepts/native-messaging)** — **primary** — accessed 2026-06-06 — high
- **What they contributed:** Content scripts default to the **ISOLATED** world (real boundary from page JS); **MAIN** world only for page-context needs (e.g. patching `fetch` for Docs `/save`). Service worker is short-lived (no persistent state). `nativeMessaging` only from the service worker, not content scripts. → Content script = pure capture (events → `CaptureEvent`); service worker = router to WASM core / native host; verifier = core compiled to WASM; permission-minimization (`activeTab`, scoped hosts) is an architectural choice.

## Synthesis

The v1 architecture writes itself from these:

- **Crates:** `humanshipd-core` (pure domain — capture event, record, C2PA assertion, verification; zero platform deps), `humanshipd-ports` (traits: `CaptureSink` primary; `RecordStore`/`Clock`/`SyncTransport` driven), `humanshipd-schema` (versioned record + upcaster registry), `humanshipd-wasm`/`-ffi` (generated-type boundaries), and thin `adapter-extension` / `adapter-desktop` / `verifier-wasm`.
- **Capture port:** one trait, many primary adapters (Docs content script, web-field listener, desktop Accessibility) that only translate native events to a plain `CaptureEvent` and call the port.
- **Schema discipline:** semantic version, additive-only, never rename/repurpose, new shape ⇒ new type, upcasters for every old version — adopt Protobuf/Avro-style rigor instead of free-form JSON churn.
- **Local-first:** sync is an unimplemented driven port; don't leak sync assumptions into the local design.
- **Thin adapters:** core → WASM (verifier + extension logic) and native lib (desktop) from one codebase; generated cross-language types prevent drift; ISOLATED-world content scripts; minimal permissions.

## Downstream uses

- v1 architecture brainstorm and the forthcoming architecture spec (crate boundaries, capture port, schema versioning).
- Reinforces the existing "one core, thin shells" decision with named precedent (1Password/Bitwarden/Tailscale).
