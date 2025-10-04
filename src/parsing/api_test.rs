use super::{assume_json_value, parse, parse_with};
use crate::ParseOptions;
use crate::parsing::ParseError;
use crate::serde_adapter::SerdeAdapterError;
use assert_matches::assert_matches;
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Debug, Deserialize, PartialEq, Eq, Default)]
struct Credentials {
    username: String,
    password: String,
}

mod parse_api {
    use super::*;

    #[test]
    fn should_return_default_when_input_is_empty_then_use_struct_default() {
        let query = "";

        let parsed = parse::<Credentials>(query).expect("parse should succeed");

        assert_eq!(parsed, Credentials::default());
    }

    #[test]
    fn should_populate_struct_fields_when_pairs_present_then_match_expected_credentials() {
        let query = "username=neo&password=matrix";

        let parsed = parse::<Credentials>(query).expect("parse should succeed");

        let expected = Credentials {
            username: "neo".to_string(),
            password: "matrix".to_string(),
        };
        assert_eq!(parsed, expected);
    }

    #[test]
    fn should_produce_nested_map_when_parsing_json_value_then_create_nested_object() {
        let query = "user[name]=alice&user[role]=admin";

        let parsed = parse::<Value>(query).expect("parse should succeed");

        let expected = json!({
            "user": {
                "name": "alice",
                "role": "admin"
            }
        });
        assert_eq!(parsed, expected);
    }

    #[test]
    fn should_return_default_when_trimmed_input_has_no_pairs_then_use_struct_default() {
        let query = "&&";

        let parsed = parse::<Credentials>(query).expect("parse should succeed");

        assert_eq!(parsed, Credentials::default());
    }

    #[test]
    fn should_error_when_query_contains_interior_question_mark_then_report_index() {
        let query = "user?name=neo";

        let result = parse::<Value>(query);

        assert_matches!(
            result,
            Err(ParseError::UnexpectedQuestionMark { index }) if index == 4
        );
    }

    #[test]
    fn should_return_json_null_when_pairs_absent_then_use_value_null_default() {
        let query = "&&";

        let parsed = parse::<Value>(query).expect("parse should succeed");

        assert_eq!(parsed, Value::Null);
    }

    #[test]
    fn should_report_question_mark_index_with_prefix_then_include_offset_in_error() {
        let query = "?user?name=neo";

        let result = parse::<Value>(query);

        assert_matches!(
            result,
            Err(ParseError::UnexpectedQuestionMark { index }) if index == 5
        );
    }

    #[test]
    fn should_return_duplicate_key_error_when_key_repeats_then_propagate_duplicate_key() {
        let query = "foo=one&foo=two";

        let result = parse::<Value>(query);

        assert_matches!(
            result,
            Err(ParseError::DuplicateKey { ref key }) if key == "foo"
        );
    }

    #[test]
    fn should_return_invalid_percent_error_when_value_encoding_is_invalid_then_report_index() {
        let query = "name=%GG";

        let result = parse::<Value>(query);

        assert_matches!(
            result,
            Err(ParseError::InvalidPercentEncoding { index }) if index == 5
        );
    }

    #[test]
    fn should_return_json_null_when_query_has_only_separators_then_use_value_default() {
        let query = "?&&";

        let parsed = parse::<Value>(query).expect("parse should succeed");

        assert_eq!(parsed, Value::Null);
    }

    #[test]
    fn should_wrap_deserialize_error_when_target_type_rejects_value_then_return_parse_error() {
        #[allow(dead_code)]
        #[derive(Debug, Deserialize, Default)]
        struct StrictFlag {
            flag: bool,
        }

        let query = "flag=not-bool";

        let result = parse::<StrictFlag>(query);

        assert_matches!(
            result,
            Err(ParseError::Serde(SerdeAdapterError::Deserialize(inner))) => {
                assert!(inner.to_string().contains("invalid boolean"));
            }
        );
    }
}

mod parse_with_api {
    use super::*;

    #[test]
    fn should_decode_plus_signs_when_space_option_is_enabled_then_convert_plus_to_space() {
        let query = "message=hello+world";
        let options = ParseOptions {
            space_as_plus: true,
            ..ParseOptions::default()
        };

        let parsed = parse_with::<Value>(query, &options).expect("parse_with should succeed");

        let expected = json!({ "message": "hello world" });
        assert_eq!(parsed, expected);
    }

    #[test]
    fn should_return_error_when_parameter_limit_is_exceeded_then_report_too_many_parameters() {
        let query = "a=1&b=2";
        let options = ParseOptions {
            max_params: Some(1),
            ..ParseOptions::default()
        };

        let result = parse_with::<Value>(query, &options);

        assert_matches!(
            result,
            Err(ParseError::TooManyParameters {
                limit: 1,
                actual: 2
            })
        );
    }

    #[test]
    fn should_return_default_when_map_is_empty_after_parsing_then_use_type_default() {
        let query = "&&";

        let parsed = parse_with::<Value>(query, &ParseOptions::default())
            .expect("parse_with should succeed");

        assert_eq!(parsed, Value::Null);
    }

    #[test]
    fn should_return_struct_default_when_parse_with_sees_empty_map_then_invoke_type_default() {
        let query = "&&";

        let parsed = parse_with::<Credentials>(query, &ParseOptions::default())
            .expect("parse_with should succeed");

        assert_eq!(parsed, Credentials::default());
    }

    #[test]
    fn should_wrap_deserialize_failures_when_target_type_rejects_value_then_return_serde_error() {
        #[allow(dead_code)]
        #[derive(Debug, Deserialize, Default)]
        struct StrictFlag {
            flag: bool,
        }

        let query = "flag=definitely_not_bool";

        let result = parse_with::<StrictFlag>(query, &ParseOptions::default());

        let err = result.expect_err("strict flag should fail to deserialize");
        assert_matches!(
            err,
            ParseError::Serde(SerdeAdapterError::Deserialize(inner)) => {
                assert!(inner.to_string().contains("invalid boolean literal"));
            }
        );
    }

    #[test]
    fn should_error_when_input_exceeds_max_length_then_propagate_limit() {
        let query = "message=too-long";
        let options = ParseOptions {
            max_length: Some(3),
            ..ParseOptions::default()
        };

        let result = parse_with::<Value>(query, &options);

        assert_matches!(
            result,
            Err(ParseError::InputTooLong { limit }) if limit == 3
        );
    }
}

mod json_specialization {
    use super::*;

    #[test]
    fn should_transfer_json_value_when_assume_json_value_used_then_preserve_structure() {
        let complex = json!({
            "profile": {
                "name": "trinity",
                "skills": ["matrix", "kung-fu"],
                "active": true
            }
        });

        let transferred: Value = unsafe { assume_json_value::<Value>(complex.clone()) };

        assert_eq!(transferred, complex);
    }

    #[test]
    fn should_preserve_nested_arrays_when_parsing_into_json_value_then_return_expected_structure() {
        let query = "items[0]=alpha&items[1]=beta&items[2][count]=3";

        let parsed = parse::<Value>(query).expect("parse should succeed");

        let expected = json!({
            "items": [
                "alpha",
                "beta",
                { "count": "3" }
            ]
        });

        assert_eq!(parsed, expected);
    }
}
