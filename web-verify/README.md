# humanshipd web verify (WASM)

A static page that verifies a Human Authored credential **entirely in your
browser**. It compiles `humanshipd-core`'s verification to WebAssembly, so the
browser reaches the same verdict — and shows the same honest claim — as the
command-line `verify_credential` example. The document and credential never leave
the page.

It's a detached crate (its own `[workspace]`) because it builds the core with the
`wasm` feature (c2pa's pure-Rust crypto), which must not unify with the native
OpenSSL build used by the rest of the workspace.

## Build

```bash
# from web-verify/
wasm-pack build --target web --out-dir pkg
```

This produces `pkg/humanshipd_web_verify.js` + `..._bg.wasm`, which `verify.html`
imports.

## Use

Serve the folder and open the page (a static file server is needed so the browser
can fetch the `.wasm`):

```bash
python3 -m http.server 8000     # from web-verify/
# open http://localhost:8000/verify.html
```

Pick a credential (`.c2pa`) and the document it should belong to, click **Verify**,
and the page shows the verdict, the claim, the bound document hash, the count of
flagged un-keyed (paste) insertions, a **banded provenance report** — word-count
proportions of how the text entered (typed / pasted / not captured) with a coarse
summary — and a **writing fingerprint graph**: edit position over time when the credential
carries per-edit offsets (a paste is a vertical jump, a revisit to earlier text
dips back down), falling back to document length otherwise. The graph has a
**scrubbable replay** (play/pause, speed) and **jump-to-paste markers**, all
content-free (offsets, counts, timestamps — never text). Finally, a muted
**process-shape** panel offers weak, positive-only corroboration of a human-like
drafting rhythm (planning pauses, revisions, bursts) — it can affirm "consistent
with incremental composition" but never says "looks like AI", and its absence is
explicitly not evidence of AI. The bands describe *provenance*, not a guess about
whether the text is AI-written, and the report, graph, and panel appear only when
the credential is valid for the document. If the credential carries a
**self-asserted author name**, it is shown labeled "not independently verified" —
the name is signed (tamper-evident) but a local-only tool cannot attest identity.

To generate a fixture for manual testing:

```bash
cargo run --example issue_credential -- <out-dir>   # from repo root
# writes <out-dir>/credential.c2pa + <out-dir>/document.txt (a mixed typed+pasted session)
```

## Validation

Smoke-tested in a real Chromium (via the Playwright MCP): a credential checked
against its own document returns `valid`, shows the AI-paste warning in the claim,
and renders the provenance bands ("Typed, with some pastes" — Typed 62% / Pasted
38%) plus the fingerprint graph (a typed slope with a vertical jump at the paste);
checked against a *different* document it returns `INVALID` and renders **no** bands
or graph (they would describe the credential's own session, not the document on
screen). The verification, report, and timeline logic — `humanshipd-core::read_sidecar`
+ `render_report` + the derived `timeline` compiled to WASM — is regression-covered
by the core's Rust test suite (`cargo test -p humanshipd-core`), which issues,
verifies, and reports on fresh credentials each run (no fragile committed fixtures
to expire).

