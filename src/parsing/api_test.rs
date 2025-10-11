use super::{assume_json_value, parse};
use crate::ParseOptions;
use crate::parsing::ParseError;
use crate::serde_adapter::DeserializeErrorKind;
use crate::parsing::errors::ParseLocation;
use assert_matches::assert_matches;
use serde::Deserialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

#[derive(Debug, Deserialize, PartialEq, Eq, Default)]
struct Credentials {
    username: String,
    password: String,
}

fn parse_with_options<T>(query: &str, options: &ParseOptions) -> Result<T, ParseError>
where
    T: DeserializeOwned + Default + 'static,
{
    parse(query, options)
}

fn parse_with_defaults<T>(query: &str) -> Result<T, ParseError>
where
    T: DeserializeOwned + Default + 'static,
{
    parse_with_options(query, &ParseOptions::default())
}

mod parse {
    use super::*;

    #[test]
    fn given_empty_input_when_parse_then_returns_default() {
        let parsed: Credentials = parse_with_defaults("").expect("parse should succeed");

        assert_eq!(parsed, Credentials::default());
    }

    #[test]
    fn given_key_value_pairs_when_parse_then_deserializes_struct() {
        let parsed: Credentials =
            parse_with_defaults("username=neo&password=matrix").expect("parse should succeed");

        assert_eq!(parsed.username, "neo");
        assert_eq!(parsed.password, "matrix");
    }

    #[test]
    fn given_separator_only_input_when_parse_then_returns_default() {
        let parsed: Credentials =
            parse_with_defaults("&&").expect("separator-only input should yield default");

        assert_eq!(parsed, Credentials::default());
    }

    #[test]
    fn given_nested_pairs_when_parse_then_returns_json_value() {
        let parsed: Value =
            parse_with_defaults("user[name]=alice&user[role]=admin").expect("parse should succeed");

        assert_eq!(
            parsed,
            json!({ "user": { "name": "alice", "role": "admin" } })
        );
    }

    #[test]
    fn given_space_as_plus_option_when_parse_then_normalizes_spaces() {
        let options = ParseOptions::new().space_as_plus(true);

        let parsed: Value =
            parse_with_options("message=hello+world", &options).expect("parse should succeed");

        assert_eq!(parsed, json!({ "message": "hello world" }));
    }

    #[test]
    fn given_invalid_query_when_parse_then_returns_parse_error() {
        let result: Result<Value, ParseError> = parse_with_defaults("user?name=neo");

        assert_matches!(
            result,
            Err(ParseError::UnexpectedQuestionMark { index, location })
                if index == 4 && location == ParseLocation::Query
        );
    }

    #[test]
    fn given_invalid_numeric_value_when_parse_then_propagates_deserialize_error() {
        #[allow(dead_code)]
        #[derive(Debug, Deserialize, Default)]
        struct NumericOnly {
            id: u32,
        }

        let result: Result<NumericOnly, ParseError> = parse_with_defaults("id=not-a-number");

        assert_matches!(
            result,
            Err(ParseError::Serde(error))
                if matches!(
                    error.kind(),
                    DeserializeErrorKind::InvalidNumber { value }
                        if value == "not-a-number"
                )
        );
    }

    #[test]
    fn given_param_limit_when_parse_then_returns_too_many_parameters_error() {
        let options = ParseOptions::new().max_params(1);

        let result: Result<Value, ParseError> =
            parse_with_options("a=1&b=2", &options);

        assert_matches!(
            result,
            Err(ParseError::TooManyParameters { limit, actual })
                if limit == 1 && actual == 2
        );
    }

    #[test]
    fn given_length_limit_when_parse_then_returns_input_too_long_error() {
        let options = ParseOptions::new().max_length(4);

        let result: Result<Value, ParseError> =
            parse_with_options("name=neo", &options);

        assert_matches!(
            result,
            Err(ParseError::InputTooLong { limit, actual })
                if limit == 4 && actual == 8
        );
    }
}

mod assume_json_value {
    use super::*;

    #[test]
    fn given_matching_type_when_assume_json_value_then_transfers_without_copy() {
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
}
