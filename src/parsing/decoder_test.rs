use super::*;
use std::borrow::Cow;

mod decode_component {
    use super::*;

    fn scratch() -> Vec<u8> {
        Vec::new()
    }

    #[test]
    fn returns_borrowed_slice_for_plain_ascii() {
        // Arrange
        let raw = "simple";
        let mut scratch = scratch();

        // Act
        let result = decode_component(raw, false, 0, &mut scratch).expect("decode ascii");

        // Assert
        assert!(matches!(result, Cow::Borrowed("simple")));
    }

    #[test]
    fn decodes_plus_signs_as_spaces_when_enabled() {
        // Arrange
        let raw = "one+two";
        let mut scratch = scratch();

        // Act
        let result = decode_component(raw, true, 5, &mut scratch).expect("decode plus");

        // Assert
        assert!(matches!(result, Cow::Owned(string) if string == "one two"));
    }

    #[test]
    fn returns_invalid_percent_error_for_truncated_sequence() {
        // Arrange
        let raw = "%2";
        let mut scratch = scratch();

        // Act
        let error = decode_component(raw, false, 10, &mut scratch).expect_err("truncated percent");

        // Assert
        match error {
            ParseError::InvalidPercentEncoding { index } => assert_eq!(index, 10),
            other => panic!("expected InvalidPercentEncoding error, got {other:?}"),
        }
    }

    #[test]
    fn returns_invalid_character_error_for_control_character() {
        // Arrange
        let raw = "bad\u{0007}";
        let mut scratch = scratch();

        // Act
        let error = decode_component(raw, false, 3, &mut scratch).expect_err("control char");

        // Assert
        match error {
            ParseError::InvalidCharacter { character, index } => {
                assert_eq!(character, '\u{0007}');
                assert_eq!(index, 6);
            }
            other => panic!("expected InvalidCharacter error, got {other:?}"),
        }
    }

    #[test]
    fn returns_invalid_utf8_error_for_bad_percent_encoding() {
        // Arrange
        let raw = "%FF";
        let mut scratch = scratch();

        // Act
        let error = decode_component(raw, false, 2, &mut scratch).expect_err("invalid utf8");

        // Assert
        assert!(matches!(error, ParseError::InvalidUtf8));
    }
}
