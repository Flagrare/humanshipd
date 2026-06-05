# Research: App-Agnostic, On-Device Capture Architecture (Cluely-class)

- **Slug:** `2026-06-05-app-agnostic-capture-architecture`
- **Date:** 2026-06-05
- **Status:** complete
- **Triggered by:** Founder rejected the "make writers switch to our editor" model — wants the tool to BLEND INTO existing apps (Word, Google Docs, Scrivener, Final Draft, scriptwriting) and be extensible, citing Cluely ("runs at OS level, reads other apps — why can't we?"). Plus hard constraints added mid-discussion: open-source, local-only, zero telemetry, at most a stateless tamper-check server.
- **Informed:** Capture-layer design decision (chat, 2026-06-05) and forthcoming design spec. Companion: [`2026-06-05-proof-of-human-authorship-feasibility.md`](./2026-06-05-proof-of-human-authorship-feasibility.md) (which under-covered the screen/accessibility path).

## Question

Is an app-agnostic, OS-level capture layer feasible that observes a user's writing across arbitrary apps, runs entirely on-device (no cloud, no telemetry), and can be open-sourced? Which capture modality (or combination) is most feasible across macOS/Windows/Linux, and what are the honest limits?

## Sources

### Open-source precedents (the architecture ships)
#### [screenpipe](https://github.com/mediar-ai/screenpipe) ([architecture](https://docs.screenpi.pe/architecture), [license/privacy](https://screenpi.pe/privacy))
- **Authors / Org:** mediar-ai (YC S26)
- **Type:** OSS repo (Rust; MIT core, `ee/` enterprise dir → repo marked NOASSERTION)
- **Published:** ongoing · **Accessed:** 2026-06-05
- **Relevance:** high (the closest blueprint)
- **What this contributed:** ~19k-star production tool doing local 24/7 capture with an **accessibility-tree walk as the primary text source and OCR as fallback** (Apple Vision / Windows OCR / Tesseract), event-driven triggers (app switch, UI interaction, typing-pause debounce, ~5s idle), local SQLite at `~/.screenpipe`. Directly validates the recommended AX-first architecture and local-only viability. Caveat: ships **anonymous analytics** — must be stripped to meet the zero-telemetry mandate.

#### [Glass by Pickle](https://github.com/pickle-com/glass) · [cheating-daddy](https://github.com/sohzm/cheating-daddy)
- **Authors / Org:** Pickle (Daniel Park); sohzm
- **Type:** OSS repos (Electron/JS, both GPL-3.0)
- **Published:** 2025 · **Accessed:** 2026-06-05
- **Relevance:** medium
- **What this contributed:** "Open-source Cluely" (~7.5k stars) and a sibling (~5.3k stars) prove the always-on, "invisible" screen+audio capture pattern as OSS — confirming a low barrier to the capture layer. **GPL-3.0 copyleft** is a licensing consideration if their code is reused (our project would likely prefer MIT/Apache, so reuse screenpipe's patterns, not Glass's code).

#### [Tearing down the Rewind app](https://kevinchen.co/blog/rewind-ai-app-teardown/) · [Rewind specs / Meta acquisition](https://ucstrategies.com/news/rewind-ai-mac-memory-search-tool-specs-privacy-pricing-2026/)
- **Authors / Org:** Kevin Chen; UC Strategies / Rewind
- **Type:** blog teardown; news/vendor
- **Published:** ~2023 / 2026 · **Accessed:** 2026-06-05
- **Relevance:** medium-high (best-documented pipeline)
- **What this contributed:** Detailed teardown of the exact on-device pipeline: ScreenCaptureKit selective capture, accessibility APIs for frontmost-window/bundle-ID, Vision OCR per frame at 0.5 fps → H.264, SQLite/FTS4, on-launch screen+mic+accessibility prompts (~180 MB/hr). Closed-source comparison; **acquired by Meta Dec 2025, capture disabled Dec 19 2025** — leaving screenpipe as the OSS successor.

### macOS capture & accessibility
#### [AXUIElement](https://developer.apple.com/documentation/applicationservices/axuielement) · [AXIsProcessTrustedWithOptions](https://developer.apple.com/documentation/applicationservices/1459186-axisprocesstrustedwithoptions) · [worked example](https://macdevelopers.wordpress.com/2014/01/31/accessing-text-value-from-any-system-wide-application-via-accessibility-api/)
- **Authors / Org:** Apple; mac developers blog
- **Type:** vendor doc; blog
- **Published:** ongoing / 2014 · **Accessed:** 2026-06-05
- **Relevance:** high (the clean primary text source on macOS)
- **What this contributed:** System-wide AXUIElement + focused-element/value/selected-text attributes read live text of arbitrary apps' fields — cleaner and lower-cost than OCR. `kAXTrustedCheckOptionPrompt` drives the permission flow. Long-standing, practical technique.

