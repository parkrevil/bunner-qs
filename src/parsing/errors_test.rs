use super::*;
use crate::serde_adapter::{DeserializeError, SerdeQueryError, SerializeError};

mod parse_error_display {
    use super::*;

    #[test]
    fn renders_duplicate_key_error_with_key_name() {
        // Arrange
        let error = ParseError::DuplicateKey {
            key: "color".into(),
        };

        // Act
        let message = error.to_string();

        // Assert
        assert_eq!(message, "duplicate key 'color' not allowed");
    }

    #[test]
    fn renders_too_many_parameters_with_counts() {
        // Arrange
        let error = ParseError::TooManyParameters {
            limit: 3,
            actual: 5,
        };

        // Act
        let message = error.to_string();

        // Assert
        assert_eq!(message, "too many parameters: received 5, limit 3");
    }
}

mod serde_conversion_behavior {
    use super::*;

    #[test]
    fn prefixes_message_when_wrapping_deserialize_error() {
        // Arrange
        let serde_error =
            SerdeQueryError::from(DeserializeError::InvalidBool { value: "NO".into() });

        // Act
        let error = ParseError::from(serde_error);

        // Assert
        assert_eq!(
            error.to_string(),
            "failed to deserialize parsed query into target type: failed to deserialize query map: invalid boolean literal `NO`",
        );
    }

    #[test]
    fn prefixes_message_when_wrapping_serialize_error() {
        // Arrange
        let serde_error = SerdeQueryError::from(SerializeError::Unsupported("tuple variant"));

        // Act
        let error = ParseError::from(serde_error);

        // Assert
        assert_eq!(
            error.to_string(),
            "failed to deserialize parsed query into target type: failed to serialize values into query map: unsupported serialization form: tuple variant",
        );
    }
}
