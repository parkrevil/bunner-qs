use super::parse_key_path;

#[test]
fn parses_plain_key_without_brackets() {
    let segments = parse_key_path("profile");
    assert_eq!(segments.as_slice(), ["profile"]);
}

#[test]
fn parses_nested_bracket_segments() {
    let segments = parse_key_path("user[0][name]");
    assert_eq!(segments.as_slice(), ["user", "0", "name"]);
}

#[test]
fn parses_trailing_segment_after_brackets() {
    let segments = parse_key_path("items[42]status");
    assert_eq!(segments.as_slice(), ["items", "42", "status"]);
}

#[test]
fn parses_empty_bracket_as_empty_segment() {
    let segments = parse_key_path("flag[]");
    assert_eq!(segments.as_slice(), ["flag", ""]);
}

#[test]
fn handles_unmatched_open_bracket() {
    let segments = parse_key_path("foo[bar");
    assert_eq!(segments.as_slice(), ["foo", "bar"]);
}

#[test]
fn returns_empty_for_empty_input() {
    let segments = parse_key_path("");
    assert!(segments.is_empty());
}
