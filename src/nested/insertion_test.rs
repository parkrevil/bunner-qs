use super::{
    ArenaSetContext, arena_set_nested_value, get_root_value, handle_map_segment,
    handle_seq_segment, insert_nested_value_arena, resolve_segments, try_insert_or_duplicate,
    with_string_promotion_suppressed,
};
use crate::DuplicateKeyBehavior;
use crate::ParseError;
use crate::nested::pattern_state::{PatternStateGuard, acquire_pattern_state};
use crate::nested::segment::{ContainerType, ResolvedSegment};
use crate::parsing::arena::{ArenaQueryMap, ArenaValue, ParseArena};
use crate::parsing_helpers::expect_duplicate_key;
use ahash::RandomState;
use hashbrown::HashMap;
use smallvec::SmallVec;
use std::borrow::Cow;

fn map_with_capacity<'arena>(arena: &'arena ParseArena, capacity: usize) -> ArenaQueryMap<'arena> {
    ArenaQueryMap::with_capacity(arena, capacity)
}

fn insert_value<'arena>(
    arena: &'arena ParseArena,
    map: &mut ArenaQueryMap<'arena>,
    path: &[&str],
    value: &str,
    state: &mut PatternStateGuard,
    duplicate_keys: DuplicateKeyBehavior,
) -> Result<(), ParseError> {
    insert_nested_value_arena(
        arena,
        map,
        path,
        arena.alloc_str(value),
        state,
        duplicate_keys,
    )
}

