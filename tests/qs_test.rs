use bunner_qs_rs::{
    OptionsValidationError, ParseOptions, Qs, QsParseError, QsStringifyError, StringifyOptions,
};
use serde::Serialize;

mod with_parse {
    use super::*;

    #[test]
    fn given_invalid_options_when_with_parse_called_then_return_validation_error() {
        // Arrange
        let options = ParseOptions {
            max_params: Some(0),
            ..ParseOptions::default()
        };
        let qs = Qs::new();

        // Act
        let result = qs.with_parse(options);

        // Assert
        assert!(matches!(
            result,
            Err(OptionsValidationError::NonZeroRequired {
                field: "max_params"
            })
        ));
    }
}

mod with_stringify {
    use super::*;

    #[test]
    fn given_valid_stringify_options_when_with_stringify_called_then_store_options() {
        // Arrange
        let options = StringifyOptions::new().space_as_plus(true);
        let qs = Qs::new();

        // Act
        let configured = qs
            .with_stringify(options.clone())
            .expect("stringify options configuration should succeed");

        // Assert
        let stored = configured
            .stringify_options()
            .expect("stringify options should be stored");
        assert_eq!(stored.space_as_plus, options.space_as_plus);
    }
}

mod parse {
    use super::*;

    #[test]
    fn given_missing_parse_options_when_parse_called_then_return_missing_options_error() {
        // Arrange
        let qs = Qs::new();

        // Act
        let result: Result<serde_json::Value, QsParseError> = qs.parse("city=Seoul");

        // Assert
        assert!(matches!(result, Err(QsParseError::MissingParseOptions)));
    }

    #[test]
    fn given_parse_options_when_parse_called_then_return_deserialized_json_value() {
        // Arrange
        let qs = Qs::new()
            .with_parse(ParseOptions::default())
            .expect("parse options configuration should succeed");

        // Act
        let parsed: serde_json::Value = qs.parse("city=Seoul").expect("parsing should succeed");

        // Assert
        assert_eq!(parsed.get("city"), Some(&serde_json::Value::from("Seoul")));
    }

    #[test]
    fn given_invalid_query_when_parse_called_then_return_parse_error() {
        // Arrange
        let qs = Qs::new()
            .with_parse(ParseOptions::default())
            .expect("parse options configuration should succeed");

        // Act
        let result: Result<serde_json::Value, QsParseError> = qs.parse("broken=%E4%ZZ");

        // Assert
        assert!(matches!(result, Err(QsParseError::Parse(_))));
    }
}

mod stringify {
    use super::*;

    #[derive(Serialize)]
    struct Payload<'a> {
        city: &'a str,
    }

    #[test]
    fn given_missing_stringify_options_when_stringify_called_then_return_missing_options_error() {
        // Arrange
        let qs = Qs::new();
        let payload = Payload { city: "Seoul" };

        // Act
        let result = qs.stringify(&payload);

        // Assert
        assert!(matches!(
            result,
            Err(QsStringifyError::MissingStringifyOptions)
        ));
    }

    #[test]
    fn given_stringify_options_when_stringify_called_then_return_encoded_query() {
        // Arrange
        let qs = Qs::new()
            .with_stringify(StringifyOptions::default())
            .expect("stringify options configuration should succeed");
        let payload = Payload { city: "Seoul" };

        // Act
        let encoded = qs.stringify(&payload).expect("stringify should succeed");

        // Assert
        assert_eq!(encoded, "city=Seoul");
    }

    #[test]
    fn given_unsupported_value_when_stringify_called_then_return_stringify_error() {
        // Arrange
        let qs = Qs::new()
            .with_stringify(StringifyOptions::default())
            .expect("stringify options configuration should succeed");
        let value = "plain";

        // Act
        let result = qs.stringify(&value);

        // Assert
        assert!(matches!(result, Err(QsStringifyError::Stringify(_))));
    }
}

mod parse_options {
    use super::*;

    #[test]
    fn given_parse_options_configured_when_parse_options_called_then_return_reference() {
        // Arrange
        let options = ParseOptions::default();
        let qs = Qs::new()
            .with_parse(options.clone())
            .expect("parse options configuration should succeed");

        // Act
        let returned = qs.parse_options();

        // Assert
        let stored = returned.expect("parse options should be present");
        assert_eq!(stored.space_as_plus, options.space_as_plus);
        assert_eq!(stored.max_params, options.max_params);
        assert_eq!(stored.max_length, options.max_length);
        assert_eq!(stored.max_depth, options.max_depth);
    }
}
