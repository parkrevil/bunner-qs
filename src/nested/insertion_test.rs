use super::{insert_nested_value_arena, resolve_segments};
use crate::ParseError;
use crate::nested::pattern_state::acquire_pattern_state;
use crate::parsing::arena::{ArenaQueryMap, ArenaValue, ParseArena};

fn map_with_capacity<'arena>(arena: &'arena ParseArena, capacity: usize) -> ArenaQueryMap<'arena> {
    ArenaQueryMap::with_capacity(arena, capacity)
}

#[test]
fn insert_nested_value_inserts_scalar_at_root() {
    let arena = ParseArena::new();
    let mut map = map_with_capacity(&arena, 0);
    let mut state = acquire_pattern_state();

    insert_nested_value_arena(
        &arena,
        &mut map,
        &["token"],
        arena.alloc_str("abc123"),
        &mut state,
    )
    .expect("root insertion should succeed");

    let entries = map.entries_slice();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].0, "token");
    match &entries[0].1 {
        ArenaValue::String(value) => assert_eq!(*value, "abc123"),
        _ => panic!("expected string value at root"),
    }
}

#[test]
fn insert_nested_value_builds_sequence_of_maps_for_array_pattern() {
    let arena = ParseArena::new();
    let mut map = map_with_capacity(&arena, 0);
    let mut state = acquire_pattern_state();

    insert_nested_value_arena(
        &arena,
        &mut map,
        &["items", "", "name"],
        arena.alloc_str("alice"),
        &mut state,
    )
    .expect("first array element should insert");

    insert_nested_value_arena(
        &arena,
        &mut map,
        &["items", "", "name"],
        arena.alloc_str("bob"),
        &mut state,
    )
    .expect("second array element should insert at next index");

    let entries = map.entries_slice();
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].0, "items");
    match &entries[0].1 {
        ArenaValue::Seq(items) => {
            assert_eq!(items.len(), 2);
            for (idx, expected) in ["alice", "bob"].iter().enumerate() {
                match &items[idx] {
                    ArenaValue::Map { entries, .. } => {
                        assert_eq!(entries.len(), 1);
                        let (key, value) = &entries[0];
                        assert_eq!(*key, "name");
                        match value {
                            ArenaValue::String(text) => assert_eq!(*text, *expected),
                            _ => panic!("expected string leaf"),
                        }
                    }
                    _ => panic!("expected map entry in sequence"),
                }
            }
        }
        _ => panic!("expected sequence container"),
    }
}

#[test]
fn insert_nested_value_rejects_duplicate_scalar_entries() {
    let arena = ParseArena::new();
    let mut map = map_with_capacity(&arena, 0);
    let mut state = acquire_pattern_state();

    insert_nested_value_arena(
        &arena,
        &mut map,
        &["token"],
        arena.alloc_str("first"),
        &mut state,
    )
    .expect("initial insertion should succeed");

    let error = insert_nested_value_arena(
        &arena,
        &mut map,
        &["token"],
        arena.alloc_str("second"),
        &mut state,
    )
    .expect_err("duplicate insert should fail");

    match error {
        ParseError::DuplicateKey { key } => assert_eq!(key, "token"),
        other => panic!("expected duplicate key error, got {other:?}"),
    }
}

#[test]
fn resolve_segments_generates_sequential_indices_for_arrays() {
    let mut state = acquire_pattern_state();

    let first = resolve_segments(&mut state, &["items", "", "name"]).expect("first resolve");
    assert_eq!(first.len(), 3);
    assert_eq!(first[0].as_str(), "items");
    assert_eq!(first[1].as_str(), "0");
    assert_eq!(first[2].as_str(), "name");

    let second = resolve_segments(&mut state, &["items", "", "name"]).expect("second resolve");
    assert_eq!(second[1].as_str(), "1");
}

#[test]
fn resolve_segments_returns_original_for_simple_path() {
    let mut state = acquire_pattern_state();

    let resolved = resolve_segments(&mut state, &["profile", "name"]).expect("resolve");
    assert_eq!(resolved.len(), 2);
    assert_eq!(resolved[0].as_str(), "profile");
    assert_eq!(resolved[1].as_str(), "name");
}
