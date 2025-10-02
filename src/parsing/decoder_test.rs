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
    fn should_decode_plus_signs_as_spaces_when_space_as_plus_is_enabled_then_convert_plus_to_space()
    {
        // Arrange
        let raw = "one+two";
        let mut scratch = scratch();

        // Act
        let result = decode_component(raw, true, 5, &mut scratch).expect("decode plus");

        // Assert
        assert!(matches!(result, Cow::Owned(string) if string == "one two"));
    }

    #[test]
    fn should_decode_leading_lowercase_hex_sequence_then_return_expected_character() {
        // Arrange
        let raw = "%2a";
        let mut scratch = scratch();

        // Act
        let result = decode_component(raw, false, 0, &mut scratch)
            .expect("leading lowercase percent sequence should decode");

        // Assert
        assert!(matches!(result, Cow::Owned(text) if text == "*"));
    }

    #[test]
    fn should_resolve_lowercase_hex_digits_using_helper_then_return_expected_value() {
        // Act
        let lower_a = super::hex_value(b'a');
        let lower_f = super::hex_value(b'f');

        // Assert
        assert_eq!(lower_a, Some(10));
        assert_eq!(lower_f, Some(15));
    }

    #[test]
    fn should_return_invalid_percent_error_when_second_hex_digit_is_invalid_then_report_index() {
        // Arrange
        let raw = "%2G";
        let mut scratch = scratch();

        // Act
        let error = decode_component(raw, false, 12, &mut scratch)
            .expect_err("invalid second hex digit should fail");

        // Assert
        match error {
            ParseError::InvalidPercentEncoding { index } => assert_eq!(index, 12),
            other => panic!("expected InvalidPercentEncoding error, got {other:?}"),
        }
    }

    #[test]
    fn should_return_invalid_percent_error_when_sequence_is_truncated_then_report_truncation_index()
    {
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
    fn should_return_invalid_character_error_when_control_character_is_present_then_report_character_and_index()
     {
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
    fn should_return_invalid_utf8_error_when_percent_encoding_is_invalid_then_signal_invalid_utf8()
    {
        // Arrange
        let raw = "%FF";
        let mut scratch = scratch();

        // Act
        let error = decode_component(raw, false, 2, &mut scratch).expect_err("invalid utf8");

        // Assert
        assert!(matches!(error, ParseError::InvalidUtf8));
    }

    #[test]
    fn should_return_invalid_percent_error_when_hex_digit_is_invalid_then_report_index() {
        // Arrange
        let raw = "%4Z";
        let mut scratch = scratch();

        // Act
        let error = decode_component(raw, false, 7, &mut scratch)
            .expect_err("invalid hex digit should error");

        // Assert
        match error {
            ParseError::InvalidPercentEncoding { index } => assert_eq!(index, 7),
            other => panic!("expected InvalidPercentEncoding error, got {other:?}"),
        }
    }

    #[test]
    fn should_return_invalid_character_when_ascii_run_contains_control_after_percent_sequence() {
        // Arrange
        let raw = "%20ok\u{001F}";
        let mut scratch = scratch();

        // Act
        let error = decode_component(raw, false, 0, &mut scratch)
            .expect_err("control character should be rejected");

        // Assert
        match error {
            ParseError::InvalidCharacter { character, index } => {
                assert_eq!(character, '\u{001F}');
                assert_eq!(index, 5);
            }
            other => panic!("expected InvalidCharacter error, got {other:?}"),
        }
    }

    #[test]
    fn should_report_invalid_character_when_control_follows_percent_sequence_then_return_error() {
        // Arrange
        let raw = "%41\u{0007}";
        let mut scratch = scratch();

        // Act
        let error = decode_component(raw, false, 0, &mut scratch)
            .expect_err("control character should trigger error");

        // Assert
        match error {
            ParseError::InvalidCharacter { character, index } => {
                assert_eq!(character, '\u{0007}');
                assert_eq!(index, 3);
            }
            other => panic!("expected InvalidCharacter error, got {other:?}"),
        }
    }

    #[test]
    fn should_preserve_multibyte_segments_when_present_with_percent_encoding_then_collect_utf8() {
        // Arrange
        let raw = "😊%20";
        let mut scratch = scratch();

        // Act
        let result = decode_component(raw, false, 0, &mut scratch)
            .expect("emoji plus percent should decode");

        // Assert
        assert!(matches!(result, Cow::Owned(text) if text == "😊 "));
    }

    #[test]
    fn should_decode_lowercase_hex_sequence_then_normalize_percent_encoding() {
        // Arrange
        let raw = "stars%2a%2a";
        let mut scratch = scratch();

        // Act
        let result = decode_component(raw, false, 0, &mut scratch).expect("lowercase hex");

        // Assert
        assert!(matches!(result, Cow::Owned(text) if text == "stars**"));
    }

    #[test]
    fn should_decode_plus_with_ascii_prefix_and_suffix_then_split_runs_correctly() {
        // Arrange
        let raw = "pre+mid+post";
        let mut scratch = scratch();

        // Act
        let result =
            decode_component(raw, true, 100, &mut scratch).expect("plus signs should decode");

        // Assert
        assert!(matches!(result, Cow::Owned(text) if text == "pre mid post"));
    }
}