fn insert_sequence_values<'arena>(
    arena: &'arena ParseArena,
    map: &mut ArenaQueryMap<'arena>,
    path: &[&str],
    values: &[&str],
    state: &mut PatternStateGuard,
    duplicate_keys: DuplicateKeyBehavior,
) {
    for value in values {
        insert_value(arena, map, path, value, state, duplicate_keys)
            .expect("sequence insert should succeed");
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

mod insert_nested_value_arena {
    use super::*;

    #[test]
    fn should_noop_when_path_is_empty_then_leave_map_unchanged() {
        // Arrange
        let arena = ParseArena::new();
        let mut map = map_with_capacity(&arena, 0);
        let mut state = acquire_pattern_state();

        // Act
        insert_nested_value_arena(
            &arena,
            &mut map,
            &[],
            arena.alloc_str("ignored"),
            &mut state,
            DuplicateKeyBehavior::Reject,
        )
        .expect("empty path should be ignored");

        // Assert
        assert!(map.is_empty());
    }

    #[test]
    fn should_return_duplicate_key_when_nested_path_conflicts_with_scalar_then_keep_scalar() {
        // Arrange
        let arena = ParseArena::new();
        let mut map = map_with_capacity(&arena, 0);
        let mut state = acquire_pattern_state();
        insert_value(
            &arena,
            &mut map,
            &["profile"],
            "raw",
            &mut state,
            DuplicateKeyBehavior::LastWins,
        )
        .expect("initial scalar insert");

        // Act
        let error = insert_value(
            &arena,
            &mut map,
            &["profile", "name"],
            "neo",
            &mut state,
            DuplicateKeyBehavior::LastWins,
        )
        .expect_err("scalar conflict should be rejected");

        // Assert
        expect_duplicate_key(error, "profile");
        assert_single_string_entry(&map, "profile", "raw");
    }

    #[test]
    fn should_overwrite_placeholder_sequence_entry_when_existing_value_is_empty() {
        // Arrange
        let arena = ParseArena::new();
        let mut map = map_with_capacity(&arena, 0);
        let mut state = acquire_pattern_state();
        insert_value(
            &arena,
            &mut map,
            &["items", "0"],
            "",
            &mut state,
            DuplicateKeyBehavior::Reject,
        )
        .expect("placeholder insert");

        // Act
        insert_value(
            &arena,
            &mut map,
            &["items", "0"],
            "actual",
            &mut state,
            DuplicateKeyBehavior::Reject,
        )
        .expect("placeholder should accept concrete value");

        // Assert
        let entries = map.entries_slice();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, "items");
        let sequence = match &entries[0].1 {
            ArenaValue::Seq(items) => items,
            _ => panic!("expected sequence"),
        };
        assert_eq!(sequence.len(), 1);
        match &sequence[0] {
            ArenaValue::String(text) => assert_eq!(*text, "actual"),
            _ => panic!("expected string entry"),
        }
    }

    #[test]
    fn should_return_duplicate_key_when_nested_map_rejects_duplicate_field() {
        // Arrange
        let arena = ParseArena::new();
        let mut map = map_with_capacity(&arena, 0);
        let mut state = acquire_pattern_state();
        insert_value(
            &arena,
            &mut map,
            &["user", "name"],
            "alice",
            &mut state,
            DuplicateKeyBehavior::Reject,
        )
        .expect("initial nested insert");

        // Act
        let error = insert_value(
            &arena,
            &mut map,
            &["user", "name"],
            "bob",
            &mut state,
            DuplicateKeyBehavior::Reject,
        )
        .expect_err("duplicate nested value should fail");

        // Assert
        expect_duplicate_key(error, "name");
    }

    #[test]
    fn should_replace_nested_map_value_when_last_wins_enabled_then_overwrite_value() {
        // Arrange
        let arena = ParseArena::new();
        let mut map = map_with_capacity(&arena, 0);
        let mut state = acquire_pattern_state();
        insert_value(
            &arena,
            &mut map,
            &["user", "name"],
            "alice",
            &mut state,
            DuplicateKeyBehavior::LastWins,
        )
        .expect("initial nested insert");

        // Act
        insert_value(
            &arena,
            &mut map,
            &["user", "name"],
            "bob",
            &mut state,
            DuplicateKeyBehavior::LastWins,
        )
        .expect("last wins should overwrite nested map value");

        // Assert
        let entries = map.entries_slice();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, "user");
        let nested = match &entries[0].1 {
            ArenaValue::Map { entries, .. } => entries,
            _ => panic!("expected nested map"),
        };
        assert_eq!(nested.len(), 1);
        assert_eq!(nested[0].0, "name");
        match &nested[0].1 {
            ArenaValue::String(text) => assert_eq!(*text, "bob"),
            _ => panic!("expected string value"),
        }
    }

    #[test]
    fn should_keep_nested_map_value_when_first_wins_then_preserve_existing_value() {
        // Arrange
        let arena = ParseArena::new();
        let mut map = map_with_capacity(&arena, 0);
        let mut state = acquire_pattern_state();
        insert_value(
            &arena,
            &mut map,
            &["user", "email"],
            "primary@example.com",
            &mut state,
            DuplicateKeyBehavior::FirstWins,
        )
        .expect("initial nested insert");

        // Act
        insert_value(
            &arena,
            &mut map,
            &["user", "email"],
            "secondary@example.com",
            &mut state,
            DuplicateKeyBehavior::FirstWins,
        )
        .expect("first wins should preserve nested map value");

        // Assert
        let entries = map.entries_slice();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, "user");
        let nested = match &entries[0].1 {
            ArenaValue::Map { entries, .. } => entries,
            _ => panic!("expected nested map"),
        };
        assert_eq!(nested.len(), 1);
        assert_eq!(nested[0].0, "email");
        match &nested[0].1 {
            ArenaValue::String(text) => assert_eq!(*text, "primary@example.com"),
            _ => panic!("expected string value"),
        }
    }

    #[test]
    fn should_return_duplicate_key_when_sequence_index_skips_existing_length() {
        // Arrange
        let arena = ParseArena::new();
        let mut map = map_with_capacity(&arena, 0);
        let mut state = acquire_pattern_state();

        // Act
        let error = insert_value(
            &arena,
            &mut map,
            &["items", "1"],
            "late",
            &mut state,
            DuplicateKeyBehavior::Reject,
        )
        .expect_err("sparse sequence index should be rejected");

        // Assert
        expect_duplicate_key(error, "items");
    }

    #[test]
    fn should_replace_sequence_value_when_last_wins_enabled_then_overwrite_entry() {
        // Arrange
        let arena = ParseArena::new();
        let mut map = map_with_capacity(&arena, 0);
        let mut state = acquire_pattern_state();
        insert_value(
            &arena,
            &mut map,
            &["items", "0"],
            "first",
            &mut state,
            DuplicateKeyBehavior::LastWins,
        )
        .expect("initial sequence insert");

        // Act
        insert_value(
            &arena,
            &mut map,
            &["items", "0"],
            "second",
            &mut state,
            DuplicateKeyBehavior::LastWins,
        )
        .expect("last wins should overwrite sequence value");

        // Assert
        let entries = map.entries_slice();
        assert_eq!(entries.len(), 1);
        let sequence = match &entries[0].1 {
            ArenaValue::Seq(items) => items,
            _ => panic!("expected sequence"),
        };
        match &sequence[0] {
            ArenaValue::String(text) => assert_eq!(*text, "second"),
            _ => panic!("expected string entry"),
        }
    }

    #[test]
    fn should_keep_sequence_value_when_first_wins_then_preserve_existing_entry() {
        // Arrange
        let arena = ParseArena::new();
        let mut map = map_with_capacity(&arena, 0);
        let mut state = acquire_pattern_state();
        insert_value(
            &arena,
            &mut map,
            &["items", "0"],
            "initial",
            &mut state,
            DuplicateKeyBehavior::FirstWins,
        )
        .expect("initial sequence insert");

        // Act
        insert_value(
            &arena,
            &mut map,
            &["items", "0"],
            "ignored",
            &mut state,
            DuplicateKeyBehavior::FirstWins,
        )
        .expect("first wins should ignore duplicates");

        // Assert
        let entries = map.entries_slice();
        assert_eq!(entries.len(), 1);
        let sequence = match &entries[0].1 {
            ArenaValue::Seq(items) => items,
            _ => panic!("expected sequence"),
        };
        match &sequence[0] {
            ArenaValue::String(text) => assert_eq!(*text, "initial"),
            _ => panic!("expected string entry"),
        }
    }

    #[test]
    fn should_store_string_value_when_root_scalar_insert_occurs_then_store_value_in_map() {
        // Arrange
        let arena = ParseArena::new();
        let mut map = map_with_capacity(&arena, 0);
        let mut state = acquire_pattern_state();

        // Act
        insert_value(
            &arena,
            &mut map,
            &["token"],
            "abc123",
            &mut state,
            DuplicateKeyBehavior::Reject,
        )
        .expect("root insertion should succeed");

        // Assert
        assert_single_string_entry(&map, "token", "abc123");
    }

    #[test]
    fn should_convert_duplicate_error_when_root_insertion_fails_via_helper() {
        // Act
        let error = try_insert_or_duplicate("token", || Err(()))
            .expect_err("helper should convert duplicate into ParseError");

        // Assert
        expect_duplicate_key(error, "token");
    }

    #[test]
    fn should_return_ok_when_helper_insert_succeeds() {
        // Act
        try_insert_or_duplicate("token", || Ok(()))
            .expect("helper should propagate successful insertion");
    }

    #[test]
    fn should_expand_sequence_of_maps_when_array_pattern_is_used_then_create_sequence_entries() {
        // Arrange
        let arena = ParseArena::new();
        let mut map = map_with_capacity(&arena, 0);
        let mut state = acquire_pattern_state();
        let path = ["items", "", "name"];

        // Act
        insert_sequence_values(
            &arena,
            &mut map,
            &path,
            &["alice", "bob"],
            &mut state,
            DuplicateKeyBehavior::Reject,
        );

        // Assert
        assert_sequence_of_maps(&map, "items", "name", &["alice", "bob"]);
    }

    #[test]
    fn should_return_duplicate_key_when_sequence_segment_is_non_numeric_then_reject_insertion() {
        // Arrange
        let arena = ParseArena::new();
        let mut map = map_with_capacity(&arena, 0);
        let mut state = acquire_pattern_state();
        insert_value(
            &arena,
            &mut map,
            &["items", "", "name"],
            "alice",
            &mut state,
            DuplicateKeyBehavior::Reject,
        )
        .expect("initial array-backed insertion");

        // Act
        let error = insert_value(
            &arena,
            &mut map,
            &["items", "name"],
            "override",
            &mut state,
            DuplicateKeyBehavior::Reject,
        )
        .expect_err("non-numeric segment should fail");

        // Assert
        expect_duplicate_key(error, "items");
    }

    #[test]
    fn should_return_duplicate_key_when_scalar_repeats_then_return_duplicate_key_error() {
        // Arrange
        let arena = ParseArena::new();
        let mut map = map_with_capacity(&arena, 0);
        let mut state = acquire_pattern_state();
        insert_value(
            &arena,
            &mut map,
            &["token"],
            "first",
            &mut state,
            DuplicateKeyBehavior::Reject,
        )
        .expect("initial insert");

        // Act
        let error = insert_value(
            &arena,
            &mut map,
            &["token"],
            "second",
            &mut state,
            DuplicateKeyBehavior::Reject,
        )
        .expect_err("duplicate insert should fail");

        // Assert
        expect_duplicate_key(error, "token");
    }

    #[test]
    fn should_return_duplicate_key_when_root_value_missing_in_helper() {
        // Arrange
        let arena = ParseArena::new();
        let mut map = map_with_capacity(&arena, 0);

        // Act
        let error = match get_root_value(&mut map, "profile", "profile") {
            Ok(_) => panic!("expected duplicate error"),
            Err(err) => err,
        };

        // Assert
        expect_duplicate_key(error, "profile");
    }

    #[test]
    fn should_keep_initial_value_when_scalar_repeats_and_first_wins_then_preserve_first_value() {
        // Arrange
        let arena = ParseArena::new();
        let mut map = map_with_capacity(&arena, 0);
        let mut state = acquire_pattern_state();
        insert_value(
            &arena,
            &mut map,
            &["token"],
            "first",
            &mut state,
            DuplicateKeyBehavior::FirstWins,
        )
        .expect("initial insert");

        // Act
        insert_value(
            &arena,
            &mut map,
            &["token"],
            "second",
            &mut state,
            DuplicateKeyBehavior::FirstWins,
        )
        .expect("duplicate insert should be ignored");

        // Assert
        assert_single_string_entry(&map, "token", "first");
    }

    #[test]
    fn should_replace_with_latest_value_when_scalar_repeats_and_last_wins_then_store_latest_value()
    {
        // Arrange
        let arena = ParseArena::new();
        let mut map = map_with_capacity(&arena, 0);
        let mut state = acquire_pattern_state();
        insert_value(
            &arena,
            &mut map,
            &["token"],
            "first",
            &mut state,
            DuplicateKeyBehavior::LastWins,
        )
        .expect("initial insert");

        // Act
        insert_value(
            &arena,
            &mut map,
            &["token"],
            "second",
            &mut state,
            DuplicateKeyBehavior::LastWins,
        )
        .expect("duplicate insert should overwrite");

        // Assert
        assert_single_string_entry(&map, "token", "second");
    }

    #[test]
    fn should_return_ok_when_depth_exceeds_segment_length_then_exit_early() {
        // Arrange
        let arena = ParseArena::new();
        let state = acquire_pattern_state();
        let segments = [ResolvedSegment::new(Cow::Borrowed("root"))];
        let mut current = ArenaValue::map(&arena);
        let ctx = ArenaSetContext {
            arena: &arena,
            state: &state,
            root_key: "root",
            duplicate_keys: DuplicateKeyBehavior::LastWins,
        };

        // Act
        arena_set_nested_value(
            &ctx,
            &mut current,
            &segments,
            segments.len(),
            arena.alloc_str("ignored"),
        )
        .expect("depth exhaustion should succeed");

        // Assert
        match current {
            ArenaValue::Map { entries, .. } => assert!(entries.is_empty()),
            _ => panic!("expected map container"),
        }
    }

    #[test]
    fn should_promote_string_node_to_map_when_state_has_no_hint_then_create_child_entry() {
        // Arrange
        let arena = ParseArena::new();
        let state = acquire_pattern_state();
        let segments = [
            ResolvedSegment::new(Cow::Borrowed("root")),
            ResolvedSegment::new(Cow::Borrowed("child")),
        ];
        let placeholder = arena.alloc_str("");
        let mut current = ArenaValue::string(placeholder);
        let ctx = ArenaSetContext {
            arena: &arena,
            state: &state,
            root_key: "root",
            duplicate_keys: DuplicateKeyBehavior::LastWins,
        };

        // Act
        arena_set_nested_value(&ctx, &mut current, &segments, 1, arena.alloc_str("leaf"))
            .expect("string node should promote to map");

        // Assert
        match current {
            ArenaValue::Map { entries, .. } => {
                assert_eq!(entries.len(), 1);
                assert_eq!(entries[0].0, "child");
                match &entries[0].1 {
                    ArenaValue::String(text) => assert_eq!(*text, "leaf"),
                    _ => panic!("expected string leaf"),
                }
            }
            _ => panic!("expected promoted map"),
        }
    }

    #[test]
    fn should_use_container_hint_to_convert_sequence_into_object_then_store_nested_value() {
        // Arrange
        let arena = ParseArena::new();
        let mut state = acquire_pattern_state();
        let resolved = resolve_segments(&mut state, &["items", "name"]).expect("resolve");
        assert_eq!(
            state.container_type(&["items"]),
            Some(ContainerType::Object)
        );
        let mut current = ArenaValue::seq_with_capacity(&arena, 0);
        let ctx = ArenaSetContext {
            arena: &arena,
            state: &state,
            root_key: "items",
            duplicate_keys: DuplicateKeyBehavior::LastWins,
        };

        // Act
        arena_set_nested_value(&ctx, &mut current, &resolved, 1, arena.alloc_str("value"))
            .expect("state hint should convert to map container");

        // Assert
        match current {
            ArenaValue::Map { entries, .. } => {
                assert_eq!(entries.len(), 1);
                assert_eq!(entries[0].0, "name");
                match &entries[0].1 {
                    ArenaValue::String(text) => assert_eq!(*text, "value"),
                    _ => panic!("expected string leaf"),
                }
            }
            _ => panic!("expected map container after state hint conversion"),
        }
    }

    #[test]
    fn should_return_duplicate_key_when_string_node_cannot_promote_with_override() {
        // Arrange
        let arena = ParseArena::new();
        let mut state = acquire_pattern_state();
        let resolved = resolve_segments(&mut state, &["root", "child"]).expect("resolve");
        let mut current = ArenaValue::string(arena.alloc_str("leaf"));
        let ctx = ArenaSetContext {
            arena: &arena,
            state: &state,
            root_key: "root",
            duplicate_keys: DuplicateKeyBehavior::Reject,
        };

        // Act
        let error = with_string_promotion_suppressed(|| {
            arena_set_nested_value(&ctx, &mut current, &resolved, 1, arena.alloc_str("value"))
        })
        .expect_err("suppressed promotion should surface duplicate key error");

        // Assert
        expect_duplicate_key(error, "root");
    }
}

