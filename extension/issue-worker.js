// Web Worker: sign a Human Authored credential entirely in the browser, off the
// popup's thread. This replaces the native messaging host — it runs the *same*
// core Rust issuance (humanshipd_core::credential::issue_sidecar_with_author),
// compiled to WASM, that the host called. No separate process, no install step.

import init, { issue_credential } from "./pkg/humanshipd_web_verify.js";

let ready = null;
const ensureReady = () => (ready ||= init());

self.onmessage = async (event) => {
  try {
    await ensureReady();
    // event.data is the captured session (+ optional author). issue_credential
    // returns the .c2pa manifest bytes as a Uint8Array.
    const manifest = issue_credential(JSON.stringify(event.data));
    self.postMessage({ ok: true, manifest }, [manifest.buffer]);
  } catch (e) {
    self.postMessage({ ok: false, error: String(e && e.message ? e.message : e) });
  }
};
