//! Validation spike for the Decision-4 *verify path on a real file*: extract the
//! text from a real Google-exported `.docx`, and confirm its ISCC Content-Code
//! matches the ISCC of Google's own `.txt` export of the same document — i.e. a
//! reader who drops in their `.docx` verifies against the same writing.
//!
//! Run: cargo run --example docx_extract -- <doc.docx> <doc.txt>

use humanshipd_core::fingerprint::text_iscc;
use std::io::Read;

fn main() {
    let mut args = std::env::args().skip(1);
    let (docx_path, txt_path) = (args.next().expect("docx path"), args.next().expect("txt path"));

    let docx_text = extract_docx_text(&docx_path);
    let txt = std::fs::read_to_string(&txt_path).expect("read txt");

    println!("docx-extracted ({} chars): {:?}", docx_text.chars().count(), preview(&docx_text));
    println!("txt-export     ({} chars): {:?}", txt.chars().count(), preview(&txt));

    let iscc_docx = text_iscc(&docx_text).expect("iscc docx");
    let iscc_txt = text_iscc(&txt).expect("iscc txt");
    let different = text_iscc("An unrelated paragraph about coral reefs and warm shallow seas.").expect("iscc");

    println!("\nISCC(docx) = {iscc_docx}");
    println!("ISCC(txt)  = {iscc_txt}");
    println!("hamming(docx, txt)       = {}/64   <- same writing, two real formats", hamming(&digest(&iscc_docx), &digest(&iscc_txt)));
    println!("hamming(docx, different) = {}/64", hamming(&digest(&iscc_docx), &digest(&different)));
}

/// Minimal OOXML text extraction: concatenate `<w:t>` run text, newline per `</w:p>`.
fn extract_docx_text(path: &str) -> String {
    let file = std::fs::File::open(path).expect("open docx");
    let mut zip = zip::ZipArchive::new(file).expect("read zip");
    let mut xml = String::new();
    zip.by_name("word/document.xml").expect("document.xml").read_to_string(&mut xml).expect("read xml");

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
            i += rest.chars().next().map(|c| c.len_utf8()).unwrap_or(1);
        }
    }
    out
}

fn unescape(s: &str) -> String {
    s.replace("&amp;", "&").replace("&lt;", "<").replace("&gt;", ">").replace("&quot;", "\"").replace("&apos;", "'")
}

fn preview(s: &str) -> String {
    s.trim_start_matches('\u{feff}').chars().take(70).collect()
}

fn digest(iscc: &str) -> Vec<u8> {
    let body = iscc.strip_prefix("ISCC:").unwrap_or(iscc);
    let bytes = b32_decode(body);
    bytes[bytes.len().saturating_sub(8)..].to_vec()
}

fn b32_decode(s: &str) -> Vec<u8> {
    const ALPHA: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    let (mut bits, mut nbits, mut out) = (0u32, 0u32, Vec::new());
    for c in s.bytes() {
        let Some(v) = ALPHA.iter().position(|&x| x == c) else { continue };
        bits = (bits << 5) | v as u32;
        nbits += 5;
        if nbits >= 8 {
            nbits -= 8;
            out.push((bits >> nbits) as u8);
        }
    }
    out
}

fn hamming(a: &[u8], b: &[u8]) -> u32 {
    a.iter().zip(b).map(|(x, y)| (x ^ y).count_ones()).sum()
}
