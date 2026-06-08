use humanshipd_core::fingerprint::{classify, iscc_distance, text_iscc, Band};
use humanshipd_core::formats::{extract_named, extract_text, DocFormat};

mod common;
use common::minimal_docx;

const ESSAY: &str = "Provenance beats inference. We record the process by which text \
    was written and bind a verifiable credential to the result, rather than guessing \
    whether a machine produced it.";

#[test]
fn detects_format_from_the_file_extension() {
    assert_eq!(DocFormat::from_name("essay.docx"), DocFormat::Docx);
    assert_eq!(DocFormat::from_name("essay.PDF"), DocFormat::Pdf);
    assert_eq!(DocFormat::from_name("essay.txt"), DocFormat::Text);
    assert_eq!(DocFormat::from_name("no-extension"), DocFormat::Text);
}

#[test]
fn text_extraction_is_a_utf8_passthrough() {
    let out = extract_text(DocFormat::Text, ESSAY.as_bytes()).unwrap();
    assert_eq!(out, ESSAY);
}

#[test]
fn docx_extraction_recovers_the_same_content_code() {
    // The same writing wrapped in OOXML must extract to text whose Content-Code
    // matches the plain text — that's what lets a .docx verify cross-format.
    let docx = minimal_docx(ESSAY);
    let extracted = extract_named("essay.docx", &docx).unwrap();

    let d = iscc_distance(&text_iscc(&extracted).unwrap(), &text_iscc(ESSAY).unwrap())
        .expect("comparable codes");
    assert_eq!(classify(d), Band::SameContent, "docx distance was {d}");
}

#[test]
fn unknown_extension_falls_back_to_text() {
    let out = extract_named("mystery", ESSAY.as_bytes()).unwrap();
    assert_eq!(out, ESSAY);
}

#[cfg(not(feature = "native"))]
#[test]
fn pdf_extraction_is_unavailable_without_native() {
    assert!(extract_text(DocFormat::Pdf, b"%PDF-1.4").is_err());
}
