use super::{ContainerType, ResolvedSegment, SegmentKey, SegmentKind};
use std::borrow::{Borrow, Cow};

mod classify {
    use super::*;

    fn classify_segments(inputs: &[&str]) -> Vec<SegmentKind> {
        inputs
            .iter()
            .map(|segment| SegmentKind::classify(segment))
            .collect()
    }

    #[test]
    fn when_segment_is_empty_it_should_return_empty_kind() {
        // Arrange
        let input = "";

        // Act
        let kind = SegmentKind::classify(input);

        // Assert
        assert_eq!(kind, SegmentKind::Empty);
    }

    #[test]
    fn when_segment_has_only_digits_it_should_return_numeric_kind() {
        // Arrange
        let inputs = ["123", "0001"];

        // Act
        let kinds = classify_segments(&inputs);

        // Assert
        assert!(kinds.iter().all(|kind| kind == &SegmentKind::Numeric));
    }

    #[test]
    fn when_segment_contains_non_numeric_it_should_return_other_kind() {
        // Arrange
        let inputs = ["١٢٣", "42a"];

        // Act
        let kinds = classify_segments(&inputs);

        // Assert
        assert!(kinds.iter().all(|kind| kind == &SegmentKind::Other));
    }
}

mod container_type {
    use super::*;

    #[test]
    fn when_kind_is_empty_it_should_map_to_array_container() {
        // Arrange
        let kind = SegmentKind::Empty;

        // Act
        let container = kind.container_type();

        // Assert
        assert_eq!(container, ContainerType::Array);
    }

    #[test]
    fn when_kind_is_numeric_it_should_map_to_array_container() {
        // Arrange
        let kind = SegmentKind::Numeric;

        // Act
        let container = kind.container_type();

        // Assert
        assert_eq!(container, ContainerType::Array);
    }

    #[test]
    fn when_kind_is_other_it_should_map_to_object_container() {
        // Arrange
        let kind = SegmentKind::Other;

        // Act
        let container = kind.container_type();

        // Assert
        assert_eq!(container, ContainerType::Object);
    }
}

mod segment_key_new {
    use super::*;

    #[test]
    fn when_created_from_str_it_should_clone_bytes_and_allow_borrow() {
        // Arrange
        let source = String::from("status");

        // Act
        let key = SegmentKey::new(&source);

        // Assert
        drop(source);
        assert_eq!(key.as_str(), "status");
        assert_eq!(<SegmentKey as Borrow<[u8]>>::borrow(&key), b"status");
    }
}

mod resolved_segment_new {
    use super::*;

    #[test]
    fn when_wrapping_borrowed_text_it_should_preserve_reference_and_kind() {
        // Arrange
        let segment = ResolvedSegment::new(Cow::Borrowed("items"));

        // Act
        let text = segment.as_str();
        let kind = segment.kind;

        // Assert
        assert_eq!(text, "items");
        assert_eq!(kind, SegmentKind::Other);
    }

    #[test]
    fn when_wrapping_owned_digits_it_should_detect_numeric_kind() {
        // Arrange
        let segment = ResolvedSegment::new(Cow::Owned(String::from("123")));

        // Act
        let text = segment.as_str();
        let kind = segment.kind;

        // Assert
        assert_eq!(text, "123");
        assert_eq!(kind, SegmentKind::Numeric);
    }
}
