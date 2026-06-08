//! Per-format text front-end for verification (Decision 4).
//!
//! Verification matches on *text*, but real documents arrive in containers. This
//! module pulls the plain text out of a file so the single ISCC matching engine in
//! [`crate::fingerprint`] can compare it — one front-end feeding one engine.
//!
//! `.txt` and `.docx` work everywhere: the OOXML path inflates with pure-Rust
//! `flate2`, so it compiles to WASM and the in-browser verifier handles Word files.
//! `.pdf` extraction pulls a heavier native dependency and is therefore native-only;
//! in the browser it returns an error pointing the user at the command-line tool.

use crate::error::CoreError;
use std::io::Read;

/// A document container we can pull text out of.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocFormat {
    /// Treat the bytes as UTF-8 text.
    Text,
    /// Office Open XML (`.docx`).
    Docx,
    /// Portable Document Format (`.pdf`).
    Pdf,
}

impl DocFormat {
    /// Guess the format from a file name's extension. Unknown ⇒ [`DocFormat::Text`]
    /// (the safe default — interpret the bytes as UTF-8).
    pub fn from_name(name: &str) -> Self {
        match name.rsplit('.').next().map(str::to_ascii_lowercase).as_deref() {
            Some("docx") => DocFormat::Docx,
            Some("pdf") => DocFormat::Pdf,
            _ => DocFormat::Text,
        }
    }
}

/// Extract plain text from `bytes` interpreted as `format`.
pub fn extract_text(format: DocFormat, bytes: &[u8]) -> Result<String, CoreError> {
    match format {
        DocFormat::Text => Ok(String::from_utf8_lossy(bytes).into_owned()),
        DocFormat::Docx => extract_docx(bytes),
        DocFormat::Pdf => extract_pdf(bytes),
    }
}

/// Detect the format from `name`, then extract — the one call the verify path uses.
pub fn extract_named(name: &str, bytes: &[u8]) -> Result<String, CoreError> {
    extract_text(DocFormat::from_name(name), bytes)
}

/// Minimal OOXML extraction: concatenate `<w:t>` run text from `word/document.xml`,
/// one newline per paragraph close. Deliberately crude — ISCC normalizes whitespace
/// away, so header/footer/tracked-change nuances barely move the Content-Code.
fn extract_docx(bytes: &[u8]) -> Result<String, CoreError> {
    let mut zip = zip::ZipArchive::new(std::io::Cursor::new(bytes))
        .map_err(|e| CoreError::Format(format!("docx: not a zip: {e}")))?;
    let mut xml = String::new();
    zip.by_name("word/document.xml")
        .map_err(|e| CoreError::Format(format!("docx: missing word/document.xml: {e}")))?
        .read_to_string(&mut xml)
        .map_err(|e| CoreError::Format(format!("docx: unreadable document.xml: {e}")))?;
    Ok(extract_wt_runs(&xml))
}

/// Pull the text out of `<w:t>…</w:t>` runs, emitting a newline at each `</w:p>`.
fn extract_wt_runs(xml: &str) -> String {
    let mut out = String::new();
    let mut i = 0;
    while i < xml.len() {
        let rest = &xml[i..];
        if rest.starts_with("</w:p>") {
            out.push('\n');
            i += "</w:p>".len();
        } else if rest.starts_with("<w:t") {
            let gt = rest.find('>').map(|p| i + p).unwrap_or(i);
            if xml.as_bytes()[gt - 1] == b'/' {
                i = gt + 1; // self-closing <w:t/>
            } else if let Some(close) = xml[gt + 1..].find("</w:t>") {
                out.push_str(&unescape(&xml[gt + 1..gt + 1 + close]));
                i = gt + 1 + close + "</w:t>".len();
            } else {
                i = gt + 1;
            }
        } else {
            i += rest.chars().next().map(char::len_utf8).unwrap_or(1);
        }
    }
    out
}

fn unescape(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

/// PDF text extraction (native builds): heavier dependency, host-only.
#[cfg(feature = "native")]
fn extract_pdf(bytes: &[u8]) -> Result<String, CoreError> {
    pdf_extract::extract_text_from_mem(bytes)
        .map_err(|e| CoreError::Format(format!("pdf: {e}")))
}

/// PDF extraction is unavailable in the browser (WASM) build — the dependency isn't
/// WASM-compatible. Verify a `.pdf` with the command-line tool instead.
#[cfg(not(feature = "native"))]
fn extract_pdf(_bytes: &[u8]) -> Result<String, CoreError> {
    Err(CoreError::Format(
        "PDF text extraction isn't available in the browser; verify a .pdf with the command-line tool"
            .to_string(),
    ))
}
