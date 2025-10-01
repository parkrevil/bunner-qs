use super::Segment;

fn append(initial: &str, segment: Segment<'_>) -> String {
    let mut buffer = String::from(initial);
    super::append_segment(&mut buffer, segment);
    buffer
}

fn append_all(initial: &str, segments: &[Segment<'_>]) -> String {
    segments
        .iter()
        .fold(String::from(initial), |mut buffer, segment| {
            super::append_segment(&mut buffer, *segment);
            buffer
        })
}

mod append_segment {
    use super::*;

    #[test]
    fn should_write_key_for_root_segment_when_segment_is_root_then_write_segment_key() {
        // Arrange
        let initial = "";

        // Act
        let result = append(initial, Segment::Root("profile"));

        // Assert
        assert_eq!(result, "profile");
    }

    #[test]
    fn should_bracket_sub_key_for_object_segment_when_segment_is_object_then_append_bracketed_key() {
        // Arrange
        let initial = "profile";

        // Act
        let result = append(initial, Segment::Object("details"));

        // Assert
        assert_eq!(result, "profile[details]");
    }

    #[test]
    fn should_use_decimal_index_for_array_segment_when_segment_is_array_then_append_decimal_index() {
        // Arrange
        let initial = "items";

        // Act
        let result = append(initial, Segment::Array(42));

        // Assert
        assert_eq!(result, "items[42]");
    }

    #[test]
    fn should_append_zero_digit_for_zero_index_when_segment_index_is_zero_then_append_zero_digit() {
        // Arrange
        let initial = "list";

        // Act
        let result = append(initial, Segment::Array(0));

        // Assert
        assert_eq!(result, "list[0]");
    }

    #[test]
    fn should_build_full_key_path_when_chained_when_multiple_segments_are_appended_then_build_full_path() {
        // Arrange
        let segments = [
            Segment::Root("order"),
            Segment::Object("items"),
            Segment::Array(7),
            Segment::Object("sku"),
        ];

        // Act
        let result = append_all("", &segments);

        // Assert
        assert_eq!(result, "order[items][7][sku]");
    }
}