#### [Accessibility permission requires sandbox off](https://jano.dev/apple/macos/swift/2025/01/08/Accessibility-Permission.html) · [ScreenCaptureKit](https://developer.apple.com/forums/tags/screencapturekit) · [Sequoia screen-recording re-prompt / Persistent Content Capture entitlement](https://mjtsai.com/blog/2024/08/08/sequoia-screen-recording-prompts-and-the-persistent-content-capture-entitlement/)
- **Authors / Org:** Jano; Apple; Michael Tsai (citing Apple/Craig Hockenberry)
- **Type:** blog; vendor doc/forum; blog
- **Published:** 2025 / ongoing / 2024 · **Accessed:** 2026-06-05
- **Relevance:** high (the macOS friction)
- **What this contributed:** Accessibility is a TCC grant separate from Screen Recording and **requires App Sandbox OFF → cannot ship on the Mac App Store** (Developer ID + notarization instead; unsigned apps fail TCC registration). Sequoia re-prompts Screen Recording ~monthly with no permanent-allow; the silencing "Persistent Content Capture" entitlement is restricted to VNC apps via Apple approval — so always-on screen capture has real consent friction (a reason to prefer AX over screenshots).

#### [VNRecognizeTextRequest (Vision OCR)](https://developer.apple.com/documentation/vision/vnrecognizetextrequest) · [CGEventTap / EventTapper](https://github.com/usagimaru/EventTapper)
- **Authors / Org:** Apple; usagimaru
- **Type:** vendor doc; OSS repo
- **Published:** ongoing · **Accessed:** 2026-06-05
- **Relevance:** high
- **What this contributed:** On-device Vision OCR (offline, >95% on clear text, same engine as Live Text) = the macOS OCR fallback. CGEventTap (Input Monitoring for keystrokes; Accessibility for global NSEvent monitors) = the mechanism to observe keystroke *activity/timing* and correlate it with text deltas to distinguish **paste from typing**.

### Windows capture & accessibility
#### [Windows.Graphics.Capture / DXGI Desktop Duplication](https://learn.microsoft.com/en-us/windows/win32/direct3ddxgi/desktop-dup-api) · [Windows.Media.Ocr](https://learn.microsoft.com/en-us/uwp/api/windows.media.ocr) · [UI Automation](https://learn.microsoft.com/en-us/windows/apps/dev-tools/winapp-cli/ui-automation) · [Chrome UIA default](https://developer.chrome.com/blog/windows-uia-support-update)
- **Authors / Org:** Microsoft; Google Chrome team
- **Type:** vendor doc; blog
- **Published:** ongoing (Chrome native UIA ~v138) · **Accessed:** 2026-06-05
- **Relevance:** high (Windows is the least-intrusive OS here)
- **What this contributed:** WGC (Win10 1803+) is the modern capture path; built-in offline `Windows.Media.Ocr` works but **requires MSIX package identity**. **UIA** reads text across WPF/WinForms/Win32/WinUI/Electron (Electron needs `--force-renderer-accessibility`); Chromium enables native UIA by default since Chrome 138. UIA reads generally need no scary prompt — the best per-app text coverage and lightest permission UX of the three OSes.

### Linux capture & accessibility
#### [XDG ScreenCast portal](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.ScreenCast.html) · [restore-token impl (OBS PR #5559)](https://github.com/obsproject/obs-studio/pull/5559) · [AT-SPI2](https://www.freedesktop.org/wiki/Accessibility/AT-SPI2/) · [Wayland a11y notes / Newton](https://github.com/splondike/wayland-accessibility-notes)
- **Authors / Org:** freedesktop/Flatpak; OBS (Georges Stavracas); GNOME Accessibility; splondike
- **Type:** spec; OSS PR; spec; OSS notes/blog
- **Published:** ongoing / 2021 / 2024 · **Accessed:** 2026-06-05
- **Relevance:** medium-high (Linux is the fragile case)
- **What this contributed:** Wayland screencast = portal + PipeWire; **restore tokens (portal v4+) enable persistent re-capture without re-prompting** (proven in OBS). AT-SPI2 (D-Bus) text extraction works mainly under **GNOME**, KDE improving, others unreliable; Newton targets sandbox-compatible Wayland a11y but isn't mature. X11 has full unguarded access but is deprecated. Net: workable but the least reliable OS.

### Google Docs: a blind spot for OS-level AX, but NOT for in-page extensions
> **Correction (2026-06-05):** an earlier version of this doc framed Google Docs as a flat "blind spot forcing OCR." That is true only for an *external, OS-level* tool. The standard, proven way to capture Docs is an **in-page browser extension** (the method Aidify, Grammarly Authorship, and Draftback all use), which sidesteps the canvas problem entirely and yields the richest signal of all.

