use super::ensure_no_control;

fn ensure(input: &str) -> Result<(), ()> {
    ensure_no_control(input)
}

mod ensure_no_control_tests {
    use super::*;

    #[test]
    fn when_input_has_clean_ascii_it_should_allow_value() {
        // Arrange
        let input = "user=alice&count=42";

        // Act
        let result = ensure(input);

        // Assert
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn when_input_contains_unicode_above_control_range_it_should_allow_value() {
        // Arrange
        let input = "café\u{00A0}preview";

        // Act
        let result = ensure(input);

        // Assert
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn when_input_has_ascii_control_characters_it_should_reject_value() {
        // Arrange
        let controls = ["line1\nline2", "null\0byte"];

        // Act
        let all_rejected = controls.iter().all(|value| ensure(value).is_err());

        // Assert
        assert!(all_rejected);
    }

    #[test]
    fn when_input_has_delete_character_it_should_reject_value() {
        // Arrange
        let delete = format!("header:{}tail", char::from(0x7F));

        // Act
        let result = ensure(&delete);

        // Assert
        assert!(result.is_err());
    }
}