mod resolve_segments {
    use super::*;

    #[test]
    fn should_increment_indices_when_array_segment_repeats_then_update_sequence_indices() {
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
    fn should_return_original_segments_when_path_is_literal_then_preserve_path_segments() {
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

    #[test]
    fn should_return_single_segment_when_path_contains_only_root_segment() {
        // Arrange
        let mut state = acquire_pattern_state();
        let path = ["token"];

        // Act
        let resolved = resolve_segments(&mut state, &path).expect("resolve");

        // Assert
        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].as_str(), "token");
    }
}

mod helper_segments {
    use super::*;

    fn make_ctx<'arena, 'pattern>(
        arena: &'arena ParseArena,
        state: &'pattern PatternStateGuard,
        root: &'pattern str,
        duplicate_keys: DuplicateKeyBehavior,
    ) -> ArenaSetContext<'arena, 'pattern> {
        ArenaSetContext {
            arena,
            state,
            root_key: root,
            duplicate_keys,
        }
    }

    #[test]
    fn should_error_when_map_segment_missing_value_in_vacant_branch() {
        // Arrange
        let arena = ParseArena::new();
        let state = acquire_pattern_state();
        let ctx = make_ctx(&arena, &state, "root", DuplicateKeyBehavior::LastWins);
        let mut entries = arena.alloc_vec();
        let mut index = HashMap::<&str, usize, RandomState>::with_capacity_and_hasher(
            0,
            RandomState::default(),
        );
        let segments = [ResolvedSegment::new(Cow::Borrowed("root"))];
        let mut path: SmallVec<[&str; 16]> = SmallVec::new();
        let mut missing: Option<&str> = None;

        // Act
        let result = handle_map_segment(
            &ctx,
            &mut entries,
            &mut index,
            &segments,
            &mut path,
            0,
            "child",
            true,
            &mut missing,
        );

        // Assert
        match result {
            Ok(_) => panic!("expected duplicate key error"),
            Err(err) => expect_duplicate_key(err, "child"),
        }
    }

