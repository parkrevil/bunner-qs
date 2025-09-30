use super::ensure_no_control;

#[test]
fn allows_clean_ascii_strings() {
    let result = ensure_no_control("user=alice&count=42");

    assert_eq!(result, Ok(()));
}

#[test]
fn allows_unicode_characters_above_control_range() {
    let result = ensure_no_control("café\u{00A0}preview");

    assert_eq!(result, Ok(()));
}

#[test]
fn rejects_ascii_control_characters() {
    assert!(ensure_no_control("line1\nline2").is_err());
    assert!(ensure_no_control("null\0byte").is_err());
}

#[test]
fn rejects_delete_character() {
    let input = format!("header:{}tail", char::from(0x7F));

    assert!(ensure_no_control(&input).is_err());
}
