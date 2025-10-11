use super::*;
use crate::serde_adapter::{DeserializeError, DeserializeErrorKind};

mod to_string {
    use super::*;

    #[test]
    fn should_format_input_too_long_when_to_string_called_then_include_limit_and_actual() {
        let error = ParseError::InputTooLong {
            limit: 5,
            actual: 8,
        };
        let message = error.to_string();

        assert_eq!(
            message,
            "input exceeds maximum length of 5 characters (received 8)"
        );
    }

    #[test]
    fn should_format_too_many_parameters_when_to_string_called_then_include_limit_and_actual() {
        let error = ParseError::TooManyParameters {
            limit: 2,
            actual: 3,
        };
        let message = error.to_string();

        assert_eq!(message, "too many parameters: received 3, limit 2");
    }

    #[test]
    fn should_format_duplicate_root_key_when_to_string_called_then_include_key_name() {
        let error = ParseError::DuplicateRootKey {
            key: "color".into(),
        };
        let message = error.to_string();

        assert_eq!(message, "duplicate root key 'color' not allowed");
    }

    #[test]
    fn should_format_duplicate_map_entry_when_to_string_called_then_include_parent_and_segment() {
        let error = ParseError::DuplicateMapEntry {
            parent: "user".into(),
            segment: "name".into(),
        };
        let message = error.to_string();

        assert_eq!(
            message,
            "duplicate map entry 'name' under 'user' not allowed"
        );
    }

    #[test]
    fn should_format_duplicate_sequence_index_when_to_string_called_then_include_index() {
        let error = ParseError::DuplicateSequenceIndex {
            parent: "items".into(),
            index: 2,
        };
        let message = error.to_string();

        assert_eq!(
            message,
            "duplicate sequence index 2 under 'items' not allowed"
        );
    }

    #[test]
    fn should_format_key_pattern_conflict_when_to_string_called_then_include_parent_and_segment() {
        let error = ParseError::KeyPatternConflict {
            parent: "key[path]".into(),
            segment: "field".into(),
        };
        let message = error.to_string();

        assert_eq!(
            message,
            "incompatible key pattern for segment 'field' under 'key[path]'"
        );
    }

    #[test]
    fn should_format_invalid_percent_encoding_when_to_string_called_then_include_location_and_index()
     {
        let error = ParseError::InvalidPercentEncoding {
            index: 7,
            location: ParseLocation::Value,
        };
        let message = error.to_string();

        assert_eq!(
            message,
            "invalid percent-encoding in value at byte offset 7"
        );
    }

    #[test]
    fn should_format_unexpected_question_mark_when_to_string_called_then_include_index() {
        let error = ParseError::UnexpectedQuestionMark {
            index: 4,
            location: ParseLocation::Key,
        };
        let message = error.to_string();

        assert_eq!(message, "unexpected '?' character in key at byte offset 4");
    }

    #[test]
    fn should_format_invalid_character_when_to_string_called_then_include_character_and_location() {
        let error = ParseError::InvalidCharacter {
            character: '\u{0007}',
            index: 3,
            location: ParseLocation::Query,
        };
        let message = error.to_string();

        assert_eq!(
            message,
            "invalid character `\u{7}` in query at byte offset 3"
        );
    }

    #[test]
    fn should_format_unmatched_bracket_when_to_string_called_then_include_bracket_and_key() {
        let error = ParseError::UnmatchedBracket {
            key: "items[".into(),
            bracket: '[',
        };
        let message = error.to_string();

        assert_eq!(message, "unmatched '[' bracket sequence in key 'items['");
    }

    #[test]
    fn should_format_depth_exceeded_when_to_string_called_then_include_limit_and_depth() {
        let error = ParseError::DepthExceeded {
            key: "items[0][value]".into(),
            limit: 1,
            depth: 2,
        };
        let message = error.to_string();

        assert_eq!(
            message,
            "maximum bracket depth exceeded in key 'items[0][value]' (depth 2, limit 1)"
        );
    }

    #[test]
    fn should_format_invalid_utf8_when_to_string_called_then_include_location() {
        let error = ParseError::InvalidUtf8 {
            location: ParseLocation::Parameter,
        };
        let message = error.to_string();

        assert_eq!(message, "decoded component in parameter is not valid UTF-8");
    }
}

mod as_str {
    use super::*;

    #[test]
    fn should_return_query_label_when_as_str_called_then_yield_query_literal() {
        let location = ParseLocation::Query;
        let label = location.as_str();

        assert_eq!(label, "query");
    }

    #[test]
    fn should_return_key_label_when_as_str_called_then_yield_key_literal() {
        let location = ParseLocation::Key;
        let label = location.as_str();

        assert_eq!(label, "key");
    }

    #[test]
    fn should_return_value_label_when_as_str_called_then_yield_value_literal() {
        let location = ParseLocation::Value;
        let label = location.as_str();

        assert_eq!(label, "value");
    }

    #[test]
    fn should_return_parameter_label_when_as_str_called_then_yield_parameter_literal() {
        let location = ParseLocation::Parameter;
        let label = location.as_str();

        assert_eq!(label, "parameter");
    }
}

mod from {
    use super::*;

    #[test]
    fn should_prefix_invalid_bool_message_when_from_called_then_include_context_prefix() {
        let serde_error =
            DeserializeError::from_kind(DeserializeErrorKind::InvalidBool { value: "NO".into() });
        let error = ParseError::from(serde_error);

        assert_eq!(
            error.to_string(),
            "failed to deserialize parsed query into target type: invalid boolean literal `NO`",
        );
    }

    #[test]
    fn should_prefix_unknown_field_message_when_from_called_then_include_field_name() {
        let serde_error = DeserializeError::from_kind(DeserializeErrorKind::UnknownField {
            field: "mode".into(),
            expected: "(none)".into(),
        });
        let error = ParseError::from(serde_error);

        assert_eq!(
            error.to_string(),
            "failed to deserialize parsed query into target type: unknown field `mode`; expected one of: (none)"
        );
    }
}
