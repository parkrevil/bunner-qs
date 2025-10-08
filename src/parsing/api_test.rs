use super::{assume_json_value, parse};
use crate::ParseOptions;
use crate::parsing::errors::ParseError;
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
    fn should_return_default_for_empty_input() {
        let parsed: Credentials = parse_with_defaults("").expect("parse should succeed");

        assert_eq!(parsed, Credentials::default());
    }

    #[test]
    fn should_deserialize_struct_from_pairs() {
        let parsed: Credentials =
            parse_with_defaults("username=neo&password=matrix").expect("parse should succeed");

        assert_eq!(parsed.username, "neo");
        assert_eq!(parsed.password, "matrix");
    }

    #[test]
    fn should_deserialize_json_value_from_pairs() {
        let parsed: Value =
            parse_with_defaults("user[name]=alice&user[role]=admin").expect("parse should succeed");

        assert_eq!(
            parsed,
            json!({ "user": { "name": "alice", "role": "admin" } })
        );
    }

    #[test]
    fn should_apply_space_as_plus_option() {
        let options = ParseOptions::new().space_as_plus(true);

        let parsed: Value =
            parse_with_options("message=hello+world", &options).expect("parse should succeed");

        assert_eq!(parsed, json!({ "message": "hello world" }));
    }

    #[test]
    fn should_surface_parse_error_for_invalid_query() {
        let result: Result<Value, ParseError> = parse_with_defaults("user?name=neo");

        assert_matches!(
            result,
            Err(ParseError::UnexpectedQuestionMark { index }) if index == 4
        );
    }
}

mod assume_json_value {
    use super::*;

    #[test]
    fn should_transfer_json_value_without_copying() {
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
