use super::parse_key_path;

mod parse_key_path {
    use super::*;

    #[test]
    fn when_key_has_no_brackets_it_should_return_single_segment() {
        // Arrange
        let input = "profile";

        // Act
        let segments = parse_key_path(input);

        // Assert
        assert_eq!(segments.as_slice(), ["profile"]);
    }

    #[test]
    fn when_key_has_nested_indices_it_should_expand_all_segments() {
        // Arrange
        let input = "user[0][name]";

        // Act
        let segments = parse_key_path(input);

        // Assert
        assert_eq!(segments.as_slice(), ["user", "0", "name"]);
    }

    #[test]
    fn when_key_has_trailing_segment_it_should_append_tail() {
        // Arrange
        let input = "items[42]status";

        // Act
        let segments = parse_key_path(input);

        // Assert
        assert_eq!(segments.as_slice(), ["items", "42", "status"]);
    }

    #[test]
    fn when_key_has_empty_brackets_it_should_include_empty_segment() {
        // Arrange
        let input = "flag[]";

        // Act
        let segments = parse_key_path(input);

        // Assert
        assert_eq!(segments.as_slice(), ["flag", ""]);
    }

    #[test]
    fn when_bracket_is_unmatched_it_should_collect_remaining_text() {
        // Arrange
        let input = "foo[bar";

        // Act
        let segments = parse_key_path(input);

        // Assert
        assert_eq!(segments.as_slice(), ["foo", "bar"]);
    }

    #[test]
    fn when_input_is_empty_it_should_return_empty_segments() {
        // Arrange
        let input = "";

        // Act
        let segments = parse_key_path(input);

        // Assert
        assert!(segments.is_empty());
    }
}
