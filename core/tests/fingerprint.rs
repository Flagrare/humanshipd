use humanshipd_core::fingerprint::text_iscc;

const BODY: &str = "This is an original paragraph written for testing the content \
    fingerprint. It contains several sentences so the ISCC text code has enough \
    material to work with. Content codes are similarity preserving by design.";

#[test]
fn produces_a_non_empty_iscc_code() {
    let code = text_iscc(BODY).expect("iscc");
    assert!(!code.is_empty());
}

#[test]
fn identical_text_yields_an_identical_code() {
    assert_eq!(text_iscc(BODY).unwrap(), text_iscc(BODY).unwrap());
}

#[test]
fn different_text_yields_a_different_code() {
    let other = "A completely unrelated body of text about gardening and the weather.";
    assert_ne!(text_iscc(BODY).unwrap(), text_iscc(other).unwrap());
}
