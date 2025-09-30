use super::*;
use std::borrow::Cow;

mod decode_component {
    use super::*;

    fn scratch() -> Vec<u8> {
        Vec::new()
    }

    #[test]
    fn when_ascii_input_has_no_specials_it_should_return_borrowed_slice() {
        // Arrange
        let raw = "simple";
        let mut scratch = scratch();

        // Act
        let result = decode_component(raw, false, 0, &mut scratch).expect("decode ascii");

        // Assert
        assert!(matches!(result, Cow::Borrowed("simple")));
    }

    #[test]
    fn when_plus_signs_are_treated_as_spaces_it_should_decode_to_spaces() {
        // Arrange
        let raw = "one+two";
        let mut scratch = scratch();

        // Act
        let result = decode_component(raw, true, 5, &mut scratch).expect("decode plus");

        // Assert
        assert!(matches!(result, Cow::Owned(string) if string == "one two"));
    }

    #[test]
    fn when_percent_sequence_is_truncated_it_should_return_invalid_percent_error() {
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
    fn when_control_character_is_present_it_should_return_invalid_character_error() {
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
    fn when_percent_decodes_to_invalid_utf8_it_should_return_invalid_utf8_error() {
        // Arrange
        let raw = "%FF";
        let mut scratch = scratch();

        // Act
        let error = decode_component(raw, false, 2, &mut scratch).expect_err("invalid utf8");

        // Assert
        assert!(matches!(error, ParseError::InvalidUtf8));
    }
}
