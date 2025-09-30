use super::*;

mod format_expected {
    use super::*;

    #[test]
    fn formats_placeholder_when_no_fields_provided() {
        // Arrange
        const EMPTY_FIELDS: &[&str; 0] = &[];

        // Act
        let formatted = format_expected(EMPTY_FIELDS);

        // Assert
        assert_eq!(formatted, "(none)");
    }

    #[test]
    fn joins_fields_with_commas() {
        // Arrange
        const FIELDS: &[&str; 3] = &["alpha", "beta", "gamma"];

        // Act
        let formatted = format_expected(FIELDS);

        // Assert
        assert_eq!(formatted, "alpha, beta, gamma");
    }
}

mod serialize_error {
    use super::*;
    use serde::ser::Error as _;

    #[test]
    fn wraps_custom_message_for_serialize_error() {
        // Arrange
        let message = "serialization failed";

        // Act
        let error = SerializeError::custom(message);

        // Assert
        assert_eq!(error.to_string(), message);
    }

    #[test]
    fn renders_top_level_error_with_type() {
        // Arrange
        let error = SerializeError::TopLevel("string".into());

        // Act
        let rendered = error.to_string();

        // Assert
        assert_eq!(rendered, "top-level must serialize to a map, found string");
    }
}

mod deserialize_error {
    use super::*;
    use serde::de::Error as _;

    #[test]
    fn wraps_custom_message_for_deserialize_error() {
        // Arrange
        let message = "deserialization failed";

        // Act
        let error = DeserializeError::custom(message);

        // Assert
        assert_eq!(error.to_string(), message);
    }

    #[test]
    fn lists_expected_fields_for_unknown_field() {
        // Arrange
        let error = DeserializeError::UnknownField {
            field: "mystery".into(),
            expected: "alpha, beta".into(),
        };

        // Act
        let rendered = error.to_string();

        // Assert
        assert_eq!(
            rendered,
            "unknown field `mystery`; expected one of: alpha, beta"
        );
    }
}

mod serde_query_error {
    use super::*;

    #[test]
    fn prefixes_message_when_wrapping_serialize_error() {
        // Arrange
        let inner = SerializeError::Unsupported("tuple variant");

        // Act
        let error = SerdeQueryError::from(inner);

        // Assert
        assert_eq!(
            error.to_string(),
            "failed to serialize values into query map: unsupported serialization form: tuple variant"
        );
    }

    #[test]
    fn prefixes_message_when_wrapping_deserialize_error() {
        // Arrange
        let inner = DeserializeError::InvalidBool {
            value: "YES".into(),
        };

        // Act
        let error = SerdeQueryError::from(inner);

        // Assert
        assert_eq!(
            error.to_string(),
            "failed to deserialize query map: invalid boolean literal `YES`"
        );
    }
}
