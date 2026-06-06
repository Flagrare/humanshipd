// Popup: grab the current writing session from the active tab, ask the host to
// issue a credential, and download the resulting .c2pa file.

const button = document.getElementById("issue");
const status = document.getElementById("status");

function show(text, cls) {
  status.textContent = text;
  status.className = cls || "";
}

button.addEventListener("click", async () => {
  show("Collecting writing session…");
  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
  if (!tab?.id) {
    show("No active tab.", "err");
    return;
  }

  let session;
  try {
    session = await chrome.tabs.sendMessage(tab.id, { type: "getSession" });
  } catch {
    show("Could not reach the page. Open a web editor and type first.", "err");
    return;
  }
  if (!session || !session.events?.length) {
    show("No writing captured yet — type in a text field, then try again.", "err");
    return;
  }

  show("Issuing credential via local host…");
  const author = document.getElementById("author").value.trim();
  const result = await chrome.runtime.sendMessage({ type: "issue", session, author });
  if (!result?.ok) {
    show(`Host error: ${result?.error || "unknown"}`, "err");
    return;
  }

  await chrome.downloads.download({
    url: `data:application/octet-stream;base64,${result.manifest_b64}`,
    filename: "humanshipd-credential.c2pa",
    saveAs: true,
  });
  show("Credential issued and downloaded.", "ok");
});
