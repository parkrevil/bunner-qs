use super::{ContainerType, ResolvedSegment, SegmentKey, SegmentKind};
use crate::ParseError;
use assert_matches::assert_matches;
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
        let input = "";

        let kind = SegmentKind::classify(input);

        assert_eq!(kind, SegmentKind::Empty);
    }

    #[test]
    fn should_classify_digit_segment_as_numeric_when_segment_contains_only_digits_then_return_numeric_kind()
     {
        let inputs = ["123", "0001"];

        let kinds = classify_segments(&inputs);

        assert!(kinds.iter().all(|kind| kind == &SegmentKind::Numeric));
    }

    #[test]
    fn should_classify_mixed_segment_as_other_when_segment_contains_non_digit_characters_then_return_other_kind()
     {
        let inputs = ["١٢٣", "42a"];

        let kinds = classify_segments(&inputs);

        assert!(kinds.iter().all(|kind| kind == &SegmentKind::Other));
    }
}

mod container_type {
    use super::*;

    #[test]
    fn should_map_empty_kind_to_array_container_when_kind_is_empty_then_select_array_container() {
        let kind = SegmentKind::Empty;

        let container = kind.container_type();

        assert_eq!(container, ContainerType::Array);
    }

    #[test]
    fn should_map_numeric_kind_to_array_container_when_kind_is_numeric_then_select_array_container()
    {
        let kind = SegmentKind::Numeric;

        let container = kind.container_type();

        assert_eq!(container, ContainerType::Array);
    }

    #[test]
    fn should_map_other_kind_to_object_container_when_kind_is_other_then_select_object_container() {
        let kind = SegmentKind::Other;

        let container = kind.container_type();

        assert_eq!(container, ContainerType::Object);
    }
}

mod segment_key_new {
    use super::*;

    #[test]
    fn should_clone_bytes_and_allow_str_borrow_when_segment_key_is_constructed_then_allow_string_and_byte_access()
     {
        let source = String::from("status");

        let key = SegmentKey::new(&source);

        drop(source);
        assert_matches!(key.as_str(), Ok("status"));
        assert_eq!(<SegmentKey as Borrow<[u8]>>::borrow(&key), b"status");
    }
}

mod segment_key_debug {
    use super::*;
    use smallvec::SmallVec;

    #[test]
    fn should_return_invalid_utf8_error_when_segment_bytes_are_not_valid_utf8_then_surface_parse_error()
     {
        let key = SegmentKey(SmallVec::from_slice(&[0xFF]));

        let result = key.as_str();

        assert_matches!(result, Err(ParseError::InvalidUtf8));
    }

    #[test]
    fn should_include_fallback_marker_when_segment_contains_invalid_utf8_bytes_then_show_debug_marker()
     {
        let key = SegmentKey(SmallVec::from_slice(&[0xF0, 0x28]));

        let formatted = format!("{key:?}");

        assert!(formatted.contains("<invalid utf-8"));
        assert!(formatted.contains("SegmentKey"));
    }

    #[test]
    fn should_display_text_segment_when_utf8_is_valid_then_show_original_text() {
        let key = SegmentKey::new("status");

        let formatted = format!("{key:?}");

        assert!(formatted.contains("status"));
        assert!(formatted.contains("SegmentKey"));
    }
}

mod resolved_segment_new {
    use super::*;

    #[test]
    fn should_preserve_reference_and_kind_for_borrowed_text_when_resolved_segment_wraps_borrowed_text_then_preserve_reference_and_kind()
     {
        let segment = ResolvedSegment::new(Cow::Borrowed("items"));

        let text = segment.as_str();
        let kind = segment.kind;

        assert_eq!(text, "items");
        assert_eq!(kind, SegmentKind::Other);
    }

    #[test]
    fn should_detect_numeric_kind_for_owned_digits_when_resolved_segment_owns_digit_string_then_detect_numeric_kind()
     {
        let segment = ResolvedSegment::new(Cow::Owned(String::from("123")));

        let text = segment.as_str();
        let kind = segment.kind;

        assert_eq!(text, "123");
        assert_eq!(kind, SegmentKind::Numeric);
    }
}
