// Web Worker: sign a Human Authored credential entirely in the browser, off the
// popup's thread. This replaces the native messaging host — it runs the *same*
// core Rust issuance (humanshipd_core::credential::issue_sidecar_with_author),
// compiled to WASM, that the host called. No separate process, no install step.
// Also issues from an accumulated capture log (multi-session, e.g. Google Docs).

import init, { issue_credential, issue_from_capture_log, reconstruct_text_from_log } from "./pkg/humanshipd_web_verify.js";

let ready = null;
const ensureReady = () => (ready ||= init());

self.onmessage = async (event) => {
  try {
    await ensureReady();
    const { log, session, author } = event.data;
    if (log) {
      const logJson = JSON.stringify(log);
      const manifest = issue_from_capture_log(logJson, author || undefined);
      const text = reconstruct_text_from_log(logJson);
      self.postMessage({ ok: true, manifest, text }, [manifest.buffer]);
    } else {
      const manifest = issue_credential(JSON.stringify(session));
      self.postMessage({ ok: true, manifest }, [manifest.buffer]);
    }
  } catch (e) {
    self.postMessage({ ok: false, error: String(e && e.message ? e.message : e) });
  }
};
