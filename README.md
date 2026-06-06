# humanshipd

*Working codename. The thing it issues is called a **Human Authored** credential.*

**Status: research-grade preview.** The core works and is tested; the everyday apps around it are still being built (see [Project status](#project-status)).

Imagine you wrote something yourself — an essay, a chapter, a script — and later someone asks, "how do I know a person wrote this and not an AI?" Today you have no good answer. humanshipd gives you one. As you write, it quietly notes *how* the writing happened — the typing, the pauses, the edits — and turns that into a tamper-proof certificate you can attach to your finished document. Anyone can then check the certificate against the document, with no account and nothing sent to anyone.

It's the mirror image of a tool like [SynthID](https://deepmind.google/technologies/synthid/), which marks text so AI output can be *detected*. humanshipd instead vouches for the *human* writing process behind a document.

## Read this first: it attests, it doesn't "prove"

We are deliberately honest about what this can and can't do, because overpromising here would be dishonest — and because the project is open source, anyone can check our claims.

A motivated cheater can still defeat it: paste AI text onto one screen and slowly retype it onto another, and the typing looks genuine because it *is* genuine typing. No tool that watches the keyboard can tell that apart from real composition — that's a mathematical fact, not an engineering gap, and we don't pretend otherwise. Because the code is open, someone could also modify it to emit a fake certificate.

So what is the certificate actually worth? It **raises the cost of faking** authorship from "click a button" to "tediously retype everything," it **catches the lazy cheat** — pasting a wall of AI text appears as a large chunk of writing that arrived without any typing, which the certificate flags — and it gives an **honest writer real evidence** to point to. And once a certificate is issued, any later tampering with it or the document is detectable. Think of it as a credential and a deterrent, not a lie detector. The complete, unflinching list of attacks it can and can't withstand is in the [threat model](docs/superpowers/specs/2026-06-05-human-authorship-attestation-design.md#4-threat-model-published-openly).

## How it works, in plain terms

Five steps turn an act of writing into a checkable certificate.

**1. It watches the writing happen — without keeping the words.** While you write in an app like Word or TextEdit, humanshipd observes the text through your operating system's **Accessibility features** — the same built-in machinery a screen reader uses to read what's on screen aloud. It records only the *shape* of the process (when text appeared, how much, whether typing or a paste produced it), never the content. Your actual words never leave your computer.

**2. It summarizes the process.** From those observations it builds a small, content-free summary: how the writing came in bursts, where the long pauses were, how much was inserted versus deleted, and — the important tell — any large chunk that appeared with no corresponding typing, which is what an AI paste looks like.

**3. It seals that summary into a Content Credential.** The summary is packaged using **C2PA** (the Coalition for Content Provenance and Authenticity) — the open, cross-industry standard for "where did this content come from," backed by Adobe, the BBC, Microsoft and others, and the same technology behind the "Content Credentials" you may have seen on AI-generated images. The credential is cryptographically signed, so any later change to it is detectable.

**4. It ties the certificate to your exact document.** Rather than storing your document, the credential carries a **fingerprint** of it — a SHA-256 hash, a short code where changing a single character changes the whole code. That binds the certificate to *that* file and nothing else.

**5. Anyone can verify it.** A reader checks the certificate against the document: the signature confirms nothing was altered, and the fingerprint confirms it's the right document. No sign-in, no server call, no data collected.

## Try it

Nothing is packaged for download yet, but the whole loop runs from the repo. To see it end-to-end without typing a word, generate a demo credential and open the in-browser verifier (you'll need Rust and [`wasm-pack`](https://rustwasm.github.io/wasm-pack/)):

```bash
# 1. generate a demo credential and the document it's bound to
cargo run --example issue_credential -- /tmp/demo

# 2. build the in-browser verifier and serve it
cd web-verify && wasm-pack build --target web --out-dir pkg && python3 -m http.server 8000
```

Open `http://localhost:8000/verify.html`, drop in `/tmp/demo/credential.c2pa` and `/tmp/demo/document.txt`, and click **Verify**. You'll see the verdict, the provenance report (this demo is "typed, with some pastes"), and the writing fingerprint with its replay — all computed in your browser, with the files never leaving the page.

## What we believe (and build by)

These aren't slogans; they're constraints the code actually obeys.

- **Open source.** Every claim is auditable, including the embarrassing limitations above.
- **Local-only.** Your writing stays on your machine. Only fingerprints (hashes) ever travel.
- **Zero telemetry.** No tracking, no accounts, no analytics — none.
- **Not for sale.** This is meant as public infrastructure, not a product.
- **Every research claim is sourced.** The decisions behind the design are cataloged with citations in [`docs/research/`](docs/research/).

## How the certificate can travel with your work

A certificate is only useful if it stays attached to the document. There are three ways it can, in increasing order of robustness — the first is built today, the others are planned:

The simplest is a **sidecar file** — a small `.c2pa` companion that travels next to your document and binds to it by fingerprint. This works for any file type (a PDF, an EPUB, a `.txt`) and is what's implemented now.

For plain text that needs to carry its proof *inside itself*, the credential can be woven into the text as **invisible characters** (a method defined in the C2PA standard using non-printing Unicode marks). The text looks identical and survives copy-paste, but now carries its own certificate. The encoder for this is built and tested.

The most durable approach adds a **content fingerprint** (using ISCC, an ISO-standard "content code" that recognizes a document even after light edits or reformatting) plus an opt-in lookup service, so a stripped-down copy can still be matched back to its certificate. The fingerprint is now built into every credential; the lookup service is the only piece that involves a server, and even then it only ever sees fingerprints, never your text.

## Architecture

One piece holds all the logic that matters; everything else is a thin shell around it.

```
   Where you write                      The brain                  What you get
 ┌─────────────────────┐         ┌──────────────────────┐       ┌──────────────┐
 │ Browser (Docs/web)  │──┐      │   Rust core          │       │ .c2pa        │
 │ Native apps         │  ├────▶ │   • summarize process│ ────▶ │ credential   │
 │  (Word, Scrivener…) │──┘      │   • sign as C2PA     │       │ (+ verify)   │
 │ via Accessibility   │         │   • bind by fingerprint│     └──────────────┘
 └─────────────────────┘         │   • verify           │
                                 └──────────────────────┘
```

The **core** is written in Rust and contains the entire credential format, signing, and verification, so a credential made by one capture method is byte-for-byte identical to one made by another. Because that core — along with the host and the registry — is plain Rust with no OS-specific code, it builds and runs on macOS, Windows, and Linux alike; only the capture adapters are platform-specific. Those adapters are deliberately thin: a **macOS Accessibility adapter** for native apps like Word and Scrivener, and a **browser extension** for ordinary web editors, are both built; **Windows** (UI Automation) and **Linux** (AT-SPI) native adapters are planned, as is a dedicated path for Google Docs (its `<canvas>` rendering hides text from the browser extension). Where an app exposes nothing else, screenshots with on-device text recognition are the last-resort fallback.

## Project status

This is an early, honest preview. The credential engine and the verify experience are real and tested; the capture side — the apps you'd run *while* writing — is still rough.

**Working today:** the Rust core — building the process summary, issuing a signed C2PA credential bound to a file, verifying it, embedding a credential invisibly in text, and computing the ISCC durable fingerprint. An opt-in registry service for fingerprint → credential lookup, with end-to-end recovery proven. A **macOS capture tool** that runs the whole slice for real: it reads your live typing in TextEdit or Word, issues a signed credential bound to the document, and verifies it. A **browser extension** that captures the same way in ordinary web editors (typed vs. pasted, with caret position) and issues a credential through a local host, optionally attaching a self-asserted author name.

And a **browser verify page** that checks a credential entirely client-side — the very same verification logic compiled to WebAssembly, so the browser shows the same verdict as the command line. When a credential is valid it also reads back, from the signed record:

- a **provenance report** — what share of the words were typed, pasted, or never captured (word-count proportions, not a guess about AI);
- a **writing fingerprint** — edit position over time, where a paste is a vertical jump and a revisit dips back, with a scrubbable replay and jump-to-paste markers;
- a **process-shape** panel — weak, positive-only corroboration of a human-like drafting rhythm (it never claims "this is AI");
- a **self-asserted author name**, shown plainly as "not independently verified";
- and a one-click **Save as PDF / print** for a shareable report.

The whole workspace builds on macOS, Windows, and Linux.

**Not built yet:** native Windows/Linux capture adapters, a dedicated Google Docs capture path, an on-screen capture UI, and — the big one — **cryptographically verified author identity** (today's author name is self-asserted; real verification needs an external identity authority, which a local-only tool can't provide alone). Per-contributor attribution and a paste-source citation helper are likewise parked until the capture layer records who and where.

To go deeper:

- **Design spec** — the full architecture and threat model: [`docs/superpowers/specs/2026-06-05-human-authorship-attestation-design.md`](docs/superpowers/specs/2026-06-05-human-authorship-attestation-design.md), and the signals/reporting design: [`…2026-06-06-authorship-signals-and-reporting-design.md`](docs/superpowers/specs/2026-06-06-authorship-signals-and-reporting-design.md)
- **Changelog** — what's shipped, release by release: [`CHANGELOG.md`](CHANGELOG.md)
- **Research catalog** — every external source behind the decisions: [`docs/research/`](docs/research/)

## Questions people ask

**Why not just use an "AI detector"?** Because they don't work reliably — they wrongly flag human writing (especially from non-native speakers) often enough to ruin lives, and miss real AI text. Detecting AI *after the fact* is guesswork; vouching for *how a document was written* is evidence. We anchor on the second.

**Isn't this just a keylogger?** No. It records the *shape* of writing (timing and edit events), never your words, and nothing leaves your machine. The honesty here is structural: the captured summary literally cannot contain your text.

**Can't someone fake it?** Yes — see [the honest framing above](#read-this-first-it-attests-it-doesnt-prove). We treat that as a feature to disclose, not a flaw to hide.

**Why "humanshipd"?** It's a placeholder name while the project finds its feet. The thing it actually issues — the public-facing claim — is "Human Authored."

## License

[MIT](LICENSE)
