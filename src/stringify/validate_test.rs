use super::ensure_no_control;

fn ensure(input: &str) -> Result<(), ()> {
    ensure_no_control(input)
}

mod ensure_no_control {
    use super::*;

    #[test]
    fn should_allow_clean_ascii_when_input_has_no_control_characters_then_return_ok() {
        // Arrange
        let input = "user=alice&count=42";

        // Act
        let result = ensure(input);

        // Assert
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn should_allow_unicode_when_characters_exceed_control_range_then_return_ok() {
        // Arrange
        let input = "café\u{00A0}preview";

        // Act
        let result = ensure(input);

        // Assert
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn should_reject_ascii_control_characters_when_present_then_return_error() {
        // Arrange
        let controls = ["line1\nline2", "null\0byte"];

        // Act
        let all_rejected = controls.iter().all(|value| ensure(value).is_err());

        // Assert
        assert!(all_rejected);
    }

    #[test]
    fn should_reject_delete_character_when_present_then_return_error() {
        // Arrange
        let delete = format!("header:{}tail", char::from(0x7F));

        // Act
        let result = ensure(&delete);

        // Assert
        assert!(result.is_err());
    }
}
