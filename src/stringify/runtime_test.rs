use super::{StringifyError, StringifyRuntime};
use crate::config::StringifyOptions;
use crate::model::{OrderedMap, QueryMap, Value};
use assert_matches::assert_matches;

fn stringify_map(map: QueryMap, options: StringifyOptions) -> Result<String, StringifyError> {
    super::stringify_query_map_with(&map, &options)
}

fn options(space_as_plus: bool) -> StringifyOptions {
    StringifyOptions { space_as_plus }
}

fn nested_profile_map() -> QueryMap {
    let profile: OrderedMap<String, Value> = OrderedMap::from_iter([
        ("name".into(), Value::from("Alice")),
        (
            "emails".into(),
            Value::Array(vec![
                Value::from("work@example.com"),
                Value::from("home@example.com"),
            ]),
        ),
    ]);

    QueryMap::from_iter([("profile", Value::Object(profile))])
}

mod stringify_query_map_with {
    use super::*;

    #[test]
    fn should_return_empty_string_when_map_is_empty_then_produce_empty_output() {
        let map = QueryMap::new();
        let options = StringifyOptions::default();

        let result = stringify_map(map, options).expect("empty map should stringify");

        assert!(result.is_empty());
    }

    #[test]
    fn should_percent_encode_spaces_when_plus_option_disabled_then_encode_spaces_as_percent_twenty()
    {
        let map = QueryMap::from_iter([("space key", Value::from("space value"))]);
        let options = options(false);

        let result = stringify_map(map, options).expect("stringify should succeed");

        assert_eq!(result, "space%20key=space%20value");
    }

    #[test]
    fn should_convert_spaces_to_plus_when_option_enabled_then_replace_spaces_with_plus() {
        let map = QueryMap::from_iter([("space key", Value::from("space value"))]);
        let options = options(true);

        let result = stringify_map(map, options).expect("stringify should succeed");

        assert_eq!(result, "space+key=space+value");
    }

    #[test]
    fn should_error_on_control_characters_when_value_contains_newline_then_return_invalid_value_error()
     {
        let map = QueryMap::from_iter([("note", Value::from("line1\nline2"))]);
        let options = StringifyOptions::default();

        let error = stringify_map(map, options).expect_err("control characters should fail");

        assert_matches!(
            error,
            StringifyError::InvalidValue { key, value }
                if key == "note" && value == "line1\nline2"
        );
    }

    #[test]
    fn should_stringify_nested_structure_when_iterating_in_order_then_produce_encoded_nested_keys()
    {
        let map = nested_profile_map();
        let options = StringifyOptions::default();

        let result = stringify_map(map, options).expect("stringify should succeed");

        assert_eq!(
            result,
            "profile%5Bname%5D=Alice&profile%5Bemails%5D%5B0%5D=work%40example.com&profile%5Bemails%5D%5B1%5D=home%40example.com",
        );
    }

    #[test]
    fn should_error_on_control_characters_when_nested_key_contains_newline_then_return_invalid_key_error()
     {
        let profile: OrderedMap<String, Value> = OrderedMap::from_iter([
            ("valid".into(), Value::from("ok")),
            ("bad\nkey".into(), Value::from("oops")),
        ]);
        let map = QueryMap::from_iter([("profile", Value::Object(profile))]);
        let options = StringifyOptions::default();

        let error = stringify_map(map, options).expect_err("invalid subkey should fail");

        assert_matches!(
            error,
            StringifyError::InvalidKey { key } if key == "profile[bad\nkey]"
        );
    }

    #[test]
    fn should_report_full_nested_path_when_nested_value_contains_control_character_then_return_invalid_value_error()
     {
        let profile: OrderedMap<String, Value> =
            OrderedMap::from_iter([("bio".into(), Value::from("line1\nline2"))]);
        let map = QueryMap::from_iter([("profile", Value::Object(profile))]);
        let options = StringifyOptions::default();

        let error = stringify_map(map, options).expect_err("nested control character should fail");

        assert_matches!(
            error,
            StringifyError::InvalidValue { key, value }
                if key == "profile[bio]" && value == "line1\nline2"
        );
    }

    #[test]
    fn should_error_on_control_characters_when_root_key_contains_control_then_return_invalid_key_error()
     {
        let map = QueryMap::from_iter([(String::from("bad\u{0007}key"), Value::from("data"))]);
        let options = StringifyOptions::default();

        let error = stringify_map(map, options).expect_err("invalid root key should fail");

        assert_matches!(
            error,
            StringifyError::InvalidKey { key } if key == "bad\u{0007}key"
        );
    }
}

mod stringify_runtime {
    use super::*;

    #[test]
    fn should_respect_space_option_when_runtime_is_created_then_store_space_as_plus_flag() {
        let options = options(true);

        let runtime = StringifyRuntime::new(&options);

        assert!(runtime.space_as_plus);
    }

    #[test]
    fn should_allow_empty_map_when_prepare_state_receives_empty_map_then_initialize_empty_stack() {
        let map = QueryMap::new();
        let options = StringifyOptions::default();

        let state = super::super::prepare_stringify_state(&map, &options)
            .expect("expected empty map to succeed");

        assert!(state.output.is_empty());
        assert!(state.stack.is_empty());
    }
}
