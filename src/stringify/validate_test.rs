use super::ensure_no_control;

fn ensure(input: &str) -> Result<(), ()> {
    ensure_no_control(input)
}

mod ensure_no_control {
    use super::*;

    #[test]
    fn allows_clean_ascii() {
        // Arrange
        let input = "user=alice&count=42";

        // Act
        let result = ensure(input);

        // Assert
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn allows_unicode_above_control_range() {
        // Arrange
        let input = "café\u{00A0}preview";

        // Act
        let result = ensure(input);

        // Assert
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn rejects_ascii_control_characters() {
        // Arrange
        let controls = ["line1\nline2", "null\0byte"];

        // Act
        let all_rejected = controls.iter().all(|value| ensure(value).is_err());

        // Assert
        assert!(all_rejected);
    }

    #[test]
    fn rejects_delete_character() {
        // Arrange
        let delete = format!("header:{}tail", char::from(0x7F));

        // Act
        let result = ensure(&delete);

        // Assert
        assert!(result.is_err());
    }
}
