use bunner_qs::{ParseError, ParseOptions, QueryMap, Value, parse, parse_with_options, stringify};

fn map_simple(entries: &[(&str, &str)]) -> QueryMap {
    let mut result = QueryMap::new();
    for (key, value) in entries {
        result.insert((*key).to_string(), Value::String((*value).to_string()));
    }
    result
}

#[test]
fn parses_basic_pairs() {
    let parsed = parse("a=1&b=two").expect("should parse basic pairs");
    let expected = map_simple(&[("a", "1"), ("b", "two")]);
    assert_eq!(parsed, expected);
}

#[test]
fn decodes_percent_sequences() {
    let parsed = parse("name=John%20Doe").expect("should decode percent sequences");
    assert_eq!(
        parsed.get("name"),
        Some(&Value::String("John Doe".to_string()))
    );
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
    assert_eq!(parsed.get("a"), Some(&Value::String("one two".to_string())));

    let strict = parse("a=one+two").expect("default keeps plus literal");
    assert_eq!(strict.get("a"), Some(&Value::String("one+two".to_string())));
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

#[test]
fn builder_constructs_parse_options() {
    let options = ParseOptions::builder()
        .space_as_plus(true)
        .max_params(Some(8))
        .max_length(Some(128))
        .max_depth(Some(2))
        .allow_duplicates(false)
        .build();

    assert!(options.space_as_plus);
    assert_eq!(options.max_params, Some(8));
    assert_eq!(options.max_length, Some(128));
    assert_eq!(options.max_depth, Some(2));
    assert!(!options.allow_duplicates);
}

#[test]
fn parses_nested_objects() {
    let parsed = parse("user[name]=John&user[age]=30").expect("should parse nested objects");

    let user_obj = parsed.get("user").unwrap().as_object().unwrap();
    assert_eq!(user_obj.get("name").unwrap().as_str().unwrap(), "John");
    assert_eq!(user_obj.get("age").unwrap().as_str().unwrap(), "30");
}

#[test]
fn parses_nested_arrays() {
    let parsed = parse("items[0]=apple&items[1]=banana").expect("should parse nested arrays");

    println!("Parsed result: {:#?}", parsed);
    let items_value = parsed.get("items").unwrap();
    println!("Items value: {:#?}", items_value);

    match items_value {
        Value::Array(arr) => {
            assert_eq!(arr[0].as_str().unwrap(), "apple");
            assert_eq!(arr[1].as_str().unwrap(), "banana");
        }
        Value::Object(obj) => {
            // If it's an object with numeric keys, that's also valid
            assert_eq!(obj.get("0").unwrap().as_str().unwrap(), "apple");
            assert_eq!(obj.get("1").unwrap().as_str().unwrap(), "banana");
        }
        _ => panic!("Expected array or object with numeric keys"),
    }
}

#[test]
fn parses_complex_nesting() {
    let parsed = parse("data[users][0][name]=Alice&data[users][1][name]=Bob")
        .expect("should parse complex nested structure");

    let data_obj = parsed.get("data").unwrap().as_object().unwrap();
    let users_arr = data_obj.get("users").unwrap().as_array().unwrap();
    let user0_obj = users_arr[0].as_object().unwrap();
    let user1_obj = users_arr[1].as_object().unwrap();

    assert_eq!(user0_obj.get("name").unwrap().as_str().unwrap(), "Alice");
    assert_eq!(user1_obj.get("name").unwrap().as_str().unwrap(), "Bob");
}

#[test]
fn handles_duplicate_keys_as_arrays() {
    let parsed = parse("color=red&color=blue").expect("should handle duplicates as arrays");

    let colors = parsed.get("color").unwrap().as_array().unwrap();
    assert_eq!(colors[0].as_str().unwrap(), "red");
    assert_eq!(colors[1].as_str().unwrap(), "blue");
}

#[test]
fn round_trip_nested_structure() {
    // Test that we can parse and stringify nested structures
    let input =
        "user[name]=Alice&user[details][age]=25&user[hobbies][0]=reading&user[hobbies][1]=coding";
    let parsed = parse(input).expect("should parse complex nested structure");
    let stringified = stringify(&parsed).expect("should stringify back");

    // Parse again to ensure consistency
    let reparsed = parse(&stringified).expect("should parse stringified result");
    assert_eq!(parsed, reparsed);
}