    #[test]
    fn should_error_when_map_segment_updates_without_value_then_signal_duplicate() {
        // Arrange
        let arena = ParseArena::new();
        let state = acquire_pattern_state();
        let ctx = make_ctx(&arena, &state, "root", DuplicateKeyBehavior::LastWins);
        let mut entries = arena.alloc_vec();
        let mut index = HashMap::<&str, usize, RandomState>::with_capacity_and_hasher(
            0,
            RandomState::default(),
        );
        let segments = [ResolvedSegment::new(Cow::Borrowed("root"))];
        let mut path: SmallVec<[&str; 16]> = SmallVec::new();
        let mut initial = Some(arena.alloc_str("first"));
        handle_map_segment(
            &ctx,
            &mut entries,
            &mut index,
            &segments,
            &mut path,
            0,
            "child",
            true,
            &mut initial,
        )
        .expect("initial insert should succeed");

        let mut missing: Option<&str> = None;

        // Act
        let result = handle_map_segment(
            &ctx,
            &mut entries,
            &mut index,
            &segments,
            &mut SmallVec::<[&str; 16]>::new(),
            0,
            "child",
            true,
            &mut missing,
        );

        // Assert
        match result {
            Ok(_) => panic!("expected duplicate key error"),
            Err(err) => expect_duplicate_key(err, "child"),
        }
    }

