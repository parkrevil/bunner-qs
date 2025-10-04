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
    fn should_percent_encode_reserved_characters_when_encoding_key_then_percent_encode_reserved_chars()
     {
        let input = "user name+role/section?=true";

        let encoded = encode_key("", input, false);

        assert_eq!(encoded, "user%20name%2Brole%2Fsection%3F%3Dtrue");
    }

    #[test]
    fn should_append_verbatim_when_key_contains_only_safe_characters_then_leave_input_unencoded() {
        let initial = "prefix=";
        let input = "alpha_numeric-._~";

        let encoded = encode_key(initial, input, false);

        assert_eq!(encoded, "prefix=alpha_numeric-._~");
    }

    #[test]
    fn should_leave_buffer_unchanged_when_key_component_is_empty_then_preserve_existing_buffer() {
        let initial = "existing";

        let encoded = encode_key(initial, "", false);

        assert_eq!(encoded, "existing");
    }

    #[test]
    fn should_replace_spaces_with_plus_when_key_option_enabled_then_replace_spaces_with_plus() {
        let input = " dev ops/team ";

        let encoded = encode_key("", input, true);

        assert_eq!(encoded, "+dev+ops%2Fteam+");
    }
}

mod encode_value_into {
    use super::*;

    #[test]
    fn should_replace_spaces_with_plus_when_value_option_enabled_then_replace_spaces_with_plus() {
        let input = "multi word value+more";

        let encoded = encode_value("", input, true);

        assert_eq!(encoded, "multi+word+value%2Bmore");
    }

    #[test]
    fn should_percent_encode_spaces_when_plus_option_disabled_for_value_then_encode_spaces_as_percent_twenty()
     {
        let input = "space separated";

        let encoded = encode_value("", input, false);

        assert_eq!(encoded, "space%20separated");
    }

    #[test]
    fn should_leave_buffer_unchanged_when_value_component_is_empty_then_preserve_existing_buffer() {
        let initial = "existing";

        let encoded = encode_value(initial, "", true);

        assert_eq!(encoded, "existing");
    }

    #[test]
    fn should_percent_encode_reserved_characters_when_value_contains_unicode_then_percent_encode_reserved_chars()
     {
        let input = "café/tea";

        let encoded = encode_value("seed=", input, true);

        assert_eq!(encoded, "seed=caf%C3%A9%2Ftea");
    }

    #[test]
    fn should_convert_consecutive_spaces_to_plus_when_option_enabled_then_emit_plus_for_each_gap() {
        let input = "many   spaces";

        let encoded = encode_value("", input, true);

        assert_eq!(encoded, "many+++spaces");
    }

    #[test]
    fn should_encode_without_space_conversion_when_option_enabled_but_no_spaces_present_then_use_percent_encoding()
     {
        let input = "array/values?";

        let encoded = encode_value("", input, true);

        assert_eq!(encoded, "array%2Fvalues%3F");
    }
}

mod build_component_set {
    #[test]
    fn should_mark_expected_bytes_for_encoding_when_building_component_set_then_include_reserved_characters()
     {
        let component_set = Box::leak(Box::new(super::super::build_component_set()));
        let encoded = percent_encoding::utf8_percent_encode(" +-_", component_set).to_string();

        assert_eq!(encoded, "%20%2B-_");
    }
}

mod append_encoded {
    #[test]
    fn should_leave_buffer_unchanged_when_segment_is_empty_then_skip_encoding() {
        let mut buffer = String::from("seed");
        let segment = std::hint::black_box("");

        super::super::append_encoded(segment, &mut buffer);

        assert_eq!(buffer, "seed");
    }
}
