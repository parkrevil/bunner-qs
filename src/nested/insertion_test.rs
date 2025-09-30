use super::{insert_nested_value_arena, resolve_segments};
use crate::ParseError;
use crate::nested::pattern_state::{PatternStateGuard, acquire_pattern_state};
use crate::parsing::arena::{ArenaQueryMap, ArenaValue, ParseArena};

fn map_with_capacity<'arena>(arena: &'arena ParseArena, capacity: usize) -> ArenaQueryMap<'arena> {
    ArenaQueryMap::with_capacity(arena, capacity)
}

fn insert_value<'arena>(
    arena: &'arena ParseArena,
    map: &mut ArenaQueryMap<'arena>,
    path: &[&str],
    value: &str,
    state: &mut PatternStateGuard,
) -> Result<(), ParseError> {
    insert_nested_value_arena(arena, map, path, arena.alloc_str(value), state)
}

fn insert_sequence_values<'arena>(
    arena: &'arena ParseArena,
    map: &mut ArenaQueryMap<'arena>,
    path: &[&str],
    values: &[&str],
    state: &mut PatternStateGuard,
) {
    for value in values {
        insert_value(arena, map, path, value, state).expect("sequence insert should succeed");
    }
}

fn assert_single_string_entry<'arena>(map: &ArenaQueryMap<'arena>, key: &str, expected: &str) {
    let entries = map.entries_slice();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].0, key);
    match &entries[0].1 {
        ArenaValue::String(value) => assert_eq!(*value, expected),
        _ => panic!("expected string value"),
    }
}

fn assert_sequence_of_maps<'arena>(
    map: &ArenaQueryMap<'arena>,
    key: &str,
    field: &str,
    expected_values: &[&str],
) {
    let entries = map.entries_slice();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].0, key);
    let sequence = match &entries[0].1 {
        ArenaValue::Seq(items) => items,
        _ => panic!("expected sequence container"),
    };
    assert_eq!(sequence.len(), expected_values.len());
    for (item, expected) in sequence.iter().zip(expected_values.iter()) {
        match item {
            ArenaValue::Map { entries, .. } => {
                assert_eq!(entries.len(), 1);
                let (entry_key, value) = &entries[0];
                assert_eq!(*entry_key, field);
                match value {
                    ArenaValue::String(text) => assert_eq!(*text, *expected),
                    _ => panic!("expected string leaf"),
                }
            }
            _ => panic!("expected map entry in sequence"),
        }
    }
}

fn expect_duplicate_key(error: ParseError, expected: &str) {
    match error {
        ParseError::DuplicateKey { key } => assert_eq!(key, expected),
        other => panic!("expected duplicate key error, got {other:?}"),
    }
}

mod insert_nested_value_arena {
    use super::*;

    #[test]
    fn when_scalar_is_inserted_at_root_it_should_store_string_value() {
        // Arrange
        let arena = ParseArena::new();
        let mut map = map_with_capacity(&arena, 0);
        let mut state = acquire_pattern_state();

        // Act
        insert_value(&arena, &mut map, &["token"], "abc123", &mut state)
            .expect("root insertion should succeed");

        // Assert
        assert_single_string_entry(&map, "token", "abc123");
    }

    #[test]
    fn when_array_pattern_is_used_it_should_expand_sequence_of_maps() {
        // Arrange
        let arena = ParseArena::new();
        let mut map = map_with_capacity(&arena, 0);
        let mut state = acquire_pattern_state();
        let path = ["items", "", "name"];

        // Act
        insert_sequence_values(&arena, &mut map, &path, &["alice", "bob"], &mut state);

        // Assert
        assert_sequence_of_maps(&map, "items", "name", &["alice", "bob"]);
    }

    #[test]
    fn when_duplicate_scalar_is_inserted_it_should_return_duplicate_key_error() {
        // Arrange
        let arena = ParseArena::new();
        let mut map = map_with_capacity(&arena, 0);
        let mut state = acquire_pattern_state();
        insert_value(&arena, &mut map, &["token"], "first", &mut state).expect("initial insert");

        // Act
        let error = insert_value(&arena, &mut map, &["token"], "second", &mut state)
            .expect_err("duplicate insert should fail");

        // Assert
        expect_duplicate_key(error, "token");
    }
}

mod resolve_segments {
    use super::*;

    #[test]
    fn when_array_segment_repeats_it_should_increment_indices() {
        // Arrange
        let mut state = acquire_pattern_state();
        let path = ["items", "", "name"];

        // Act
        let first = resolve_segments(&mut state, &path).expect("first resolve");
        let second = resolve_segments(&mut state, &path).expect("second resolve");

        // Assert
        assert_eq!(first[1].as_str(), "0");
        assert_eq!(second[1].as_str(), "1");
    }

    #[test]
    fn when_path_has_only_literals_it_should_return_original_segments() {
        // Arrange
        let mut state = acquire_pattern_state();
        let path = ["profile", "name"];

        // Act
        let resolved = resolve_segments(&mut state, &path).expect("resolve");

        // Assert
        assert_eq!(resolved.len(), path.len());
        assert_eq!(resolved[0].as_str(), "profile");
        assert_eq!(resolved[1].as_str(), "name");
    }
}
