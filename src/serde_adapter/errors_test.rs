use super::*;

mod format_expected {
    use super::*;

    #[test]
    fn should_format_placeholder_when_no_fields_provided_then_return_none_literal() {
        // Arrange
        const EMPTY_FIELDS: &[&str; 0] = &[];

        // Act
        let formatted = format_expected(EMPTY_FIELDS);

        // Assert
        assert_eq!(formatted, "(none)");
    }

    #[test]
    fn should_join_fields_with_commas_when_multiple_fields_are_provided_then_format_expected_list()
    {
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
    fn should_wrap_custom_message_for_serialize_error_when_custom_message_is_provided_then_echo_message()
     {
        // Arrange
        let message = "serialization failed";

        // Act
        let error = SerializeError::custom(message);

        // Assert
        assert_eq!(error.to_string(), message);
    }

    #[test]
    fn should_render_top_level_error_with_type_when_top_level_variant_is_string_then_include_value_name()
     {
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
    fn should_wrap_custom_message_for_deserialize_error_when_custom_message_is_provided_then_echo_message()
     {
        // Arrange
        let message = "deserialization failed";

        // Act
        let error = DeserializeError::custom(message);

        // Assert
        assert_eq!(error.to_string(), message);
    }

    #[test]
    fn should_list_expected_fields_for_unknown_field_when_field_is_missing_then_include_expected_list()
     {
        // Arrange
        let error = DeserializeError::from_kind(DeserializeErrorKind::UnknownField {
            field: "mystery".into(),
            expected: "alpha, beta".into(),
        });

        // Act
        let rendered = error.to_string();

        // Assert
        assert_eq!(
            rendered,
            "unknown field `mystery`; expected one of: alpha, beta"
        );
    }

    #[test]
    fn should_append_path_information_when_path_segments_present_then_show_context() {
        // Arrange
        let error = DeserializeError::from_kind(DeserializeErrorKind::InvalidBool {
            value: "YES".into(),
        })
        .with_path(vec![PathSegment::Key("flag".into()), PathSegment::Index(2)]);

        // Act
        let rendered = error.to_string();

        // Assert
        assert_eq!(rendered, "invalid boolean literal `YES` at flag[2]");
    }
}

mod serde_query_error {
    use super::*;

    #[test]
    fn should_prefix_message_when_wrapping_serialize_error_then_include_original_detail() {
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
    fn should_prefix_message_when_wrapping_deserialize_error_then_include_original_detail() {
        // Arrange
        let inner = DeserializeError::from_kind(DeserializeErrorKind::InvalidBool {
            value: "YES".into(),
        });

        // Act
        let error = SerdeQueryError::from(inner);

        // Assert
        assert_eq!(
            error.to_string(),
            "failed to deserialize query map: invalid boolean literal `YES`"
        );
    }
}
