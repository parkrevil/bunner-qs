use super::{arena_ensure_container, arena_initial_container};
use crate::ParseError;
use crate::nested::segment::ContainerType;
use crate::parsing::arena::{ArenaValue, ParseArena};

#[test]
fn initial_container_returns_sequence_for_array_request() {
    let arena = ParseArena::new();

    let container = arena_initial_container(&arena, ContainerType::Array, 8);

    if let ArenaValue::Seq(items) = container {
        assert!(items.is_empty(), "sequence should start empty");
    } else {
        panic!("expected sequence container");
    }
}

#[test]
fn initial_container_returns_map_for_object_request() {
    let arena = ParseArena::new();

    let container = arena_initial_container(&arena, ContainerType::Object, 4);

    if let ArenaValue::Map { entries, .. } = container {
        assert!(entries.is_empty(), "map should start empty");
    } else {
        panic!("expected map container");
    }
}

#[test]
fn ensure_container_keeps_existing_sequence_when_expected_array() {
    let arena = ParseArena::new();
    let mut value = ArenaValue::seq_with_capacity(&arena, 0);

    if let ArenaValue::Seq(items) = &mut value {
        items.push(ArenaValue::string(arena.alloc_str("existing")));
    } else {
        panic!("value should start as sequence");
    }

    arena_ensure_container(&arena, &mut value, ContainerType::Array, "profile")
        .expect("existing sequence should satisfy expected array");

    match value {
        ArenaValue::Seq(items) => {
            assert_eq!(items.len(), 1, "existing items should be preserved");
            match &items[0] {
                ArenaValue::String(text) => assert_eq!(*text, "existing"),
                ArenaValue::Seq(_) => panic!("expected string item, found sequence"),
                ArenaValue::Map { .. } => panic!("expected string item, found map"),
            }
        }
        ArenaValue::Map { .. } => panic!("expected sequence after ensure"),
        ArenaValue::String(_) => panic!("sequence should not become string"),
    }
}

#[test]
fn ensure_container_converts_map_into_sequence_when_array_expected() {
    let arena = ParseArena::new();
    let mut value = ArenaValue::map(&arena);

    arena_ensure_container(&arena, &mut value, ContainerType::Array, "profile")
        .expect("map should be replaced by sequence");

    match value {
        ArenaValue::Seq(items) => assert!(items.is_empty(), "new sequence should be empty"),
        ArenaValue::Map { .. } => panic!("map should have been converted to sequence"),
        ArenaValue::String(_) => panic!("string should not appear after conversion"),
    }
}

#[test]
fn ensure_container_converts_sequence_into_map_when_object_expected() {
    let arena = ParseArena::new();
    let mut value = ArenaValue::seq_with_capacity(&arena, 0);

    arena_ensure_container(&arena, &mut value, ContainerType::Object, "profile")
        .expect("sequence should be replaced by map");

    match value {
        ArenaValue::Map { entries, .. } => assert!(entries.is_empty(), "new map should be empty"),
        ArenaValue::Seq(_) => panic!("sequence should have been converted to map"),
        ArenaValue::String(_) => panic!("string should not appear after conversion"),
    }
}

#[test]
fn ensure_container_returns_duplicate_key_for_string_when_array_expected() {
    let arena = ParseArena::new();
    let mut value = ArenaValue::string(arena.alloc_str("leaf"));

    let error = arena_ensure_container(&arena, &mut value, ContainerType::Array, "profile")
        .expect_err("string nodes should trigger duplicate key error");

    assert_duplicate_key(error, "profile");
}

#[test]
fn ensure_container_returns_duplicate_key_for_string_when_object_expected() {
    let arena = ParseArena::new();
    let mut value = ArenaValue::string(arena.alloc_str("leaf"));

    let error = arena_ensure_container(&arena, &mut value, ContainerType::Object, "settings")
        .expect_err("string nodes should trigger duplicate key error");

    assert_duplicate_key(error, "settings");
}

fn assert_duplicate_key(error: ParseError, expected_key: &str) {
    match error {
        ParseError::DuplicateKey { key } => assert_eq!(key, expected_key),
        other => panic!("expected duplicate key error, got {other:?}"),
    }
}
