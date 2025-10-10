use super::*;
use crate::serde_adapter::{DeserializeError, DeserializeErrorKind};

mod parse_error_display {
    use super::*;

    #[test]
    fn should_render_duplicate_key_error_with_key_name_when_duplicate_key_error_occurs_then_include_key_in_message()
     {
        let error = ParseError::DuplicateRootKey {
            key: "color".into(),
        };

        let message = error.to_string();

        assert_eq!(message, "duplicate root key 'color' not allowed");
    }

    #[test]
    fn should_render_too_many_parameters_error_with_counts_when_parameter_limit_is_exceeded_then_display_limit_and_actual()
     {
        let error = ParseError::TooManyParameters {
            limit: 3,
            actual: 5,
        };

        let message = error.to_string();

        assert_eq!(message, "too many parameters: received 5, limit 3");
    }
}

mod from {
    use super::*;

    #[test]
    fn should_prefix_message_when_wrapping_deserialize_error_then_include_error_context() {
        let serde_error =
            DeserializeError::from_kind(DeserializeErrorKind::InvalidBool { value: "NO".into() });

        let error = ParseError::from(serde_error);

        assert_eq!(
            error.to_string(),
            "failed to deserialize parsed query into target type: invalid boolean literal `NO`",
        );
    }
}
