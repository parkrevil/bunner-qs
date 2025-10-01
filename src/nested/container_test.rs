use super::{arena_ensure_container, arena_initial_container};
use crate::nested::segment::ContainerType;
use crate::parsing::arena::{ArenaValue, ParseArena};
use crate::parsing_helpers::{expect_duplicate_key, make_sequence, make_string};

fn assert_sequence_items(value: &ArenaValue<'_>, expected: &[&str]) {
    match value {
        ArenaValue::Seq(items) => {
            assert_eq!(items.len(), expected.len());
            for (item, expected_text) in items.iter().zip(expected.iter()) {
                match item {
                    ArenaValue::String(text) => assert_eq!(*text, *expected_text),
                    _ => panic!("expected string item"),
                }
            }
        }
        _ => panic!("expected sequence"),
    }
}

fn assert_empty_map(value: &ArenaValue<'_>) {
    match value {
        ArenaValue::Map { entries, .. } => assert!(entries.is_empty()),
        _ => panic!("expected map"),
    }
}

mod arena_initial_container {
    use super::*;

    #[test]
    fn should_initialize_empty_sequence_when_array_requested_then_return_empty_sequence_container()
    {
        // Arrange
        let arena = ParseArena::new();

        // Act
        let container = arena_initial_container(&arena, ContainerType::Array, 8);

        // Assert
        assert_sequence_items(&container, &[]);
    }

    #[test]
    fn should_initialize_empty_map_when_object_requested_then_return_empty_map_container() {
        // Arrange
        let arena = ParseArena::new();

        // Act
        let container = arena_initial_container(&arena, ContainerType::Object, 4);

        // Assert
        assert_empty_map(&container);
    }
}

mod arena_ensure_container {
    use super::*;

    #[test]
    fn should_reuse_sequence_when_array_expectation_matches_then_preserve_existing_items() {
        // Arrange
        let arena = ParseArena::new();
        let mut value = make_sequence(&arena, &["existing"]);

        // Act
        arena_ensure_container(&arena, &mut value, ContainerType::Array, "profile")
            .expect("sequence should satisfy expectation");

        // Assert
        assert_sequence_items(&value, &["existing"]);
    }

    #[test]
    fn should_convert_map_to_sequence_when_array_expected_then_return_empty_sequence() {
        // Arrange
        let arena = ParseArena::new();
        let mut value = ArenaValue::map(&arena);

        // Act
        arena_ensure_container(&arena, &mut value, ContainerType::Array, "profile")
            .expect("map should convert to sequence");

        // Assert
        assert_sequence_items(&value, &[]);
    }

    #[test]
    fn should_convert_sequence_to_map_when_object_expected_then_return_empty_map() {
        // Arrange
        let arena = ParseArena::new();
        let mut value = make_sequence(&arena, &[]);

        // Act
        arena_ensure_container(&arena, &mut value, ContainerType::Object, "profile")
            .expect("sequence should convert to map");

        // Assert
        assert_empty_map(&value);
    }

    #[test]
    fn should_report_error_when_string_conflicts_with_array_then_return_duplicate_key_error() {
        // Arrange
        let arena = ParseArena::new();
        let mut value = make_string(&arena, "leaf");

        // Act
        let error = arena_ensure_container(&arena, &mut value, ContainerType::Array, "profile")
            .expect_err("string should conflict with array expectation");

        // Assert
        expect_duplicate_key(error, "profile");
    }

    #[test]
    fn should_report_error_when_string_conflicts_with_object_then_return_duplicate_key_error() {
        // Arrange
        let arena = ParseArena::new();
        let mut value = make_string(&arena, "leaf");

        // Act
        let error = arena_ensure_container(&arena, &mut value, ContainerType::Object, "settings")
            .expect_err("string should conflict with object expectation");

        // Assert
        expect_duplicate_key(error, "settings");
    }
}