    #[test]
    fn should_error_when_numeric_segment_overflows_then_report_duplicate_key() {
        // Arrange
        let arena = ParseArena::new();
        let state = acquire_pattern_state();
        let ctx = make_ctx(&arena, &state, "items", DuplicateKeyBehavior::LastWins);
        let mut items = arena.alloc_vec();
        let overflow = "184467440737095516160"; // larger than usize::MAX
        let segments = [ResolvedSegment::new(Cow::Borrowed(overflow))];
        let mut path: SmallVec<[&str; 16]> = SmallVec::new();
        let mut value_to_set = Some(arena.alloc_str("value"));

        // Act
        let result = handle_seq_segment(
            &ctx,
            &mut items,
            &segments,
            &mut path,
            0,
            overflow,
            true,
            &mut value_to_set,
        );

        // Assert
        match result {
            Ok(_) => panic!("expected duplicate key error"),
            Err(err) => expect_duplicate_key(err, "items"),
        }
    }

    #[test]
    fn should_error_when_last_wins_sequence_missing_value_then_signal_duplicate() {
        // Arrange
        let arena = ParseArena::new();
        let state = acquire_pattern_state();
        let ctx = make_ctx(&arena, &state, "items", DuplicateKeyBehavior::LastWins);
        let mut items = arena.alloc_vec();
        items.push(ArenaValue::string(arena.alloc_str("existing")));
        let segments = [ResolvedSegment::new(Cow::Borrowed("0"))];
        let mut path: SmallVec<[&str; 16]> = SmallVec::new();
        let mut missing: Option<&str> = None;

        // Act
        let result = handle_seq_segment(
            &ctx,
            &mut items,
            &segments,
            &mut path,
            0,
            "0",
            true,
            &mut missing,
        );

        // Assert
        match result {
            Ok(_) => panic!("expected duplicate key error"),
            Err(err) => expect_duplicate_key(err, "0"),
        }
    }

