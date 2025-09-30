use super::{ParseError, preflight};
use crate::config::ParseOptions;

mod preflight {
    use super::*;

    #[test]
    fn returns_error_when_input_exceeds_max_length() {
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
    fn returns_trimmed_slice_and_offset_for_leading_question_mark() {
        // Arrange
        let raw = "?foo=bar";
        let options = ParseOptions::default();

        // Act
        let result = preflight(raw, &options).expect("preflight should succeed");

        // Assert
        assert_eq!(result, ("foo=bar", 1));
    }

    #[test]
    fn returns_error_with_index_for_internal_question_mark() {
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
    fn returns_invalid_character_error_for_disallowed_control() {
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
    fn reports_offset_for_space_after_prefix() {
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
