# Research: Identifying "The Same Document" Across Sessions (Native Files)

- **Slug:** `2026-06-08-native-document-identity`
- **Date:** 2026-06-08
- **Status:** complete
- **Triggered by:** Open Question B — keying the whole-document, accumulated credential (Decision 5) for native files, which can be renamed/moved/copied/Saved-As/format-converted. Decided to combine embedded GUID + OS file identity + content fingerprint.
- **Informed:** Decision 7 (native-file document identity).

## Question

How do mature systems identify "the same document" across editing sessions and file operations (rename, move, copy, Save-As, format conversion), so we can combine an embedded GUID (D) + OS file identity (B) + content fingerprint (C)?

## Sources

### Embedded identity & lineage standards
- **[Adobe XMP Media Management](https://developer.adobe.com/xmp/docs/xmp-namespaces/xmp-mm/)** + [XMP Asset Relationships](https://www.adobe.com/content/dam/cc/us/en/acom/products/xmp/Pdfs/XMPAssetRelationships.pdf) — **primary** — accessed 2026-06-08 — high
- **What it contributed (the canonical model):** `xmpMM:DocumentID` = a UUID created once, "the common identifier for all versions and renditions" — **does not change on edit/save**. `xmpMM:InstanceID` = UUID **updated each save** (the version). `xmpMM:OriginalDocumentID` = retained source ID that **survives format conversion** (on save-to-new-format, mint a new DocumentID but keep the source here). `xmpMM:DerivedFrom` = edge to the immediate predecessor; `xmpMM:History` = high-level event log. The triad: **DocumentID = identity, InstanceID = version, OriginalDocumentID/DerivedFrom = lineage.**
- **[MS-DOCX `w15:docId`](https://learn.microsoft.com/en-us/openspecs/office_standards/ms-docx/b5058d55-0aa8-44e0-9a37-0c84b6e9f68b)** + OOXML `rsid`/`rsidRoot` — **primary** — accessed 2026-06-08 — high
- **What it contributed:** Word's `docId` GUID "a unique identifier for a set of documents derived from a common source" — **persists across Save As**. `rsid`/`rsidRoot` = per-editing-session IDs (shared `rsidRoot` across copies proves common origin) — free session-granular provenance. Format-local (lost converting out of OOXML).
- **[ISO 32000-2 §14.4 PDF `/ID`](https://www.iso.org/standard/75839.html)** — **primary** — accessed 2026-06-08 — medium — two-element array: permanent ID (stable, unchanged on incremental update) + changing ID (per-revision). Same stable/version split.
- **[C2PA Spec 2.4](https://spec.c2pa.org/specifications/specifications/2.4/specs/C2PA_Specification.html)** — **primary** — accessed 2026-06-08 — high — `instanceID` is mandatory and, **when XMP is present, C2PA reuses `xmpMM:InstanceID`/`DocumentID`** (explicit interop). Lineage = `c2pa.ingredient` assertions: a **parent ingredient** references the source's manifest/instanceID, so each edit appends to a provenance chain back to origin.
- **EPUB `dc:identifier`** (W3C) — stable unique ID + Release Identifier (`id@modified`) for versions — same pattern.

### OS file identity + content re-association
- **[Apple NSDocument/bookmarks](https://developer.apple.com/documentation/appkit/nsdocument)** + Eclectic Light (inodes/bookmarks/safe-saves) + [Mothers Ruin (bookmark internals)](https://mothersruin.com/software/Archaeology/reverse/bookmarks.html) — **primary/secondary** — accessed 2026-06-08 — high
- **What they contributed:** macOS identity = inode (`NSFileSystemFileNumber`) / security-scoped bookmark (path + per-component inodes + volume UUID). Survives rename/intra-volume move; **breaks on copy, cross-volume move, atomic "safe" save (new inode every save), and cloud-sync recreation.** Live moves followed via `NSFilePresenter`; bookmarks persisted for re-acquisition across launches.
- **[Microsoft change journal / USN](https://learn.microsoft.com/windows/win32/fileio/change-journals)** + [`FILE_OBJECTID_INFORMATION`](https://learn.microsoft.com) — **primary** — accessed 2026-06-08 — high — NTFS **FileID (FRN)** stable across rename/intra-volume move; **ObjectID** for cross-location tracking but may change on copy/move; both break on copy/off-volume. USN correlates rename old→new by FRN.
- **[git-diff rename detection](https://git-scm.com/docs/git-diff)** — **primary** — accessed 2026-06-08 — high — rename/copy detection by **content similarity** (default 50%, `-M90%`, `-M100%` exact). The precedent for "same file, edited" via content. Similarity hashing (MinHash/LSH/SimHash) is the scale technique.

## Synthesis

Universal pattern across every mature system: a **stable UUID/GUID identity created once** (XMP `DocumentID`, Word `docId`, PDF `/ID[0]`, EPUB `dc:identifier`) + a **per-save version ID** (XMP `InstanceID`, PDF `/ID[1]`, Word per-session `rsid`), with **lineage expressed as a reference edge** to the predecessor (XMP `DerivedFrom`/`OriginalDocumentID`, Word shared `docId`/`rsidRoot`, **C2PA parent ingredient**). Format conversion is the hard case — only `OriginalDocumentID` and C2PA ingredients are *designed* to survive it.

Mapped to humanshipd's B+C+D, by **authority tier**:
- **D — embedded GUID = authoritative identity.** Adopt the **XMP-MM two-tier model** (DocumentID = our session-accumulation key, created once; InstanceID per save; OriginalDocumentID for conversion survival) — which **C2PA already reuses**, so it's in-standard, not bespoke. Respect native carriers where present (Word `docId`+`rsidRoot`, PDF `/ID`, EPUB `dc:identifier`). Express "new version of document X" as a **C2PA parent ingredient**. Only D may *assert* identity.
- **B — OS file identity = fast hint, never authoritative.** Security-scoped bookmark (macOS) / FRN+ObjectID+USN (Windows) / (dev,ino) (Linux), to cheaply locate the file and follow live moves. Advisory — atomic saves, copies, cross-volume moves, and cloud sync all break it.
- **C — content fingerprint = confirmable fallback.** Exact hash (instant same-content) + similarity (our ISCC / SimHash) to *propose* a re-link when B is broken and D absent. Never asserts identity alone.

**Resolution order D → B → C, with reconcile-on-disagreement:**
1. D matches, content wildly different → **same document, new version** (trust D — rescues the "heavily edited" case content-similarity would miss).
2. Content matches, D differs/absent → **copy / Save-As / fork** → don't merge sessions; keep the distinct GUID.
3. B matches but D differs → **inode/FRN reuse or overwrite** → D wins, discard B.
4. D matches but B points elsewhere → **expected after atomic save/move/sync** → refresh B from the D-confirmed file.
- **Invariant:** a fast/content signal may *propose* a link; only a verified embedded GUID may *assert* identity; conflicts resolve to the cryptographically strongest verifiable signal.

**Caveat:** embedded IDs aren't tamper-proof (rsid/docId editable, XMP strippable) — they establish *identity and lineage*, not *integrity*; the signing/attestation layer (Decision 6) must bind over the combined `DocumentID + OS identity + content fingerprint` triple.

## Downstream uses

- Decision 7 (native-file document identity) — the layered B+C+D model + resolution order.
- Harmonizes with: Decision 4 (ISCC content identity = the "C" fingerprint), Decision 3/§9 (C2PA ingredients for lineage), the C2PA/XMP adoption stance.
