use super::{ParseError, preflight};
use crate::config::ParseOptions;
use crate::parsing::errors::ParseLocation;
use assert_matches::assert_matches;

mod preflight {
    use super::*;

    #[test]
    fn should_return_error_when_input_exceeds_max_length_then_report_limit() {
        let raw = "abcdef";
        let options = ParseOptions {
            max_length: Some(3),
            ..ParseOptions::default()
        };

        let result = preflight(raw, &options);

        assert_matches!(
            result,
            Err(ParseError::InputTooLong { limit, actual: _ }) if limit == 3
        );
    }

    #[test]
    fn should_return_trimmed_slice_and_offset_when_leading_question_mark_present_then_strip_prefix()
    {
        let raw = "?foo=bar";
        let options = ParseOptions::default();

        let result = preflight(raw, &options).expect("preflight should succeed");

        assert_eq!(result, ("foo=bar", 1));
    }

    #[test]
    fn should_return_error_with_index_when_internal_question_mark_found_then_report_position() {
        let raw = "a?=1";
        let options = ParseOptions::default();

        let result = preflight(raw, &options);

        assert_matches!(
            result,
            Err(ParseError::UnexpectedQuestionMark { index, location }) if index == 1
                && location == ParseLocation::Query
        );
    }

    #[test]
    fn should_return_invalid_character_error_when_control_character_disallowed_then_report_character_and_index()
     {
        let raw = format!("foo{}bar", '\u{001F}');
        let options = ParseOptions::default();

        let result = preflight(&raw, &options);

        assert_matches!(
            result,
            Err(ParseError::InvalidCharacter {
                character,
                index,
                location,
            }) if character == '\u{001F}' && index == 3 && location == ParseLocation::Query
        );
    }

    #[test]
    fn should_report_offset_when_space_after_prefix_detected_then_return_invalid_character_error() {
        let raw = "?foo bar";
        let options = ParseOptions::default();

        let result = preflight(raw, &options);

        assert_matches!(
            result,
            Err(ParseError::InvalidCharacter {
                character,
                index,
                location,
            }) if character == ' ' && index == 4 && location == ParseLocation::Query
        );
    }

    #[test]
    fn should_return_empty_trimmed_slice_when_only_prefix_present_then_preserve_offset() {
        let raw = "?";
        let options = ParseOptions::default();

        let result = preflight(raw, &options).expect("preflight should succeed");

        assert_eq!(result, ("", 1));
    }

    #[test]
    fn should_report_invalid_character_when_space_present_without_prefix_then_report_position() {
        let raw = "foo bar";
        let options = ParseOptions::default();

        let result = preflight(raw, &options);

        assert_matches!(
            result,
            Err(ParseError::InvalidCharacter {
                character,
                index,
                location,
            }) if character == ' ' && index == 3 && location == ParseLocation::Query
        );
    }
}
