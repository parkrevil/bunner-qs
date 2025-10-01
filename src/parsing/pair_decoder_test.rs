use super::*;
use crate::parsing::ParseError;
use std::borrow::Cow;

mod decode_pair {
    use super::*;

    #[test]
    fn should_return_borrowed_components_when_plain_ascii_then_avoid_allocation() {
        // Arrange
        let options = ParseOptions::default();
        let mut scratch = Vec::new();

        // Act
        let (key, value) =
            decode_pair("foo", "bar", 0, 4, &options, &mut scratch).expect("decode succeeds");

        // Assert
        assert!(matches!(key, Cow::Borrowed("foo")));
        assert!(matches!(value, Cow::Borrowed("bar")));
        assert!(scratch.is_empty());
    }

    #[test]
    fn should_decode_plus_signs_when_space_as_plus_enabled_then_replace_with_spaces() {
        // Arrange
        let options = ParseOptions {
            space_as_plus: true,
            ..ParseOptions::default()
        };
        let mut scratch = Vec::new();

        // Act
        let (key, value) = decode_pair("hello+world", "value+here", 0, 17, &options, &mut scratch)
            .expect("decode succeeds");

        // Assert
        assert_eq!(key.as_ref(), "hello world");
        assert_eq!(value.as_ref(), "value here");
    }

    #[test]
    fn should_return_error_when_brackets_unmatched_then_return_unmatched_bracket_error() {
        // Arrange
        let options = ParseOptions::default();
        let mut scratch = Vec::new();

        // Act
        let error =
            decode_pair("foo[", "bar", 0, 4, &options, &mut scratch).expect_err("decode fails");

        // Assert
        assert!(matches!(error, ParseError::UnmatchedBracket { ref key } if key == "foo["));
    }
}
