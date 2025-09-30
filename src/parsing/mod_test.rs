use super::{ParseError, parse, parse_with};
use crate::config::ParseOptions;
use serde::Deserialize;
use serde_json::{Value, json};

#[derive(Debug, Deserialize, PartialEq, Eq, Default)]
struct Credentials {
    username: String,
    password: String,
}

mod parse_tests {
    use super::*;

    #[test]
    fn when_input_is_empty_it_returns_default_value() {
        // Arrange
        let query = "";

        // Act
        let parsed = parse::<Credentials>(query).expect("parse should succeed");

        // Assert
        assert_eq!(parsed, Credentials::default());
    }

    #[test]
    fn when_pairs_describe_struct_it_populates_fields() {
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
    fn when_target_is_json_value_it_returns_nested_map() {
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

mod parse_with_tests {
    use super::*;

    #[test]
    fn when_space_as_plus_is_enabled_it_decodes_plus_signs() {
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
    fn when_parameter_limit_is_exceeded_it_returns_error() {
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
