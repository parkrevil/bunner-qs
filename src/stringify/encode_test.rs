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
    fn percent_encodes_reserved_characters() {
        // Arrange
        let input = "user name+role/section?=true";

        // Act
        let encoded = encode_key("", input, false);

        // Assert
        assert_eq!(encoded, "user%20name%2Brole%2Fsection%3F%3Dtrue");
    }

    #[test]
    fn appends_verbatim_when_no_reserved_characters() {
        // Arrange
        let initial = "prefix=";
        let input = "alpha_numeric-._~";

        // Act
        let encoded = encode_key(initial, input, false);

        // Assert
        assert_eq!(encoded, "prefix=alpha_numeric-._~");
    }

    #[test]
    fn does_not_modify_buffer_for_empty_component() {
        // Arrange
        let initial = "existing";

        // Act
        let encoded = encode_key(initial, "", false);

        // Assert
        assert_eq!(encoded, "existing");
    }

    #[test]
    fn replaces_spaces_with_plus_when_enabled() {
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
    fn replaces_spaces_with_plus_when_enabled() {
        // Arrange
        let input = "multi word value+more";

        // Act
        let encoded = encode_value("", input, true);

        // Assert
        assert_eq!(encoded, "multi+word+value%2Bmore");
    }

    #[test]
    fn percent_encodes_spaces_when_plus_disabled() {
        // Arrange
        let input = "space separated";

        // Act
        let encoded = encode_value("", input, false);

        // Assert
        assert_eq!(encoded, "space%20separated");
    }

    #[test]
    fn does_not_modify_buffer_for_empty_component() {
        // Arrange
        let initial = "existing";

        // Act
        let encoded = encode_value(initial, "", true);

        // Assert
        assert_eq!(encoded, "existing");
    }

    #[test]
    fn percent_encodes_reserved_characters_without_spaces() {
        // Arrange
        let input = "café/tea";

        // Act
        let encoded = encode_value("seed=", input, true);

        // Assert
        assert_eq!(encoded, "seed=caf%C3%A9%2Ftea");
    }
}
