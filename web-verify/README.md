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
flagged un-keyed (paste) insertions, and a **banded provenance report** — word-count
proportions of how the text entered (typed / pasted / not captured) with a coarse
summary. The bands describe *provenance*, not a guess about whether the text is
AI-written, and they appear only when the credential is valid for the document.

To generate a fixture for manual testing:

```bash
cargo run --example issue_credential -- <out-dir>   # from repo root
# writes <out-dir>/credential.c2pa + <out-dir>/document.txt (a mixed typed+pasted session)
```

## Validation

Smoke-tested in a real Chromium (via the Playwright MCP): a credential checked
against its own document returns `valid`, shows the AI-paste warning in the claim,
and renders the provenance bands ("Typed, with some pastes" — Typed 62% / Pasted
38%); checked against a *different* document it returns `INVALID` and renders **no**
bands (they would describe the credential's own session, not the document on
screen). The verification and report logic — `humanshipd-core::read_sidecar` +
`render_report` compiled to WASM — is regression-covered by the core's Rust test
suite (`cargo test -p humanshipd-core`), which issues, verifies, and reports on
fresh credentials each run (no fragile committed fixtures to expire).

