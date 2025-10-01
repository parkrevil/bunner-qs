use super::*;
use std::borrow::Cow;

mod decode_component {
    use super::*;

    fn scratch() -> Vec<u8> {
        Vec::new()
    }

    #[test]
    fn should_return_borrowed_slice_when_input_is_plain_ascii_then_avoid_allocation() {
        // Arrange
        let raw = "simple";
        let mut scratch = scratch();

        // Act
        let result = decode_component(raw, false, 0, &mut scratch).expect("decode ascii");

        // Assert
        assert!(matches!(result, Cow::Borrowed("simple")));
    }

    #[test]
    fn should_decode_plus_signs_as_spaces_when_space_as_plus_is_enabled_then_convert_plus_to_space() {
        // Arrange
        let raw = "one+two";
        let mut scratch = scratch();

        // Act
        let result = decode_component(raw, true, 5, &mut scratch).expect("decode plus");

        // Assert
        assert!(matches!(result, Cow::Owned(string) if string == "one two"));
    }

    #[test]
    fn should_return_invalid_percent_error_when_sequence_is_truncated_then_report_truncation_index() {
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
    fn should_return_invalid_character_error_when_control_character_is_present_then_report_character_and_index() {
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
    fn should_return_invalid_utf8_error_when_percent_encoding_is_invalid_then_signal_invalid_utf8() {
        // Arrange
        let raw = "%FF";
        let mut scratch = scratch();

        // Act
        let error = decode_component(raw, false, 2, &mut scratch).expect_err("invalid utf8");

        // Assert
        assert!(matches!(error, ParseError::InvalidUtf8));
    }
}
