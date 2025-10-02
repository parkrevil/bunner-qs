use super::{ContainerType, ResolvedSegment, SegmentKey, SegmentKind};
use crate::ParseError;
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
    fn should_classify_empty_segment_as_empty_when_segment_is_empty_then_return_empty_kind() {
        // Arrange
        let input = "";

        // Act
        let kind = SegmentKind::classify(input);

        // Assert
        assert_eq!(kind, SegmentKind::Empty);
    }

    #[test]
    fn should_classify_digit_segment_as_numeric_when_segment_contains_only_digits_then_return_numeric_kind()
     {
        // Arrange
        let inputs = ["123", "0001"];

        // Act
        let kinds = classify_segments(&inputs);

        // Assert
        assert!(kinds.iter().all(|kind| kind == &SegmentKind::Numeric));
    }

    #[test]
    fn should_classify_mixed_segment_as_other_when_segment_contains_non_digit_characters_then_return_other_kind()
     {
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
    fn should_map_empty_kind_to_array_container_when_kind_is_empty_then_select_array_container() {
        // Arrange
        let kind = SegmentKind::Empty;

        // Act
        let container = kind.container_type();

        // Assert
        assert_eq!(container, ContainerType::Array);
    }

    #[test]
    fn should_map_numeric_kind_to_array_container_when_kind_is_numeric_then_select_array_container()
    {
        // Arrange
        let kind = SegmentKind::Numeric;

        // Act
        let container = kind.container_type();

        // Assert
        assert_eq!(container, ContainerType::Array);
    }

    #[test]
    fn should_map_other_kind_to_object_container_when_kind_is_other_then_select_object_container() {
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
    fn should_clone_bytes_and_allow_str_borrow_when_segment_key_is_constructed_then_allow_string_and_byte_access()
     {
        // Arrange
        let source = String::from("status");

        // Act
        let key = SegmentKey::new(&source);

        // Assert
        drop(source);
        assert!(matches!(key.as_str(), Ok("status")));
        assert_eq!(<SegmentKey as Borrow<[u8]>>::borrow(&key), b"status");
    }
}

mod segment_key_debug {
    use super::*;
    use smallvec::SmallVec;

    #[test]
    fn should_return_invalid_utf8_error_when_segment_bytes_are_not_valid_utf8() {
        // Arrange
        let key = SegmentKey(SmallVec::from_slice(&[0xFF]));

        // Act
        let result = key.as_str();

        // Assert
        assert!(matches!(result, Err(ParseError::InvalidUtf8)));
    }

    #[test]
    fn should_include_fallback_marker_in_debug_output_when_segment_contains_invalid_utf8_bytes() {
        // Arrange
        let key = SegmentKey(SmallVec::from_slice(&[0xF0, 0x28]));

        // Act
        let formatted = format!("{key:?}");

        // Assert
        assert!(formatted.contains("<invalid utf-8"));
        assert!(formatted.contains("SegmentKey"));
    }
}

mod resolved_segment_new {
    use super::*;

    #[test]
    fn should_preserve_reference_and_kind_for_borrowed_text_when_resolved_segment_wraps_borrowed_text_then_preserve_reference_and_kind()
     {
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
    fn should_detect_numeric_kind_for_owned_digits_when_resolved_segment_owns_digit_string_then_detect_numeric_kind()
     {
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
