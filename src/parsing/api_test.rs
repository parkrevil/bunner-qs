use super::{parse, parse_with};
use crate::ParseOptions;
use crate::parsing::ParseError;
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
    fn returns_default_for_empty_input() {
        // Arrange
        let query = "";

        // Act
        let parsed = parse::<Credentials>(query).expect("parse should succeed");

        // Assert
        assert_eq!(parsed, Credentials::default());
    }

    #[test]
    fn populates_struct_fields_from_pairs() {
        // Arrange
        let query = "username=neo&password=matrix";

        // Act
        let parsed = parse::<Credentials>(query).expect("parse should succeed");

        // Assert
        let expected = Credentials {
            username: "neo".to_string(),
            password: "matrix".to_string(),
        };
        assert_eq!(parsed, expected);
    }

    #[test]
    fn produces_nested_map_for_json_value() {
        // Arrange
        let query = "user[name]=alice&user[role]=admin";

        // Act
        let parsed = parse::<Value>(query).expect("parse should succeed");

        // Assert
        let expected = json!({
            "user": {
                "name": "alice",
                "role": "admin"
            }
        });
        assert_eq!(parsed, expected);
    }
}

mod parse_with_api {
    use super::*;

    #[test]
    fn decodes_plus_signs_when_space_as_plus_enabled() {
        // Arrange
        let query = "message=hello+world";
        let options = ParseOptions {
            space_as_plus: true,
            ..ParseOptions::default()
        };

        // Act
        let parsed = parse_with::<Value>(query, &options).expect("parse_with should succeed");

        // Assert
        let expected = json!({ "message": "hello world" });
        assert_eq!(parsed, expected);
    }

    #[test]
    fn returns_error_when_parameter_limit_exceeded() {
        // Arrange
        let query = "a=1&b=2";
        let options = ParseOptions {
            max_params: Some(1),
            ..ParseOptions::default()
        };

        // Act
        let result = parse_with::<Value>(query, &options);

        // Assert
        assert!(matches!(
            result,
            Err(ParseError::TooManyParameters {
                limit: 1,
                actual: 2
            })
        ));
    }
}
