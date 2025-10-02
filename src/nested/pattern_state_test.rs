use super::{ContainerType, PatternStateGuard, ResolvedSegment, acquire_pattern_state};
use crate::parsing_helpers::expect_duplicate_key;
use std::borrow::Cow;

fn make_segments<'a>(parts: &'a [&'a str]) -> Vec<ResolvedSegment<'a>> {
    parts
        .iter()
        .map(|segment| ResolvedSegment::new(Cow::Borrowed(*segment)))
        .collect()
}

fn resolve_numeric<'a>(
    guard: &mut PatternStateGuard,
    path: &[ResolvedSegment<'a>],
    alias: &str,
) -> Cow<'a, str> {
    guard
        .resolve(path, "", alias)
        .expect("numeric segment should resolve")
}

mod resolve {
    use super::*;

    #[test]
    fn should_increment_indices_when_numeric_segment_repeats_then_return_sequential_indices() {
        // Arrange
        let mut guard = acquire_pattern_state();
        let path = make_segments(&["items"]);

        // Act
        let first = resolve_numeric(&mut guard, &path, "items");
        let second = resolve_numeric(&mut guard, &path, "items");

        // Assert
        assert_eq!(first, "0");
        assert_eq!(second, "1");
    }

    #[test]
    fn should_report_duplicate_key_when_segment_kind_conflicts_then_return_duplicate_key_error() {
        // Arrange
        let mut guard = acquire_pattern_state();
        let path = make_segments(&["items"]);
        resolve_numeric(&mut guard, &path, "items");

        // Act
        let error = guard
            .resolve(&path, "field", "items")
            .expect_err("conflict");

        // Assert
        expect_duplicate_key(error, "items");
    }
}

mod container_type {
    use super::*;

    #[test]
    fn should_return_array_when_numeric_segments_are_seen_then_select_array_container() {
        // Arrange
        let mut guard = acquire_pattern_state();
        let path = make_segments(&["items"]);
        resolve_numeric(&mut guard, &path, "items");

        // Act
        let container = guard.container_type(&["items"]);

        // Assert
        assert_eq!(container, Some(ContainerType::Array));
    }

    #[test]
    fn should_return_object_when_string_segments_are_seen_then_select_object_container() {
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
    fn should_track_child_count_when_resolving_multiple_children_then_return_child_capacity() {
        // Arrange
        let mut guard = acquire_pattern_state();
        let path = make_segments(&["items"]);
        for _ in 0..3 {
            resolve_numeric(&mut guard, &path, "items");
        }

        // Act
        let capacity = guard.child_capacity(&["items"]);

        // Assert
        assert_eq!(capacity, 3);
    }
}

mod acquire_pattern_state {
    use super::*;

    #[test]
    fn should_reset_state_when_guard_is_returned_to_pool_then_clear_cached_metadata() {
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

    #[test]
    fn should_reuse_free_nodes_after_reset_then_restart_numeric_indices() {
        // Arrange
        {
            let mut guard = acquire_pattern_state();
            let path = make_segments(&["items"]);
            resolve_numeric(&mut guard, &path, "items");
        }

        // Act
        let mut guard = acquire_pattern_state();
        let path = make_segments(&["items"]);
        let first = resolve_numeric(&mut guard, &path, "items");
        let second = resolve_numeric(&mut guard, &path, "items");

        // Assert
        assert_eq!(first, "0");
        assert_eq!(second, "1");
        assert_eq!(guard.child_capacity(&["items"]), 2);
    }
}