#### [Google Docs switches to canvas rendering](https://thenewstack.io/google-docs-switches-to-canvas-rendering-sidelining-the-dom/) ([WebAIM](https://webaim.org/blog/seismic-change-to-docs/))
- **Authors / Org:** The New Stack; WebAIM
- **Type:** news; blog
- **Published:** 2021 · **Accessed:** 2026-06-05
- **Relevance:** high (defines the OS-level limit)
- **What this contributed:** Google Docs renders text to canvas — the visible text is **not in the DOM or the standard accessibility tree**; a hidden a11y DOM appears only when the user enables screen-reader mode. So *external OS-level AX* extraction of Docs is unreliable by default. This bounds the OS-level adapter — it does **not** bound the in-page-extension adapter below.

#### [How I reverse-engineered Google Docs to play back any document's keystrokes](https://features.jsomers.net/how-i-reverse-engineered-google-docs/) ([Draftback](https://draftback.com/))
- **Authors / Org:** James Somers
- **Type:** blog + tool (Chrome extension)
- **Published:** ~2015 (ongoing) · **Accessed:** 2026-06-05
- **Relevance:** high (the way *around* the canvas problem)
- **What this contributed:** Demonstrates that Google Docs stores a **complete per-character revision history** (persistent per-character IDs) that an **in-page browser extension** can fetch and replay keystroke-by-keystroke — richer than Accessibility or OCR. The revision fetch hits *Google's* endpoint for the user's *own* document and is processed locally (Draftback sends nothing to a third-party server), so it is compatible with a local-only / zero-telemetry design (with that nuance noted). This is the proven capture path for Docs and the model Aidify/Grammarly Authorship also follow.

### Avoiding a second codebase for Google Docs — exhausted alternatives + the drift fix
> Investigated 2026-06-05 in response to the maintenance concern that a Docs browser extension + a native adapter = two codebases that drift.

**All single-codebase capture paths for Docs fail; the only fine-grained signal lives in the page. The fix is architectural: one shared Rust core, with the browser side reduced to a thin event-forwarding shim.**

