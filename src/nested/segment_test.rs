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
    fn classifies_empty_segment_as_empty() {
        // Arrange
        let input = "";

        // Act
        let kind = SegmentKind::classify(input);

        // Assert
        assert_eq!(kind, SegmentKind::Empty);
    }

    #[test]
    fn classifies_digit_segment_as_numeric() {
        // Arrange
        let inputs = ["123", "0001"];

        // Act
        let kinds = classify_segments(&inputs);

        // Assert
        assert!(kinds.iter().all(|kind| kind == &SegmentKind::Numeric));
    }

    #[test]
    fn classifies_mixed_segment_as_other() {
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
    fn maps_empty_kind_to_array_container() {
        // Arrange
        let kind = SegmentKind::Empty;

        // Act
        let container = kind.container_type();

        // Assert
        assert_eq!(container, ContainerType::Array);
    }

    #[test]
    fn maps_numeric_kind_to_array_container() {
        // Arrange
        let kind = SegmentKind::Numeric;

        // Act
        let container = kind.container_type();

        // Assert
        assert_eq!(container, ContainerType::Array);
    }

    #[test]
    fn maps_other_kind_to_object_container() {
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
    fn clones_bytes_and_allows_borrow_from_str() {
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
    fn preserves_reference_and_kind_for_borrowed_text() {
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
    fn detects_numeric_kind_for_owned_digits() {
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
