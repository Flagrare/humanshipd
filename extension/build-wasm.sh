#!/usr/bin/env bash
# Build the WASM bundle the extension uses to sign + verify credentials in-browser.
# Run this once before loading the extension, and again after changing the Rust in
# core/ or web-verify/. No native host, no install step — just this.
#
# Usage: bash extension/build-wasm.sh   (requires wasm-pack)
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
wasm-pack build "$ROOT/web-verify" --target web --out-dir "$ROOT/extension/pkg"
echo "Built extension/pkg — now load extension/ unpacked at chrome://extensions."
