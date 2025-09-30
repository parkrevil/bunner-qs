use super::parse_key_path;

mod parse_key_path {
    use super::*;

    #[test]
    fn parses_key_without_brackets_into_single_segment() {
        // Arrange
        let input = "profile";

        // Act
        let segments = parse_key_path(input);

        // Assert
        assert_eq!(segments.as_slice(), ["profile"]);
    }

    #[test]
    fn parses_key_with_nested_indices_into_segments() {
        // Arrange
        let input = "user[0][name]";

        // Act
        let segments = parse_key_path(input);

        // Assert
        assert_eq!(segments.as_slice(), ["user", "0", "name"]);
    }

    #[test]
    fn parses_trailing_segment_after_index() {
        // Arrange
        let input = "items[42]status";

        // Act
        let segments = parse_key_path(input);

        // Assert
        assert_eq!(segments.as_slice(), ["items", "42", "status"]);
    }

    #[test]
    fn parses_empty_brackets_as_empty_segment() {
        // Arrange
        let input = "flag[]";

        // Act
        let segments = parse_key_path(input);

        // Assert
        assert_eq!(segments.as_slice(), ["flag", ""]);
    }

    #[test]
    fn collects_remaining_text_for_unmatched_bracket() {
        // Arrange
        let input = "foo[bar";

        // Act
        let segments = parse_key_path(input);

        // Assert
        assert_eq!(segments.as_slice(), ["foo", "bar"]);
    }

    #[test]
    fn returns_empty_segments_for_empty_input() {
        // Arrange
        let input = "";

        // Act
        let segments = parse_key_path(input);

        // Assert
        assert!(segments.is_empty());
    }
}
