use super::ensure_no_control;

fn ensure(input: &str) -> Result<(), ()> {
    ensure_no_control(input)
}

mod ensure_no_control {
    use super::*;

    #[test]
    fn should_allow_clean_ascii_when_input_has_no_control_characters_then_return_ok() {
        let input = "user=alice&count=42";

        let result = ensure(input);

        assert_eq!(result, Ok(()));
    }

    #[test]
    fn should_allow_unicode_when_characters_exceed_control_range_then_return_ok() {
        let input = "café\u{00A0}preview";

        let result = ensure(input);

        assert_eq!(result, Ok(()));
    }

    #[test]
    fn should_reject_ascii_control_characters_when_present_then_return_error() {
        let controls = ["line1\nline2", "null\0byte"];

        let all_rejected = controls.iter().all(|value| ensure(value).is_err());

        assert!(all_rejected);
    }

    #[test]
    fn should_reject_delete_character_when_present_then_return_error() {
        let delete = format!("header:{}tail", char::from(0x7F));

        let result = ensure(&delete);

        assert!(result.is_err());
    }
}
