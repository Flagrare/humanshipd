# Research: Battle-Tested Models for Representing & Capturing a Live Edit Stream

- **Slug:** `2026-06-06-edit-stream-models-capture-pipeline`
- **Date:** 2026-06-06
- **Status:** complete
- **Triggered by:** Designing the capture layer of humanshipd — want the internal edit-event model and capture architecture grounded in the canonical models (OT, CRDT, event sourcing, real editor op-logs, keystroke-logging schemas) rather than ad hoc.
- **Informed:** The `EditEvent`/op model that feeds `core/src/record.rs::ProcessStats`; the capture pipeline shape across the gdocs / native-ax / web / ocr adapters.

## Question

What are the best, battle-tested models for representing and capturing a live stream of edit operations (insert/delete with position + timestamp), and what should humanshipd's internal edit-event model and capture pipeline look like?

## Sources

### 1. Operational Transformation (OT)
- [Operational Transformation — Wikipedia](https://en.wikipedia.org/wiki/Operational_transformation) (accessed 2026-06-06) — primitive ops = `insert(pos, text)` and `delete(pos, len)`; Ressel et al. (GROUP 1996) defined transform properties TP1/TP2.
- [Towards a unified theory of OT and CRDT — Raph Levien, Medium](https://medium.com/@raphlinus/towards-a-unified-theory-of-operational-transformation-and-crdt-70485876f72f) (accessed 2026-06-06) — Ressel 1996's TP1/TP2; almost all practical OT forgoes TP2 and relies on a **central server to establish a canonical operation order**.
- [Operational Transformation, the algorithm behind Wave/Google Docs — HN](https://news.ycombinator.com/item?id=1354427) and [Apache Wave OT teardown](https://llopv.github.io/gsoc-2017/e2ee/2017/06/30/encrypt-ot-1.html) (accessed 2026-06-06) — Wave's wire model: an op is a sequence of components (`retain N`, `insert "str"`, `delete N`) that together describe a transformation over the whole document; ops carry a revision number for ordering.
- **What this contributed:** The OT operation is **position-relative against a versioned document** (`insert(pos,s)`, `delete(pos,len)`, or Wave's retain/insert/delete component list + revision number). Correctness depends on a server-assigned total order. Positions are only meaningful relative to a known revision — fragile as a *durable archival* model, but a clean *wire* model.

### 2. CRDTs for text
- [CRDTs go brrr — Joseph Gentle (josephg.com)](https://josephg.com/blog/crdts-go-brrr/) (accessed 2026-06-06) — **primary, concrete op shape.** Each inserted character is an item: unique ID = `(agent_id, seq)`, a `parent`/origin reference to the ID it was inserted after, content, and a logical clock. Ordering: children sorted by sequence number (bigger first); ties broken by agent ID ⇒ concurrent. Deletes are tombstones. This is RGA/YATA-family.
- [Peritext: A CRDT for Collaborative Rich Text Editing — Ink & Switch (CSCW)](https://www.inkandswitch.com/peritext/static/cscw-publication.pdf) (accessed 2026-06-06) — rich-text CRDT; per-character IDs + formatting spans.
- [The CRDT Dictionary — Ian Duncan](https://www.iankduncan.com/engineering/2025-11-27-crdt-dictionary/) and [Yjs vs Automerge vs Loro 2026 — PkgPulse](https://www.pkgpulse.com/guides/yjs-vs-automerge-vs-loro-crdt-libraries-2026) (accessed 2026-06-06) — RGA (Automerge), YATA (Yjs), Logoot (tombstone-free position identifiers), Fugue (Weidner et al. 2023, provably maximal non-interleaving). All assign a **unique ID per character** + causal ordering + tombstones.
- [Designing Data Structures for Collaborative Apps — Matthew Weidner, CMU](https://www.cs.cmu.edu/~csd-phd-blog/2023/collaborative-data-design/) (accessed 2026-06-06) — the local-first design rationale; no central server required for convergence.
- **What this contributed:** The durable, server-independent edit model: every edit is identified by `(agent, seq)` + logical clock, positions expressed *relationally* (after which ID), not as absolute offsets. This is why CRDTs are favored for local-first — the log is self-describing and merges without a coordinator. The cost is per-character ID overhead and tombstones.

### 3. Event sourcing
- [Event Sourcing — Martin Fowler (2005)](https://martinfowler.com/eaaDev/EventSourcing.html) (accessed 2026-06-06) — append-only log of state-changing events is the source of truth; current state is a fold over the log; supports replay and temporal queries.
- [Event Sourcing Pattern — Microsoft Azure Architecture Center](https://learn.microsoft.com/en-us/azure/architecture/patterns/event-sourcing) (accessed 2026-06-06) — durable, ordered append-only event store; **snapshots are an optimization, not a replacement** — rehydrate from latest snapshot + replay subsequent events.
- [Event Sourcing in Practice: append-only store with projections and snapshots — Let's Build](https://letsbuildsolutions.com/blog/system-design/event-sourcing-in-practice-building-an-append-only-event-store-with-projections-and-snapshots/) (accessed 2026-06-06) — schema evolution strategies: weak/additive schema, event versioning, upcasting, in-place migration.
- **What this contributed:** The architectural frame: "the writing process *is* the sequence of edit events." The raw `EditEvent` log is the source of truth; `ProcessStats`/timeline are **projections** (folds). Add a schema version field per event and an upcaster; snapshot periodically for replay performance.

### 4. How real editors store/expose edit history
- [How I reverse-engineered Google Docs — James Somers](https://features.jsomers.net/how-i-reverse-engineered-google-docs/) + our live confirmation (see [`2026-06-06-google-docs-writing-capture.md`](./2026-06-06-google-docs-writing-capture.md)) (accessed 2026-06-06) — Docs ops: `ty:"is"` insert (`ibi` 1-based index, `s` string), `ty:"ds"` delete (`si`/`ei`), `ty:"mlti"` multi-bundle; each save carries `rev` + a `bundles[]` of `commands` with `sid`/`reqId`; per-entry timestamp + session/user. **Absolute-offset, server-revision-ordered** — OT-family.
- [The data model behind Notion — Notion engineering blog](https://www.notion.com/blog/data-model-behind-notion) (accessed 2026-06-06) — UI actions become **operations** that create/update a single record, **batched into transactions** committed atomically by the server via a `TransactionQueue`. Text is small per-block LWW registers keyed by a logical clock.
- [Understanding sync engines: Figma, Linear, Google Docs — Liveblocks](https://liveblocks.io/blog/understanding-sync-engines-how-figma-linear-and-google-docs-work) and [Figma uses CRDT — HN](https://news.ycombinator.com/item?id=33635729) (accessed 2026-06-06) — Figma = CRDT-ish multiplayer (per-object LWW with a central server as arbiter), Docs = OT, Linear = transactional sync.
- **What this contributed:** Across the industry the unit is an **operation** (insert/delete/set) carrying position + timestamp + actor/session, **grouped into atomic transactions/bundles**, with a monotonic revision or logical clock for ordering. Docs gives us absolute offsets for free on the most-wanted surface.

### 5. Writing-process research data models
- [Inputlog: A logging tool for the research of writing processes — Leijten & Van Waes](https://www.researchgate.net/publication/4983063_Inputlog_A_logging_tool_for_the_research_of_writing_processes) + [Keystroke Logging in Writing Research (open PDF, U Antwerpen)](https://repository.uantwerpen.be/docman/irua/e8d12b/b39f88f0.pdf) (accessed 2026-06-06) — Inputlog logs each input event with timestamp, type, value, and document position; analysis modules derive **pauses, P-bursts (production episodes ended by ≥2s pause), R-bursts (ended by a revision action), and revisions**.
- [Exploring keystroke logging in L2 writing — ScienceDirect](https://www.sciencedirect.com/science/article/abs/pii/S2772766124000855) (accessed 2026-06-06) — confirms the standardized indices: pausing behaviour, bursts, revision activity, process variance.
- **What this contributed:** The field's standardized derived metrics — **pause threshold = 2s**, P-bursts vs R-bursts, revision counts — map directly onto our existing `ProcessStats` (`pauses.gt_2s`, `bursts`, `revisions`). No emerging interchange *format* dominates (Inputlog exports IDFX/CSV/XML), so we standardize the *metrics*, not their file format.

## Synthesis

Four mature models, each solving a different problem:

- **OT** = a *wire/transport* model for live collaborative editing. Absolute positions relative to a server revision. Battle-tested (Docs, Wave) but order-fragile without a coordinator — and Docs hands us exactly this stream for free.
- **CRDT** (RGA/YATA/Logoot/Fugue, via Yjs/Automerge) = the *durable, local-first* model. Per-edit identity `(agent, seq)` + logical clock + relational positions, mergeable with no server. This is the right shape for an archival, tamper-evident log.
- **Event sourcing** = the *architecture* around either: an append-only log as source of truth, with projections (our stats/timeline) folded from it and periodic snapshots for replay.
- **Keystroke-logging schemas** (Inputlog/ScriptLog) = the *domain vocabulary* — pauses (2s), P-/R-bursts, revisions — already reflected in `ProcessStats`.

The convergent industry primitive is: **an immutable, timestamped operation (insert/delete) carrying an actor/session ID and an ordering key (revision *or* logical clock), grouped into atomic transactions, appended to a log that is the source of truth.** humanshipd needs the *capture* half of this, not the *merge* half — there is one author and one device, so we never need OT transform functions or CRDT merge. We borrow CRDT's **durable per-edit identity + event sourcing's append-only-log-as-truth**, and ingest OT-style absolute offsets where a surface (Docs) provides them.

## Implications for humanshipd's capture model

1. **Internal `EditEvent` op model (the raw log, upstream of `ProcessStats`).** One immutable, append-only event per edit:
   - `op`: `Insert { len }` | `Delete { len }` | `Replace { del_len, ins_len }` (counts only — never content, per the metadata-only mandate).
   - `at_ms`: ms since session start (the ordering key — single author, single device ⇒ a monotonic local clock is sufficient; no logical vector clock or transform needed).
   - `offset`: `Option<u64>` — absolute caret offset when the surface gives it (Docs `ibi`, AX selection range); `None` otherwise (already mirrored by `TimelinePoint.offset`).
   - `keystrokes`: physical keystrokes correlated to this op (0 ⇒ appeared without typing — the AI-dump signal).
   - `provenance`: `Typed | Pasted | AiTool | Unknown` (already in `record.rs`).
   - `seq`: monotonic per-session sequence (CRDT-style stable identity for dedup/idempotent replay).
2. **Append-only log is the source of truth (event sourcing).** Persist each `EditEvent` immediately (critical for the MV3 service worker that dies after ~30s idle). `ProcessStats`, `spans`, and `timeline` in `record.rs` become **projections folded from the log** — keep them, but derive them, don't capture them directly. The log is hashed; `Replay.log_sha256` already binds it.
3. **Version every event + upcaster (schema evolution).** The record already has `SCHEMA = "authorshipped/record@0.4"`; add a per-event `v` and a small upcast chain so old logs replay under new code.
4. **Transactions/bundles, not loose ops.** Mirror Docs/Notion: group ops committed together (one save bundle, one paste, one AX mutation batch) so burst/revision detection sees real authoring units, not artificial fragments.
5. **Derive the standardized metrics, not new ones.** Fold the log into Inputlog's vocabulary: 2s pause threshold, P-bursts/R-bursts, revision counts — exactly the `pauses`/`bursts`/`revisions` already in `ProcessStats`.
6. **Capture pipeline shape:** per-surface adapter emits raw surface ops → normalize to `EditEvent` (counts + optional offset + keystroke correlation) → append to durable per-session log → fold into `WritingSessionRecord` projections → hash/sign/anchor. **Do NOT pull in OT transforms or a CRDT merge engine** — single-author capture needs neither; adopting them would be the ad-hoc-avoidance trap of over-engineering. Take the *data-model discipline* (durable per-edit identity, append-only truth, atomic bundles, versioned events), not the *concurrency machinery*.

## Downstream uses

- The `EditEvent`/op model upstream of `core/src/record.rs::ProcessStats`.
- The capture-fidelity spec (`docs/superpowers/specs/2026-06-06-content-binding-and-capture-fidelity-design.md`) — adapter → normalize → append → fold pipeline.
