use super::{arena_ensure_container, arena_initial_container};
use crate::ParseError;
use crate::nested::segment::ContainerType;
use crate::parsing::arena::{ArenaValue, ParseArena};

fn make_sequence<'arena>(arena: &'arena ParseArena, items: &[&str]) -> ArenaValue<'arena> {
    let mut value = ArenaValue::seq_with_capacity(arena, items.len());
    if let ArenaValue::Seq(entries) = &mut value {
        for item in items.iter() {
            entries.push(ArenaValue::string(arena.alloc_str(item)));
        }
    }
    value
}

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

fn make_string<'arena>(arena: &'arena ParseArena, text: &str) -> ArenaValue<'arena> {
    ArenaValue::string(arena.alloc_str(text))
}

mod arena_initial_container {
    use super::*;

    #[test]
    fn when_array_requested_it_should_start_with_empty_sequence() {
        // Arrange
        let arena = ParseArena::new();

        // Act
        let container = arena_initial_container(&arena, ContainerType::Array, 8);

        // Assert
        assert_sequence_items(&container, &[]);
    }

    #[test]
    fn when_object_requested_it_should_start_with_empty_map() {
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
    fn when_sequence_matches_array_expectation_it_should_be_reused() {
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
    fn when_array_expected_but_map_provided_it_should_convert_to_sequence() {
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
    fn when_object_expected_but_sequence_provided_it_should_convert_to_map() {
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
    fn when_string_conflicts_with_array_expectation_it_should_error() {
        // Arrange
        let arena = ParseArena::new();
        let mut value = make_string(&arena, "leaf");

        // Act
        let error = arena_ensure_container(&arena, &mut value, ContainerType::Array, "profile")
            .expect_err("string should conflict with array expectation");

        // Assert
        assert_duplicate_key(error, "profile");
    }

    #[test]
    fn when_string_conflicts_with_object_expectation_it_should_error() {
        // Arrange
        let arena = ParseArena::new();
        let mut value = make_string(&arena, "leaf");

        // Act
        let error = arena_ensure_container(&arena, &mut value, ContainerType::Object, "settings")
            .expect_err("string should conflict with object expectation");

        // Assert
        assert_duplicate_key(error, "settings");
    }

    fn assert_duplicate_key(error: ParseError, expected_key: &str) {
        match error {
            ParseError::DuplicateKey { key } => assert_eq!(key, expected_key),
            _ => panic!("expected duplicate key"),
        }
    }
}
