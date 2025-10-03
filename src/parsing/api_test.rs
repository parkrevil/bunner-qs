use super::{parse, parse_with};
use crate::ParseOptions;
use crate::parsing::ParseError;
use crate::serde_adapter::SerdeQueryError;
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

        assert!(matches!(
            result,
            Err(ParseError::TooManyParameters {
                limit: 1,
                actual: 2
            })
        ));
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

        match result {
            Err(ParseError::Serde(SerdeQueryError::Deserialize(err))) => {
                assert!(err.to_string().contains("invalid boolean literal"));
            }
            other => panic!("expected serde deserialize error, but received: {other:?}"),
        }
    }
}
