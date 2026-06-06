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
and the page shows the verdict, the claim, the bound document hash, and the count
of flagged un-keyed (paste) insertions.

## Validation

Smoke-tested in a real Chromium (via the Playwright MCP): a credential checked
against its own document returns `valid` with the AI-paste warning in the claim;
checked against a *different* document it returns `INVALID`. The verification
logic itself — being just `humanshipd-core::read_sidecar` compiled to WASM — is
regression-covered by the core's Rust test suite (`cargo test -p humanshipd-core`),
which issues and verifies fresh credentials each run (no fragile committed
fixtures to expire).

