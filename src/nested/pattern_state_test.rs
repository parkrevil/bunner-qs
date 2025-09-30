use super::{ContainerType, ResolvedSegment, acquire_pattern_state};
use crate::ParseError;
use std::borrow::Cow;

fn make_segments<'a>(parts: &'a [&'a str]) -> Vec<ResolvedSegment<'a>> {
    parts
        .iter()
        .map(|segment| ResolvedSegment::new(Cow::Borrowed(*segment)))
        .collect()
}

mod resolve {
    use super::*;

    #[test]
    fn when_numeric_segment_repeats_it_should_increment_indices() {
        // Arrange
        let mut guard = acquire_pattern_state();
        let path = make_segments(&["items"]);

        // Act
        let first = guard.resolve(&path, "", "items").expect("first");
        let second = guard.resolve(&path, "", "items").expect("second");

        // Assert
        assert_eq!(first, "0");
        assert_eq!(second, "1");
    }

    #[test]
    fn when_segment_kind_conflicts_it_should_return_duplicate_key() {
        // Arrange
        let mut guard = acquire_pattern_state();
        let path = make_segments(&["items"]);
        guard.resolve(&path, "0", "items").expect("numeric");

        // Act
        let error = guard
            .resolve(&path, "field", "items")
            .expect_err("conflict");

        // Assert
        match error {
            ParseError::DuplicateKey { key } => assert_eq!(key, "items"),
            _ => panic!("expected duplicate key"),
        }
    }
}

mod container_type {
    use super::*;

    #[test]
    fn when_numeric_segments_seen_it_should_report_array() {
        // Arrange
        let mut guard = acquire_pattern_state();
        let path = make_segments(&["items"]);
        guard.resolve(&path, "", "items").expect("numeric");

        // Act
        let container = guard.container_type(&["items"]);

        // Assert
        assert_eq!(container, Some(ContainerType::Array));
    }

    #[test]
    fn when_string_segments_seen_it_should_report_object() {
        // Arrange
        let mut guard = acquire_pattern_state();
        let path = make_segments(&["props"]);
        guard.resolve(&path, "field", "props").expect("string");

        // Act
        let container = guard.container_type(&["props"]);

        // Assert
        assert_eq!(container, Some(ContainerType::Object));
    }
}

mod child_capacity {
    use super::*;

    #[test]
    fn when_resolving_multiple_children_it_should_track_count() {
        // Arrange
        let mut guard = acquire_pattern_state();
        let path = make_segments(&["items"]);
        for _ in 0..3 {
            guard.resolve(&path, "", "items").unwrap();
        }

        // Act
        let capacity = guard.child_capacity(&["items"]);

        // Assert
        assert_eq!(capacity, 3);
    }
}

mod acquire_pattern_state_mod {
    use super::*;

    #[test]
    fn when_guard_is_returned_to_pool_it_should_reset_state() {
        // Arrange
        {
            let mut guard = acquire_pattern_state();
            guard.resolve(&[], "foo", "root").expect("prime pool");
        }

        // Act
        let guard = acquire_pattern_state();

        // Assert
        assert!(guard.container_type(&["foo"]).is_none());
        assert_eq!(guard.child_capacity(&[]), 0);
    }
}
