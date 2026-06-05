use humanshipd_core::text_embed::{embed, extract, strip};

#[test]
fn manifest_round_trips_through_text() {
    let text = "This is a human-written paragraph.";
    let manifest: Vec<u8> = (0u8..=255).cycle().take(500).collect();

    let carrying = embed(text, &manifest);
    assert_eq!(extract(&carrying).expect("payload"), manifest);
}

#[test]
fn embedded_payload_is_invisible_to_the_visible_text() {
    let text = "Visible prose stays exactly the same.";
    let carrying = embed(text, b"hidden credential bytes");
    assert_eq!(strip(&carrying), text, "stripping must restore the original text");
    assert!(carrying.starts_with(text), "visible prefix is the original text");
}

#[test]
fn all_byte_values_round_trip() {
    let manifest: Vec<u8> = (0u8..=255).collect();
    let carrying = embed("x", &manifest);
    assert_eq!(extract(&carrying).unwrap(), manifest);
}

#[test]
fn plain_text_without_a_payload_extracts_nothing() {
    assert_eq!(extract("just ordinary text, no credential"), None);
}
