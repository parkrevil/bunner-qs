use super::{encode_key_into, encode_value_into};

mod encode_tests {
    use super::{encode_key_into, encode_value_into};

    #[test]
    fn when_encoding_key_with_reserved_characters_it_should_percent_encode_them() {
        let mut buffer = String::new();

        encode_key_into(&mut buffer, "user name+role/section?=true", false);

        assert_eq!(buffer, "user%20name%2Brole%2Fsection%3F%3Dtrue");
    }

    #[test]
    fn when_encoding_value_with_space_as_plus_it_should_replace_spaces_with_plus() {
        let mut buffer = String::new();

        encode_value_into(&mut buffer, "multi word value+more", true);

        assert_eq!(buffer, "multi+word+value%2Bmore");
    }

    #[test]
    fn when_encoding_value_without_space_as_plus_it_should_percent_encode_spaces() {
        let mut buffer = String::new();

        encode_value_into(&mut buffer, "space separated", false);

        assert_eq!(buffer, "space%20separated");
    }

    #[test]
    fn when_component_has_no_reserved_characters_it_should_append_verbatim() {
        let mut buffer = String::from("prefix=");

        encode_key_into(&mut buffer, "alpha_numeric-._~", false);

        assert_eq!(buffer, "prefix=alpha_numeric-._~");
    }

    #[test]
    fn when_component_is_empty_it_should_not_modify_buffer() {
        let mut buffer = String::from("existing");

        encode_key_into(&mut buffer, "", false);
        encode_value_into(&mut buffer, "", true);

        assert_eq!(buffer, "existing");
    }
}
