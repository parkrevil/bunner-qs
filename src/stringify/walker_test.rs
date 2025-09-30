use super::{Segment, append_segment};

fn append(initial: &str, segment: Segment<'_>) -> String {
    let mut buffer = String::from(initial);
    append_segment(&mut buffer, segment);
    buffer
}

fn append_all(initial: &str, segments: &[Segment<'_>]) -> String {
    segments
        .iter()
        .fold(String::from(initial), |mut buffer, segment| {
            append_segment(&mut buffer, *segment);
            buffer
        })
}

mod append_segment_tests {
    use super::*;

    #[test]
    fn when_appending_root_segment_it_should_write_key_directly() {
        // Arrange
        let initial = "";

        // Act
        let result = append(initial, Segment::Root("profile"));

        // Assert
        assert_eq!(result, "profile");
    }

    #[test]
    fn when_appending_object_segment_it_should_bracket_sub_key() {
        // Arrange
        let initial = "profile";

        // Act
        let result = append(initial, Segment::Object("details"));

        // Assert
        assert_eq!(result, "profile[details]");
    }

    #[test]
    fn when_appending_array_segment_it_should_use_decimal_index() {
        // Arrange
        let initial = "items";

        // Act
        let result = append(initial, Segment::Array(42));

        // Assert
        assert_eq!(result, "items[42]");
    }

    #[test]
    fn when_array_index_is_zero_it_should_append_single_zero_digit() {
        // Arrange
        let initial = "list";

        // Act
        let result = append(initial, Segment::Array(0));

        // Assert
        assert_eq!(result, "list[0]");
    }

    #[test]
    fn when_chaining_segments_it_should_build_full_key_path() {
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
