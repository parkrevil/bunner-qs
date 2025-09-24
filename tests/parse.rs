use bunner_qs::{ParseError, ParseOptions, QueryMap, parse, parse_with_options};
fn map(entries: &[(&str, &[&str])]) -> QueryMap {
    let mut result = QueryMap::new();
    for (key, values) in entries {
        result.insert(
            (*key).to_string(),
            values.iter().map(|v| (*v).to_string()).collect(),
        );
    }
    result
}

#[test]
fn parses_basic_pairs() {
    let parsed = parse("a=1&b=two").expect("should parse basic pairs");
    let expected = map(&[("a", &["1"]), ("b", &["two"])]);
    assert_eq!(parsed, expected);
}

#[test]
fn decodes_percent_sequences() {
    let parsed = parse("name=John%20Doe").expect("should decode percent sequences");
    assert_eq!(parsed.get("name"), Some(&vec!["John Doe".to_string()]));
}

#[test]
fn rejects_invalid_percent_sequences() {
    let error = parse("a=%2").expect_err("invalid percent encoding should fail");
    assert!(matches!(error, ParseError::InvalidPercentEncoding { .. }));
}

#[test]
fn rejects_unmatched_brackets() {
    let error = parse("a[1=0").expect_err("unmatched bracket should fail");
    assert!(matches!(error, ParseError::UnmatchedBracket { .. }));
}

#[test]
fn plus_handling_respects_option() {
    let options = ParseOptions {
        space_as_plus: true,
        ..ParseOptions::default()
    };
    let parsed = parse_with_options("a=one+two", &options).expect("plus should become space");
    assert_eq!(parsed.get("a"), Some(&vec!["one two".to_string()]));

    let strict = parse("a=one+two").expect("default keeps plus literal");
    assert_eq!(strict.get("a"), Some(&vec!["one+two".to_string()]));
}

#[test]
fn duplicate_keys_can_be_restricted() {
    let options = ParseOptions {
        allow_duplicates: false,
        ..ParseOptions::default()
    };
    let error = parse_with_options("a=1&a=2", &options).expect_err("duplicate keys should fail");
    assert!(matches!(error, ParseError::DuplicateKey { .. }));
}

#[test]
fn enforces_max_params() {
    let options = ParseOptions {
        max_params: Some(1),
        ..ParseOptions::default()
    };
    let error =
        parse_with_options("a=1&b=2", &options).expect_err("parameter limit should trigger");
    assert!(matches!(error, ParseError::TooManyParameters { .. }));
}

#[test]
fn enforces_max_depth() {
    let options = ParseOptions {
        max_depth: Some(1),
        ..ParseOptions::default()
    };
    let error = parse_with_options("a[b][c]=1", &options).expect_err("depth limit should trigger");
    assert!(matches!(error, ParseError::DepthExceeded { .. }));
}

#[test]
fn allows_empty_input() {
    let parsed = parse("").expect("empty input should succeed");
    assert!(parsed.is_empty());
}
