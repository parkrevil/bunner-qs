use super::{encode_key_into, encode_value_into};

fn encode_key(initial: &str, input: &str, space_as_plus: bool) -> String {
    let mut buffer = String::from(initial);
    encode_key_into(&mut buffer, input, space_as_plus);
    buffer
}

fn encode_value(initial: &str, input: &str, space_as_plus: bool) -> String {
    let mut buffer = String::from(initial);
    encode_value_into(&mut buffer, input, space_as_plus);
    buffer
}

mod encode_key_into_tests {
    use super::*;

    #[test]
    fn when_encoding_key_with_reserved_characters_it_should_percent_encode_them() {
        // Arrange
        let input = "user name+role/section?=true";

        // Act
        let encoded = encode_key("", input, false);

        // Assert
        assert_eq!(encoded, "user%20name%2Brole%2Fsection%3F%3Dtrue");
    }

    #[test]
    fn when_component_has_no_reserved_characters_it_should_append_verbatim() {
        // Arrange
        let initial = "prefix=";
        let input = "alpha_numeric-._~";

        // Act
        let encoded = encode_key(initial, input, false);

        // Assert
        assert_eq!(encoded, "prefix=alpha_numeric-._~");
    }

    #[test]
    fn when_component_is_empty_it_should_not_modify_buffer() {
        // Arrange
        let initial = "existing";

        // Act
        let encoded = encode_key(initial, "", false);

        // Assert
        assert_eq!(encoded, "existing");
    }

    #[test]
    fn when_space_as_plus_is_enabled_it_should_replace_each_space_with_plus() {
        // Arrange
        let input = " dev ops/team ";

        // Act
        let encoded = encode_key("", input, true);

        // Assert
        assert_eq!(encoded, "+dev+ops%2Fteam+");
    }
}

mod encode_value_into_tests {
    use super::*;

    #[test]
    fn when_encoding_value_with_space_as_plus_it_should_replace_spaces_with_plus() {
        // Arrange
        let input = "multi word value+more";

        // Act
        let encoded = encode_value("", input, true);

        // Assert
        assert_eq!(encoded, "multi+word+value%2Bmore");
    }

    #[test]
    fn when_encoding_value_without_space_as_plus_it_should_percent_encode_spaces() {
        // Arrange
        let input = "space separated";

        // Act
        let encoded = encode_value("", input, false);

        // Assert
        assert_eq!(encoded, "space%20separated");
    }

    #[test]
    fn when_component_is_empty_it_should_not_modify_buffer() {
        // Arrange
        let initial = "existing";

        // Act
        let encoded = encode_value(initial, "", true);

        // Assert
        assert_eq!(encoded, "existing");
    }

    #[test]
    fn when_space_as_plus_is_enabled_without_spaces_it_should_percent_encode_reserved_characters() {
        // Arrange
        let input = "café/tea";

        // Act
        let encoded = encode_value("seed=", input, true);

        // Assert
        assert_eq!(encoded, "seed=caf%C3%A9%2Ftea");
    }
}