    #[test]
    fn should_error_when_sequence_segment_is_non_numeric_then_signal_duplicate_key() {
        // Arrange
        let arena = ParseArena::new();
        let state = acquire_pattern_state();
        let ctx = make_ctx(&arena, &state, "items", DuplicateKeyBehavior::Reject);
        let mut items = arena.alloc_vec();
        let segments = [ResolvedSegment::new(Cow::Borrowed("alpha"))];
        let mut path = SmallVec::<[&str; 16]>::new();
        let mut value_to_set = Some(arena.alloc_str("value"));

        // Act
        let error = match handle_seq_segment(
            &ctx,
            &mut items,
            &segments,
            &mut path,
            0,
            "alpha",
            true,
            &mut value_to_set,
        ) {
            Ok(_) => panic!("expected duplicate key error"),
            Err(err) => err,
        };

        // Assert
        expect_duplicate_key(error, "items");
    }

    #[test]
    fn should_error_when_sequence_append_missing_value_then_report_duplicate_key() {
        // Arrange
        let arena = ParseArena::new();
        let state = acquire_pattern_state();
        let ctx = make_ctx(&arena, &state, "items", DuplicateKeyBehavior::LastWins);
        let mut items = arena.alloc_vec();
        let segments = [ResolvedSegment::new(Cow::Borrowed("0"))];
        let mut path = SmallVec::<[&str; 16]>::new();
        let mut value_to_set: Option<&str> = None;

        // Act
        let error = match handle_seq_segment(
            &ctx,
            &mut items,
            &segments,
            &mut path,
            0,
            "0",
            true,
            &mut value_to_set,
        ) {
            Ok(_) => panic!("expected duplicate key error"),
            Err(err) => err,
        };

        // Assert
        expect_duplicate_key(error, "0");
    }

    #[test]
    fn should_error_when_sequence_placeholder_missing_value_then_report_duplicate_key() {
        // Arrange
        let arena = ParseArena::new();
        let state = acquire_pattern_state();
        let ctx = make_ctx(&arena, &state, "items", DuplicateKeyBehavior::LastWins);
        let mut items = arena.alloc_vec();
        items.push(ArenaValue::string(arena.alloc_str("")));
        let segments = [ResolvedSegment::new(Cow::Borrowed("0"))];
        let mut path = SmallVec::<[&str; 16]>::new();
        let mut value_to_set: Option<&str> = None;

        // Act
        let error = match handle_seq_segment(
            &ctx,
            &mut items,
            &segments,
            &mut path,
            0,
            "0",
            true,
            &mut value_to_set,
        ) {
            Ok(_) => panic!("expected duplicate key error"),
            Err(err) => err,
        };

        // Assert
        expect_duplicate_key(error, "0");
    }
}
