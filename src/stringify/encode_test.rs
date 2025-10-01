// no direct imports; use fully-qualified calls

fn encode_key(initial: &str, input: &str, space_as_plus: bool) -> String {
    let mut buffer = String::from(initial);
    super::encode_key_into(&mut buffer, input, space_as_plus);
    buffer
}

fn encode_value(initial: &str, input: &str, space_as_plus: bool) -> String {
    let mut buffer = String::from(initial);
    super::encode_value_into(&mut buffer, input, space_as_plus);
    buffer
}

mod encode_key_into {
    use super::*;

    #[test]
    fn should_percent_encode_reserved_characters_when_encoding_key_then_percent_encode_reserved_chars() {
        // Arrange
        let input = "user name+role/section?=true";

        // Act
        let encoded = encode_key("", input, false);

        // Assert
        assert_eq!(encoded, "user%20name%2Brole%2Fsection%3F%3Dtrue");
    }

    #[test]
    fn should_append_verbatim_when_key_contains_only_safe_characters_then_leave_input_unencoded() {
        // Arrange
        let initial = "prefix=";
        let input = "alpha_numeric-._~";

        // Act
        let encoded = encode_key(initial, input, false);

        // Assert
        assert_eq!(encoded, "prefix=alpha_numeric-._~");
    }

    #[test]
    fn should_leave_buffer_unchanged_when_key_component_is_empty_then_preserve_existing_buffer() {
        // Arrange
        let initial = "existing";

        // Act
        let encoded = encode_key(initial, "", false);

        // Assert
        assert_eq!(encoded, "existing");
    }

    #[test]
    fn should_replace_spaces_with_plus_when_key_option_enabled_then_replace_spaces_with_plus() {
        // Arrange
        let input = " dev ops/team ";

        // Act
        let encoded = encode_key("", input, true);

        // Assert
        assert_eq!(encoded, "+dev+ops%2Fteam+");
    }
}

mod encode_value_into {
    use super::*;

    #[test]
    fn should_replace_spaces_with_plus_when_value_option_enabled_then_replace_spaces_with_plus() {
        // Arrange
        let input = "multi word value+more";

        // Act
        let encoded = encode_value("", input, true);

        // Assert
        assert_eq!(encoded, "multi+word+value%2Bmore");
    }

    #[test]
    fn should_percent_encode_spaces_when_plus_option_disabled_for_value_then_encode_spaces_as_percent_twenty() {
        // Arrange
        let input = "space separated";

        // Act
        let encoded = encode_value("", input, false);

        // Assert
        assert_eq!(encoded, "space%20separated");
    }

    #[test]
    fn should_leave_buffer_unchanged_when_value_component_is_empty_then_preserve_existing_buffer() {
        // Arrange
        let initial = "existing";

        // Act
        let encoded = encode_value(initial, "", true);

        // Assert
        assert_eq!(encoded, "existing");
    }

    #[test]
    fn should_percent_encode_reserved_characters_when_value_contains_unicode_then_percent_encode_reserved_chars() {
        // Arrange
        let input = "café/tea";

        // Act
        let encoded = encode_value("seed=", input, true);

        // Assert
        assert_eq!(encoded, "seed=caf%C3%A9%2Ftea");
    }
}
