use super::parse_key_path;

mod parse_key_path {
    use super::*;

    #[test]
    fn should_parse_key_without_brackets_into_single_segment_when_key_contains_no_brackets_then_return_single_segment()
     {
        let input = "profile";

        let segments = parse_key_path(input);

        assert_eq!(segments.as_slice(), ["profile"]);
    }

    #[test]
    fn should_parse_key_with_nested_indices_into_segments_when_key_contains_nested_indices_then_return_nested_indices()
     {
        let input = "user[0][name]";

        let segments = parse_key_path(input);

        assert_eq!(segments.as_slice(), ["user", "0", "name"]);
    }

    #[test]
    fn should_parse_trailing_segment_after_index_when_key_contains_suffix_then_append_trailing_segment()
     {
        let input = "items[42]status";

        let segments = parse_key_path(input);

        assert_eq!(segments.as_slice(), ["items", "42", "status"]);
    }

    #[test]
    fn should_parse_empty_brackets_as_empty_segment_when_key_contains_empty_brackets_then_include_empty_segment()
     {
        let input = "flag[]";

        let segments = parse_key_path(input);

        assert_eq!(segments.as_slice(), ["flag", ""]);
    }

    #[test]
    fn should_collect_remaining_text_when_bracket_is_unmatched_then_append_remaining_text() {
        let input = "foo[bar";

        let segments = parse_key_path(input);

        assert_eq!(segments.as_slice(), ["foo", "bar"]);
    }

    #[test]
    fn should_stop_when_bracket_opens_at_end_then_ignore_incomplete_segment() {
        let input = std::hint::black_box("orphan[");

        let segments = parse_key_path(input);

        assert_eq!(segments.as_slice(), ["orphan"]);
    }

    #[test]
    fn should_return_empty_segments_when_input_is_empty_then_return_empty_result() {
        let input = "";

        let segments = parse_key_path(input);

        assert!(segments.is_empty());
    }
}
