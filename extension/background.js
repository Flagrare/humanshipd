// Service worker: relays an "issue" request from the popup to the local
// humanshipd native host, and returns the resulting C2PA credential.
// Holds no credential logic — it's a thin pipe to the host.

const HOST = "dev.humanshipd.host";

chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
  if (message?.type !== "issue") return false;

  const request = {
    type: "issue",
    session_id: message.session.session_id,
    surface_kind: message.session.surface_kind,
    surface_app: message.session.surface_app,
    final_text: message.session.final_text,
    events: message.session.events,
    author: message.author || null,
  };

  chrome.runtime.sendNativeMessage(HOST, request, (response) => {
    if (chrome.runtime.lastError) {
      sendResponse({ ok: false, error: chrome.runtime.lastError.message });
      return;
    }
    if (response?.type === "credential") {
      sendResponse({ ok: true, manifest_b64: response.manifest_b64 });
    } else {
      sendResponse({ ok: false, error: response?.message || "host error" });
    }
  });

  return true; // keep the message channel open for the async response
});
