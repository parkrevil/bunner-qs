use bunner_qs::{ParseError, Value, parse};

fn assert_array(value: &Value, expected: &[&str]) {
    match value {
        Value::Array(arr) => {
            assert_eq!(arr.len(), expected.len());
            for (idx, expected_value) in expected.iter().enumerate() {
                assert_eq!(arr[idx].as_str().unwrap(), *expected_value);
            }
        }
        other => panic!("expected array value, got {other:?}"),
    }
}

#[test]
fn allows_uniform_append_pattern() {
    let parsed = parse("key[]=1&key[]=2").expect("uniform append pattern should parse");
    let key_value = parsed.get("key").expect("key should exist");
    assert_array(key_value, &["1", "2"]);
}

#[test]
fn allows_uniform_numeric_pattern() {
    let parsed =
        parse("items[0]=apple&items[1]=banana").expect("uniform numeric pattern should parse");
    let items_value = parsed.get("items").expect("items should exist");
    assert_array(items_value, &["apple", "banana"]);
}

#[test]
fn rejects_mixed_append_and_numeric() {
    let error = parse("key[]=1&key[0]=1").expect_err("mixed append/numeric should fail");
    match error {
        ParseError::DuplicateKey { key } => assert_eq!(key, "key"),
        other => panic!("unexpected error {other:?}"),
    }
}

#[test]
fn rejects_scalar_and_nested_mix() {
    let error = parse("foo=1&foo[bar]=2").expect_err("mixing scalar and nested should fail");
    match error {
        ParseError::DuplicateKey { key } => assert_eq!(key, "foo"),
        other => panic!("unexpected error {other:?}"),
    }
}

#[test]
fn rejects_duplicate_scalar_values() {
    let error = parse("foo=1&foo=2").expect_err("duplicate scalar keys should fail");
    match error {
        ParseError::DuplicateKey { key } => assert_eq!(key, "foo"),
        other => panic!("unexpected error {other:?}"),
    }
}

#[test]
fn rejects_non_contiguous_numeric_indexes() {
    let error = parse("items[0]=apple&items[2]=cherry")
        .expect_err("non contiguous numeric indexes should fail");
    match error {
        ParseError::DuplicateKey { key } => assert_eq!(key, "items"),
        other => panic!("unexpected error {other:?}"),
    }
}