- **OS-level accessibility of Docs** — works only if the user manually enables Docs "screen reader support" (off by default), and even then exposes coarse text/caret via a hidden side-DOM, not an ordered per-edit/keystroke stream. Impractical + fragile. ([Google screen-reader help](https://support.google.com/docs/answer/6282736?hl=en); a11y context [WebAIM](https://webaim.org/blog/seismic-change-to-docs/))
- **Official Google APIs** — [Drive `revisions` v3](https://developers.google.com/workspace/drive/api/reference/rest/v3/revisions) are coarse saved-snapshots (30-day auto-purge, ~200 cap, [incomplete vs editor history](https://hawksey.info/blog/2022/07/working-with-the-google-drive-api-revisions-history-tips-for-handling-revision-merges/)). No edit-level API exists. Only fine-grained source = Docs' undocumented realtime mutation feed (Draftback), which is in-page only.
- **Chrome DevTools Protocol** — [Chrome 136 (Mar 2025) stopped honoring `--remote-debugging-port` on the default profile](https://developer.chrome.com/blog/remote-debugging-port) (anti-cookie-theft). Can't attach to the user's real logged-in Chrome without a separate profile = unacceptable friction + malware optics. Reject for consumers.

#### [Native messaging (Chrome)](https://developer.chrome.com/docs/extensions/develop/concepts/native-messaging) ([MDN](https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/Native_messaging), [friction analysis](https://textslashplain.com/2020/09/04/web-to-app-communication-the-native-messaging-api/))
- **Authors / Org:** Google; Mozilla; Eric Lawrence (ex-Chrome/Edge)
- **Type:** vendor doc; blog
- **Published:** maintained / 2020 · **Accessed:** 2026-06-05
- **Relevance:** high (the bridge that keeps logic in one place)
- **What this contributed:** stdio-JSON (length-prefixed UTF-8) protocol letting a thin extension forward raw events to a full-privilege native host. User-level (HKCU/`~/Library`) install avoids admin but is per-user. Basis for "extension is a dumb tap → native Rust core signs."

#### Shared-core precedents: [1Password (Serokell)](https://serokell.io/blog/rust-in-production-1password) ([Syntax transcript](https://syntax.fm/show/776/how-1password-uses-wasm-and-rust-for-local-first-dev-with-andrew-burkhart/transcript)) · [Bitwarden SDK](https://contributing.bitwarden.com/architecture/sdk/internal/web/interoperability/) · [Tailscale ts-browser-ext](https://github.com/tailscale/ts-browser-ext)
- **Authors / Org:** Serokell/1Password; Bitwarden; Tailscale
- **Type:** blog / OSS docs / OSS repo
- **Published:** recent / ongoing · **Accessed:** 2026-06-05
- **Relevance:** high (proves the drift fix at scale)
- **What this contributed:** 1Password & Bitwarden ship one Rust core compiled to **WASM** for the extension, with **Typeshare**/**tsify** auto-generating TS types so the Rust↔TS boundary can't drift. Tailscale's extension drives a native binary over Native Messaging because "extensions don't have enough APIs." Two production patterns: WASM-core (self-contained extension) or Native-Messaging-to-native-core (dumb extension). For this project the latter is the stronger drift-killer since a native companion ships anyway.

## Synthesis

**An app-agnostic, on-device, open-source capture layer is feasible and has shipped (screenpipe, Rewind, Glass; and, for web editors, Draftback/Aidify/Grammarly). The capture is the easy ~80%; the attestation semantics on top are the hard, valuable, partly-unsolved 20%.** The most feasible architecture is a **portfolio of per-surface adapters behind one common record format** — not a single universal hook. Each adapter uses the best available method for its surface:

1. **Browser-extension adapter** (in-page content script) — **Google Docs** (via the per-character revision history / live edit events, à la Draftback) **and any web editor**. Highest reach, lowest friction, proven by Aidify/Grammarly/Draftback. Local-only compatible (process in-browser; hand the record to a local companion app via native messaging). The richest signal for Docs — it does NOT need OCR.
2. **OS Accessibility adapter** (macOS AXUIElement, Windows UIA, Linux AT-SPI/GNOME) — **native desktop apps** (Word desktop, Scrivener, Final Draft). Reads the focused text element's content + change events from an allow-listed, in-focus app. Cleaner and more privacy-respecting than screenshots.
3. **Input-timing correlation** (timing only, never content; macOS CGEventTap, equivalents elsewhere) — distinguishes text that appeared *with* keystrokes (typed) from *without* (pasted): the AI-dump detector. Layered under adapters 1–2 where available.
4. **Screenshot + on-device OCR** (Vision / Windows.Media.Ocr / Tesseract) — genuine **last-resort fallback** only, for apps with no extension and poor AX coverage.

**Per-OS reality (for the OS Accessibility adapter):** Windows is the least intrusive (UIA needs no scary prompt; OCR needs MSIX). macOS is the highest friction (Accessibility + Input Monitoring grants, sandbox off → **no Mac App Store**, Developer ID + notarization, periodic Sequoia re-prompts). Linux works under GNOME/Wayland with restore tokens but is the least reliable across DEs. The **browser-extension adapter is OS-independent**, which is part of why it is the most feasible first target.

**Honest limits to design around (not hide):**
- **Cannot defeat "human retypes AI text"** — incremental human typing of AI output reads as genuine. No capture modality solves this (consistent with the feasibility study's information-theoretic conclusion).
- **What it CAN catch:** wholesale paste / instant large insertions with no corresponding keystrokes; authorship discontinuities. These are the defensible claims.
- **Browser-extension caveats:** browser-scoped (native apps still need the AX adapter); Draftback-style revision fetch hits Google's endpoint for the user's own doc (processed locally, nothing sent to us — note this nuance for the local-only claim).
- **Granularity:** screenshot diffing only resolves bursts; per-keystroke timing needs input monitoring (extra permission) or an in-page/AX edit-event stream.
- **Permission irony:** the OS-level adapters need spyware-grade permissions. Mitigation for a privacy-first OSS project: auditable code, AX-over-screenshots, allow-list only the focused writing app, store only hashes/timestamps, zero network except the stateless tamper-check.

**Drift mitigation (the two-codebases concern):** you cannot delete browser-side capture for Docs (fine-grained edit data is in-page only), but you can delete the *second codebase* in any meaningful sense. Put 100% of record/hash/sign/verify in **one Rust core** compiled native; the browser extension becomes a thin **MV3** event tap forwarding raw events over **Native Messaging** to that core (Tailscale pattern), holding zero credential logic. Drift drops to only the irreducible per-surface capture shims. (WASM-core + Typeshare/tsify, the 1Password/Bitwarden pattern, is the alternative if a self-contained offline-signing extension is ever wanted.)

**Licensing note:** prefer reusing screenpipe's *patterns* (MIT core) over Glass/cheating-daddy *code* (GPL-3.0 copyleft).

## Downstream uses

- Capture-layer recommendation delivered to founder (chat), 2026-06-05.
- Design spec (capture architecture §3): [`docs/superpowers/specs/2026-06-05-human-authorship-attestation-design.md`](../superpowers/specs/2026-06-05-human-authorship-attestation-design.md).
