use super::*;

mod format_expected {
    use super::*;

    #[test]
    fn should_format_placeholder_when_no_fields_provided_then_return_none_literal() {
        const EMPTY_FIELDS: &[&str; 0] = &[];

        let formatted = format_expected(EMPTY_FIELDS);

        assert_eq!(formatted, "(none)");
    }

    #[test]
    fn should_join_fields_with_commas_when_multiple_fields_are_provided_then_format_expected_list()
    {
        const FIELDS: &[&str; 3] = &["alpha", "beta", "gamma"];

        let formatted = format_expected(FIELDS);

        assert_eq!(formatted, "alpha, beta, gamma");
    }
}

mod serialize_error {
    use super::*;
    use serde::ser::Error as _;

    #[test]
    fn should_wrap_custom_message_for_serialize_error_when_custom_message_is_provided_then_echo_message()
     {
        let message = "serialization failed";

        let error = SerializeError::custom(message);

        assert_eq!(error.to_string(), message);
    }

    #[test]
    fn should_render_top_level_error_with_type_when_top_level_variant_is_string_then_include_value_name()
     {
        let error = SerializeError::TopLevel("string".into());

        let rendered = error.to_string();

        assert_eq!(rendered, "top-level must serialize to a map, found string");
    }

    #[test]
    fn should_render_invalid_key_error_when_key_not_string_then_include_value_name() {
        let error = SerializeError::InvalidKey("integer".into());

        let rendered = error.to_string();

        assert_eq!(rendered, "map key must be a string, found integer");
    }

    #[test]
    fn should_render_unexpected_skip_message_when_placeholder_encountered_then_use_static_message()
    {
        let error = SerializeError::UnexpectedSkip;

        let rendered = error.to_string();

        assert_eq!(
            rendered,
            "unexpected placeholder value encountered during serialization"
        );
    }

    #[test]
    fn should_render_unsupported_variant_message_when_serializer_reports_unsupported_form_then_return_expected_message()
     {
        let error = SerializeError::Unsupported("enum variant");

        let rendered = error.to_string();

        assert_eq!(rendered, "unsupported serialization form: enum variant");
    }
}

mod deserialize_error {
    use super::*;
    use serde::de::Error as _;
    use std::error::Error as _;

    #[test]
    fn should_wrap_custom_message_for_deserialize_error_when_custom_message_is_provided_then_echo_message()
     {
        let message = "deserialization failed";

        let error = DeserializeError::custom(message);

        assert_eq!(error.to_string(), message);
    }

    #[test]
    fn should_list_expected_fields_for_unknown_field_when_field_is_missing_then_include_expected_list()
     {
        let error = DeserializeError::from_kind(DeserializeErrorKind::UnknownField {
            field: "mystery".into(),
            expected: "alpha, beta".into(),
        });

        let rendered = error.to_string();

        assert_eq!(
            rendered,
            "unknown field `mystery`; expected one of: alpha, beta"
        );
    }

    #[test]
    fn should_append_path_information_when_path_segments_present_then_show_context() {
        let error = DeserializeError::from_kind(DeserializeErrorKind::InvalidBool {
            value: "YES".into(),
        })
        .with_path(vec![PathSegment::Key("flag".into()), PathSegment::Index(2)]);

        let rendered = error.to_string();

        assert_eq!(rendered, "invalid boolean literal `YES` at flag[2]");
    }

    #[test]
    fn should_append_segments_when_using_push_segment_then_extend_error_path() {
        let error = DeserializeError::from_kind(DeserializeErrorKind::Message("oops".into()))
            .push_segment(PathSegment::Key("root".into()))
            .push_segment(PathSegment::Index(3));

        let rendered = error.to_string();

        assert_eq!(rendered, "oops at root[3]");
        assert_eq!(
            error.path(),
            &[PathSegment::Key("root".into()), PathSegment::Index(3)]
        );
    }

    #[test]
    fn should_extend_existing_path_when_more_segments_added_then_merge_segments() {
        let error = DeserializeError::from_kind(DeserializeErrorKind::Message("boom".into()))
            .with_path(vec![PathSegment::Key("user".into())])
            .push_segment(PathSegment::Index(2))
            .push_segment(PathSegment::Key("name".into()));

        assert_eq!(error.to_string(), "boom at user[2].name");
        assert_eq!(
            error.path(),
            &[
                PathSegment::Key("user".into()),
                PathSegment::Index(2),
                PathSegment::Key("name".into())
            ]
        );
        let source = error.source().expect("inner source should exist");
        assert_eq!(source.to_string(), "boom");
    }

    #[test]
    fn should_preserve_existing_path_when_with_path_called_twice_then_ignore_second_assignment() {
        let error = DeserializeError::from_kind(DeserializeErrorKind::InvalidNumber {
            value: "nope".into(),
        })
        .with_path(vec![PathSegment::Key("first".into())])
        .with_path(vec![PathSegment::Key("second".into())]);

        let rendered = error.to_string();

        assert_eq!(rendered, "invalid number literal `nope` at first");
        assert_eq!(error.path(), &[PathSegment::Key("first".into())]);
    }

    #[test]
    fn should_expose_kind_as_source_when_wrapped_then_source_matches_inner_kind() {
        let kind = DeserializeErrorKind::UnexpectedType {
            expected: "array",
            found: "string",
        };
        let error = DeserializeError::from_kind(kind.clone());

        let source = error.source().expect("source should exist");

        assert_eq!(source.to_string(), kind.to_string());
    }

    #[test]
    fn should_render_expected_object_error_when_scalar_provided_then_include_struct_name() {
        let error = DeserializeError::from_kind(DeserializeErrorKind::ExpectedObject {
            struct_name: "Account",
            found: "string",
        });

        let rendered = error.to_string();

        assert_eq!(
            rendered,
            "expected an object for struct `Account`, found string"
        );
    }
}

mod path_display {
    use super::*;

    #[test]
    fn should_format_mixed_path_segments_with_indices_then_render_formatted_path() {
        let segments = vec![
            PathSegment::Key("config".into()),
            PathSegment::Index(0),
            PathSegment::Key("enabled".into()),
        ];
        let error = DeserializeError::from_kind(DeserializeErrorKind::Message("invalid".into()))
            .with_path(segments);

        assert_eq!(error.to_string(), "invalid at config[0].enabled");
    }
}
