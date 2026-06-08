use humanshipd_core::fingerprint::{
    classify, iscc_distance, text_iscc, Band, BAND_BORDERLINE_MAX, BAND_SAME_CONTENT_MAX,
    BAND_SAME_WRITING_MAX,
};

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

#[test]
fn identical_text_has_zero_distance() {
    let code = text_iscc(BODY).unwrap();
    assert_eq!(iscc_distance(&code, &code), Some(0.0));
}

#[test]
fn reformatting_stays_within_the_same_content_band() {
    // Same words, different whitespace: collapsed spaces, hard line wraps, a
    // trailing space before punctuation — the kind of noise format conversion adds.
    // ISCC normalizes whitespace, so this must land in the tightest band.
    let reformatted = BODY.replace(' ', "  ").replace(". ", " .\n");
    let a = text_iscc(BODY).unwrap();
    let b = text_iscc(&reformatted).unwrap();
    let d = iscc_distance(&a, &b).expect("comparable 256-bit codes");
    assert!(
        d <= BAND_SAME_CONTENT_MAX,
        "reformatting distance {d} should be within same-content band {BAND_SAME_CONTENT_MAX}"
    );
    assert_eq!(classify(d), Band::SameContent);
}

#[test]
fn unrelated_text_exceeds_the_no_match_threshold() {
    let a = text_iscc(BODY).unwrap();
    let b = text_iscc(
        "Tide charts and coral spawning cycles govern when the reef releases its \
         gametes; marine biologists track lunar phase, water temperature, and salinity \
         to predict the annual mass-spawning event across the atoll.",
    )
    .unwrap();
    let d = iscc_distance(&a, &b).expect("comparable 256-bit codes");
    assert!(
        d > BAND_BORDERLINE_MAX,
        "unrelated distance {d} should exceed borderline ceiling {BAND_BORDERLINE_MAX}"
    );
    assert_eq!(classify(d), Band::NoMatch);
}

#[test]
fn classify_maps_distances_to_the_locked_bands() {
    assert_eq!(classify(0.0), Band::SameContent);
    assert_eq!(classify(BAND_SAME_CONTENT_MAX), Band::SameContent);
    assert_eq!(classify(BAND_SAME_CONTENT_MAX + 0.01), Band::SameWriting);
    assert_eq!(classify(BAND_SAME_WRITING_MAX), Band::SameWriting);
    assert_eq!(classify(BAND_SAME_WRITING_MAX + 0.01), Band::Borderline);
    assert_eq!(classify(BAND_BORDERLINE_MAX), Band::Borderline);
    assert_eq!(classify(BAND_BORDERLINE_MAX + 0.01), Band::NoMatch);
    assert_eq!(classify(1.0), Band::NoMatch);
}

#[test]
fn codes_of_different_bit_length_are_incomparable() {
    // A 64-bit code (8-byte digest) can't be Hamming-compared to a 256-bit one.
    let small = iscc_lib::gen_text_code_v0(BODY, 64).unwrap().iscc;
    let big = text_iscc(BODY).unwrap();
    assert_eq!(iscc_distance(&small, &big), None);
}
