use super::{ParseError, preflight};
use crate::config::ParseOptions;

mod preflight_tests {
    use super::*;

    #[test]
    fn when_input_exceeds_max_length_it_returns_error() {
        // Arrange
        let raw = "abcdef";
        let options = ParseOptions {
            max_length: Some(3),
            ..ParseOptions::default()
        };

        // Act
        let result = preflight(raw, &options);

        // Assert
        assert!(matches!(
            result,
            Err(ParseError::InputTooLong { limit }) if limit == 3
        ));
    }

    #[test]
    fn when_leading_question_mark_exists_it_returns_trimmed_slice_and_offset() {
        // Arrange
        let raw = "?foo=bar";
        let options = ParseOptions::default();

        // Act
        let result = preflight(raw, &options).expect("preflight should succeed");

        // Assert
        assert_eq!(result, ("foo=bar", 1));
    }

    #[test]
    fn when_internal_question_mark_is_found_it_returns_error_with_index() {
        // Arrange
        let raw = "a?=1";
        let options = ParseOptions::default();

        // Act
        let result = preflight(raw, &options);

        // Assert
        assert!(matches!(
            result,
            Err(ParseError::UnexpectedQuestionMark { index }) if index == 1
        ));
    }

    #[test]
    fn when_disallowed_control_character_is_present_it_returns_invalid_character() {
        // Arrange
        let raw = format!("foo{}bar", '\u{001F}');
        let options = ParseOptions::default();

        // Act
        let result = preflight(&raw, &options);

        // Assert
        assert!(matches!(
            result,
            Err(ParseError::InvalidCharacter { character, index })
                if character == '\u{001F}' && index == 3
        ));
    }

    #[test]
    fn when_space_character_appears_after_prefix_it_reports_offset_index() {
        // Arrange
        let raw = "?foo bar";
        let options = ParseOptions::default();

        // Act
        let result = preflight(raw, &options);

        // Assert
        assert!(matches!(
            result,
            Err(ParseError::InvalidCharacter { character, index })
                if character == ' ' && index == 4
        ));
    }
}
