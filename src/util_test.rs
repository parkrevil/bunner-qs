use super::is_ascii_control;

mod is_ascii_control {
    use super::is_ascii_control;

    #[test]
    fn should_return_true_when_lower_control_character_provided_then_detect_control() {
        let character = '\n';
        let is_control = is_ascii_control(character);

        assert!(is_control);
    }

    #[test]
    fn should_return_true_when_delete_character_provided_then_detect_control() {
        let character = char::from(0x7F);
        let is_control = is_ascii_control(character);

        assert!(is_control);
    }

    #[test]
    fn should_return_false_when_visible_ascii_provided_then_report_non_control() {
        let character = 'A';
        let is_control = is_ascii_control(character);

        assert!(!is_control);
    }

    #[test]
    fn should_return_false_when_space_character_provided_then_report_non_control() {
        let character = ' ';
        let is_control = is_ascii_control(character);

        assert!(!is_control);
    }

    #[test]
    fn should_return_false_when_non_ascii_unicode_provided_then_report_non_control() {
        let character = '字';
        let is_control = is_ascii_control(character);

        assert!(!is_control);
    }
}
