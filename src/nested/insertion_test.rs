use super::{
    ArenaSetContext, NodePreparation, TraversalStep, arena_build_nested_path, arena_is_placeholder,
    arena_set_nested_value, child_capacity_hint, get_root_value, handle_map_segment,
    handle_seq_segment, insert_nested_value_arena, prepare_current_node, resolve_segments,
    try_insert_or_duplicate, unexpected_nested_string, visit_map_node, visit_seq_node,
    with_string_promotion_suppressed,
};
use crate::DuplicateKeyBehavior;
use crate::ParseError;
use crate::arena_helpers::map_with_capacity;
use crate::nested::pattern_state::{PatternStateGuard, acquire_pattern_state};
use crate::nested::segment::{ContainerType, ResolvedSegment};
use crate::parsing::arena::{ArenaQueryMap, ArenaValue, ParseArena};
use crate::parsing_helpers::expect_duplicate_key;
use ahash::RandomState;
use assert_matches::assert_matches;
use hashbrown::HashMap;
use smallvec::SmallVec;
use std::borrow::Cow;

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
    assert_eq!(entries.len(), 1, "map should contain exactly one entry");
    let (entry_key, value) = &entries[0];
    assert_eq!(entry_key, &key, "entry key should match");
    assert_matches!(value, ArenaValue::String(text) if *text == expected);
}

fn assert_sequence_of_maps<'arena>(
    map: &ArenaQueryMap<'arena>,
    key: &str,
    field: &str,
    expected_values: &[&str],
) {
    let entries = map.entries_slice();
    assert_eq!(entries.len(), 1, "map should contain exactly one entry");
    let (entry_key, value) = &entries[0];
    assert_eq!(entry_key, &key, "sequence entry key should match");

    assert_matches!(value, ArenaValue::Seq(items) => {
        assert_eq!(items.len(), expected_values.len(), "sequence length should match");
        for (item, expected) in items.iter().zip(expected_values.iter()) {
            assert_matches!(item, ArenaValue::Map { entries, .. } => {
                assert_eq!(entries.len(), 1, "nested map should contain single entry");
                let (entry_key, value) = &entries[0];
                assert_eq!(entry_key, &field, "nested entry key should match");
                assert_matches!(value, ArenaValue::String(text) if *text == *expected);
            });
        }
    });
}

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

mod arena_is_placeholder {
    use super::*;

    #[test]
    fn should_detect_placeholders_when_value_is_empty_then_return_true() {
        let arena = ParseArena::new();
        let empty = ArenaValue::string(arena.alloc_str(""));
        let filled = ArenaValue::string(arena.alloc_str("value"));

        assert!(arena_is_placeholder(&empty));
        assert!(!arena_is_placeholder(&filled));
    }
}

mod insert_nested_value_arena {
    use super::*;

    #[test]
    fn should_noop_when_path_is_empty_then_leave_map_unchanged() {
        let arena = ParseArena::new();
        let mut map = map_with_capacity(&arena, 0);
        let mut state = acquire_pattern_state();

        insert_nested_value_arena(
            &arena,
            &mut map,
            &[],
            arena.alloc_str("ignored"),
            &mut state,
            DuplicateKeyBehavior::Reject,
        )
        .expect("empty path should be ignored");

        assert!(map.is_empty());
    }

    #[test]
    fn should_store_string_value_when_root_scalar_insert_occurs_then_store_value_in_map() {
        let arena = ParseArena::new();
        let mut map = map_with_capacity(&arena, 0);
        let mut state = acquire_pattern_state();

        insert_value(
            &arena,
            &mut map,
            &["token"],
            "abc123",
            &mut state,
            DuplicateKeyBehavior::Reject,
        )
        .expect("root insertion should succeed");

        assert_single_string_entry(&map, "token", "abc123");
    }

    #[test]
    fn should_return_duplicate_key_when_scalar_repeats_then_return_duplicate_key_error() {
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

        let error = insert_value(
            &arena,
            &mut map,
            &["token"],
            "second",
            &mut state,
            DuplicateKeyBehavior::Reject,
        )
        .expect_err("duplicate insert should fail");

        expect_duplicate_key(error, "token");
    }

    #[test]
    fn should_keep_initial_value_when_scalar_repeats_and_first_wins_then_preserve_first_value() {
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

        insert_value(
            &arena,
            &mut map,
            &["token"],
            "second",
            &mut state,
            DuplicateKeyBehavior::FirstWins,
        )
        .expect("duplicate insert should be ignored");

        assert_single_string_entry(&map, "token", "first");
    }

    #[test]
    fn should_replace_with_latest_value_when_scalar_repeats_and_last_wins_then_store_latest_value()
    {
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

        insert_value(
            &arena,
            &mut map,
            &["token"],
            "second",
            &mut state,
            DuplicateKeyBehavior::LastWins,
        )
        .expect("duplicate insert should overwrite");

        assert_single_string_entry(&map, "token", "second");
    }

    #[test]
    fn should_return_duplicate_key_when_nested_path_conflicts_with_scalar_then_keep_scalar() {
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

        let error = insert_value(
            &arena,
            &mut map,
            &["profile", "name"],
            "neo",
            &mut state,
            DuplicateKeyBehavior::LastWins,
        )
        .expect_err("scalar conflict should be rejected");

        expect_duplicate_key(error, "profile");
        assert_single_string_entry(&map, "profile", "raw");
    }

    #[test]
    fn should_return_duplicate_key_when_nested_map_rejects_duplicate_field_then_propagate_error() {
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

        let error = insert_value(
            &arena,
            &mut map,
            &["user", "name"],
            "bob",
            &mut state,
            DuplicateKeyBehavior::Reject,
        )
        .expect_err("duplicate nested value should fail");

        expect_duplicate_key(error, "name");
    }

    #[test]
    fn should_replace_nested_map_value_when_last_wins_enabled_then_overwrite_value() {
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

        insert_value(
            &arena,
            &mut map,
            &["user", "name"],
            "bob",
            &mut state,
            DuplicateKeyBehavior::LastWins,
        )
        .expect("last wins should overwrite nested map value");

        let entries = map.entries_slice();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, "user");
        let nested = entries[0]
            .1
            .as_map_slice()
            .expect("expected nested map when overwriting value");
        assert_eq!(nested.len(), 1);
        let (entry_key, entry_value) = &nested[0];
        assert_eq!(*entry_key, "name");
        assert!(
            matches!(entry_value, ArenaValue::String(text) if *text == "bob"),
            "expected overwritten string value"
        );
    }

    #[test]
    fn should_keep_nested_map_value_when_first_wins_then_preserve_existing_value() {
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

        insert_value(
            &arena,
            &mut map,
            &["user", "email"],
            "secondary@example.com",
            &mut state,
            DuplicateKeyBehavior::FirstWins,
        )
        .expect("first wins should preserve nested map value");

        let entries = map.entries_slice();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, "user");
        let nested = entries[0]
            .1
            .as_map_slice()
            .expect("expected nested map when preserving first value");
        assert_eq!(nested.len(), 1);
        let (entry_key, entry_value) = &nested[0];
        assert_eq!(*entry_key, "email");
        assert!(
            matches!(entry_value, ArenaValue::String(text) if *text == "primary@example.com"),
            "expected original email value"
        );
    }

    #[test]
    fn should_overwrite_placeholder_sequence_entry_when_existing_value_is_empty_then_store_latest_value() {
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

        insert_value(
            &arena,
            &mut map,
            &["items", "0"],
            "actual",
            &mut state,
            DuplicateKeyBehavior::Reject,
        )
        .expect("placeholder should accept concrete value");

        let entries = map.entries_slice();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, "items");
        let sequence = entries[0]
            .1
            .as_seq_slice()
            .expect("expected sequence container for placeholder replacement");
        assert_eq!(sequence.len(), 1);
        assert!(
            matches!(sequence[0], ArenaValue::String(text) if text == "actual"),
            "sequence entry should contain updated string"
        );
    }

    #[test]
    fn should_expand_sequence_of_maps_when_array_pattern_is_used_then_create_sequence_entries() {
        let arena = ParseArena::new();
        let mut map = map_with_capacity(&arena, 0);
        let mut state = acquire_pattern_state();
        let path = ["items", "", "name"];

        insert_sequence_values(
            &arena,
            &mut map,
            &path,
            &["alice", "bob"],
            &mut state,
            DuplicateKeyBehavior::Reject,
        );

        assert_sequence_of_maps(&map, "items", "name", &["alice", "bob"]);
    }

    #[test]
    fn should_return_duplicate_key_when_sequence_segment_is_non_numeric_then_reject_insertion() {
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

        let error = insert_value(
            &arena,
            &mut map,
            &["items", "name"],
            "override",
            &mut state,
            DuplicateKeyBehavior::Reject,
        )
        .expect_err("non-numeric segment should fail");

        expect_duplicate_key(error, "items");
    }

    #[test]
    fn should_return_duplicate_key_when_sequence_index_skips_existing_length_then_reject_sparse_insert() {
        let arena = ParseArena::new();
        let mut map = map_with_capacity(&arena, 0);
        let mut state = acquire_pattern_state();

        let error = insert_value(
            &arena,
            &mut map,
            &["items", "1"],
            "late",
            &mut state,
            DuplicateKeyBehavior::Reject,
        )
        .expect_err("sparse sequence index should be rejected");

        expect_duplicate_key(error, "items");
    }

    #[test]
    fn should_replace_sequence_value_when_last_wins_enabled_then_overwrite_entry() {
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

        insert_value(
            &arena,
            &mut map,
            &["items", "0"],
            "second",
            &mut state,
            DuplicateKeyBehavior::LastWins,
        )
        .expect("last wins should overwrite sequence value");

        let entries = map.entries_slice();
        assert_eq!(entries.len(), 1);
        let sequence = entries[0]
            .1
            .as_seq_slice()
            .expect("expected sequence container when overwriting entry");
        assert!(
            matches!(sequence.first(), Some(ArenaValue::String(text)) if *text == "second"),
            "sequence element should update to latest value"
        );
    }

    #[test]
    fn should_keep_sequence_value_when_first_wins_then_preserve_existing_entry() {
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

        insert_value(
            &arena,
            &mut map,
            &["items", "0"],
            "ignored",
            &mut state,
            DuplicateKeyBehavior::FirstWins,
        )
        .expect("first wins should ignore duplicates");

        let entries = map.entries_slice();
        assert_eq!(entries.len(), 1);
        let sequence = entries[0]
            .1
            .as_seq_slice()
            .expect("expected sequence container when preserving first entry");
        assert!(
            matches!(sequence.first(), Some(ArenaValue::String(text)) if *text == "initial"),
            "sequence element should remain the initial value"
        );
    }
}

mod arena_build_nested_path {
    use super::*;

    #[test]
    fn should_create_root_container_when_missing_then_allocate_map_container() {
        let arena = ParseArena::new();
        let mut map = map_with_capacity(&arena, 0);
        let mut state = acquire_pattern_state();
        let resolved = resolve_segments(&mut state, &["profile", "name"]).expect("resolve");

        arena_build_nested_path(
            &arena,
            &mut map,
            &resolved,
            arena.alloc_str("neo"),
            &state,
            "profile",
            DuplicateKeyBehavior::LastWins,
        )
        .expect("should build nested path");

        let entries = map.entries_slice();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, "profile");
        let nested = entries[0]
            .1
            .as_map_slice()
            .expect("root container should be a map after build");
        assert_eq!(nested.len(), 1);
        let (child_key, child_value) = &nested[0];
        assert_eq!(*child_key, "name");
        assert!(
            matches!(child_value, ArenaValue::String(value) if *value == "neo"),
            "nested value should match expected leaf"
        );
    }

    #[test]
    fn should_reuse_existing_container_when_root_already_exists_then_avoid_duplicate_entry() {
        let arena = ParseArena::new();
        let mut map = map_with_capacity(&arena, 1);
        map.try_insert_str(&arena, "profile", ArenaValue::map(&arena))
            .expect("seed profile container");
        let mut state = acquire_pattern_state();
        let resolved = resolve_segments(&mut state, &["profile", "email"]).expect("resolve");

        arena_build_nested_path(
            &arena,
            &mut map,
            &resolved,
            arena.alloc_str("primary@example.com"),
            &state,
            "profile",
            DuplicateKeyBehavior::LastWins,
        )
        .expect("should insert into existing root container");

        let entries = map.entries_slice();
        assert_eq!(entries.len(), 1, "root should not be duplicated");
        let nested = entries[0]
            .1
            .as_map_slice()
            .expect("root container should remain a map");
        assert_eq!(nested.len(), 1);
        assert_eq!(nested[0].0, "email");
    }
}

mod arena_set_nested_value {
    use super::*;

    #[test]
    fn should_return_ok_when_depth_exceeds_segment_length_then_exit_early() {
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

        arena_set_nested_value(
            &ctx,
            &mut current,
            &segments,
            segments.len(),
            arena.alloc_str("ignored"),
        )
        .expect("depth exhaustion should succeed");

        let entries = current
            .as_map_slice()
            .expect("expected map container after exhausting depth");
        assert!(entries.is_empty());
    }

    #[test]
    fn should_promote_string_node_to_map_when_state_has_no_hint_then_create_child_entry() {
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

        arena_set_nested_value(&ctx, &mut current, &segments, 1, arena.alloc_str("leaf"))
            .expect("string node should promote to map");

        let entries = current
            .as_map_slice()
            .expect("string node should promote to map");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, "child");
    }

    #[test]
    fn should_use_container_hint_to_convert_sequence_into_object_then_store_nested_value() {
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

        arena_set_nested_value(&ctx, &mut current, &resolved, 1, arena.alloc_str("value"))
            .expect("state hint should convert to map container");

        let entries = current
            .as_map_slice()
            .expect("state hint should convert sequence into map");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, "name");
    }

    #[test]
    fn should_return_duplicate_key_when_string_node_cannot_promote_with_override_then_report_error() {
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

        let error = with_string_promotion_suppressed(|| {
            arena_set_nested_value(&ctx, &mut current, &resolved, 1, arena.alloc_str("value"))
        })
        .expect_err("suppressed promotion should surface duplicate key error");

        expect_duplicate_key(error, "root");
    }

    #[test]
    fn should_error_with_unexpected_string_when_promotion_disabled_without_hints_then_return_duplicate_key() {
        let arena = ParseArena::new();
        let state = acquire_pattern_state();
        let segments = [
            ResolvedSegment::new(Cow::Borrowed("root")),
            ResolvedSegment::new(Cow::Borrowed("child")),
        ];
        let mut current = ArenaValue::string(arena.alloc_str("leaf"));
        let ctx = ArenaSetContext {
            arena: &arena,
            state: &state,
            root_key: "root",
            duplicate_keys: DuplicateKeyBehavior::Reject,
        };

        let error = with_string_promotion_suppressed(|| {
            arena_set_nested_value(&ctx, &mut current, &segments, 1, arena.alloc_str("value"))
        })
        .expect_err("suppressed promotion without hints should report duplicate key");

        expect_duplicate_key(error, "root");
    }
}

mod prepare_current_node {
    use super::*;

    #[test]
    fn should_promote_string_node_when_promotion_allowed_then_request_retry() {
        let arena = ParseArena::new();
        let state = acquire_pattern_state();
        let ctx = make_ctx(&arena, &state, "root", DuplicateKeyBehavior::LastWins);
        let mut node = ArenaValue::string(arena.alloc_str(""));
        let path = ["root"];

        let outcome = prepare_current_node(&ctx, &mut node, &path).expect("prepare");

        assert_matches!(outcome, NodePreparation::NeedsRetry);
        assert_matches!(node, ArenaValue::Map { .. });
    }

    #[test]
    fn should_keep_string_node_when_promotion_is_suppressed_then_mark_node_ready() {
        let arena = ParseArena::new();
        let state = acquire_pattern_state();
        let ctx = make_ctx(&arena, &state, "root", DuplicateKeyBehavior::LastWins);
        let mut node = ArenaValue::string(arena.alloc_str(""));
        let path = ["root"];

        let outcome =
            with_string_promotion_suppressed(|| prepare_current_node(&ctx, &mut node, &path))
                .expect("prepare");

        assert_matches!(outcome, NodePreparation::Ready);
        assert_matches!(node, ArenaValue::String(value) if value.is_empty());
    }

    #[test]
    fn should_convert_node_to_sequence_when_array_hint_present_then_transform_container() {
        let arena = ParseArena::new();
        let mut state = acquire_pattern_state();
        resolve_segments(&mut state, &["items", ""]).expect("resolve");
        assert_eq!(state.container_type(&["items"]), Some(ContainerType::Array));
        let ctx = make_ctx(&arena, &state, "items", DuplicateKeyBehavior::LastWins);
        let mut node = ArenaValue::map(&arena);
        let path = ["items"];

        prepare_current_node(&ctx, &mut node, &path).expect("prepare");
        assert_matches!(node, ArenaValue::Seq(_));
    }
}

mod visit_map_node {
    use super::*;

    #[test]
    fn should_descend_into_new_map_child_when_entry_is_missing_then_create_child_container() {
        let arena = ParseArena::new();
        let state = acquire_pattern_state();
        let ctx = make_ctx(&arena, &state, "root", DuplicateKeyBehavior::LastWins);
        let mut entries = arena.alloc_vec();
        let mut index = HashMap::<&str, usize, RandomState>::with_capacity_and_hasher(
            0,
            RandomState::default(),
        );
        let segments = [
            ResolvedSegment::new(Cow::Borrowed("root")),
            ResolvedSegment::new(Cow::Borrowed("child")),
        ];
        let mut path = SmallVec::<[&str; 16]>::new();
        let mut value_to_set = Some(arena.alloc_str("pending"));

        let step = visit_map_node(
            &ctx,
            &mut entries,
            &mut index,
            &segments,
            &mut path,
            0,
            "child",
            false,
            &mut value_to_set,
        )
        .expect("visit_map_node should succeed");

        assert_matches!(step, TraversalStep::Descend(_));
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, "child");
        assert!(value_to_set.is_some());
    }

    #[test]
    fn should_complete_traversal_when_leaf_value_inserted_then_store_scalar_value() {
        let arena = ParseArena::new();
        let state = acquire_pattern_state();
        let ctx = make_ctx(&arena, &state, "root", DuplicateKeyBehavior::LastWins);
        let mut entries = arena.alloc_vec();
        let mut index = HashMap::<&str, usize, RandomState>::with_capacity_and_hasher(
            0,
            RandomState::default(),
        );
        let segments = [ResolvedSegment::new(Cow::Borrowed("root"))];
        let mut path = SmallVec::<[&str; 16]>::new();
        let mut value_to_set = Some(arena.alloc_str("value"));

        let step = visit_map_node(
            &ctx,
            &mut entries,
            &mut index,
            &segments,
            &mut path,
            0,
            "root",
            true,
            &mut value_to_set,
        )
        .expect("visit_map_node should succeed");

        assert_matches!(step, TraversalStep::Complete);
        assert!(value_to_set.is_none());
        assert_eq!(entries.len(), 1);
        assert!(
            matches!(&entries[0].1, ArenaValue::String(text) if *text == "value"),
            "leaf entry should contain provided value"
        );
    }
}

mod visit_seq_node {
    use super::*;

    #[test]
    fn should_descend_into_sequence_slot_when_index_is_missing_then_allocate_container() {
        let arena = ParseArena::new();
        let state = acquire_pattern_state();
        let ctx = make_ctx(&arena, &state, "items", DuplicateKeyBehavior::LastWins);
        let mut items = arena.alloc_vec();
        let segments = [
            ResolvedSegment::new(Cow::Borrowed("items")),
            ResolvedSegment::new(Cow::Borrowed("0")),
            ResolvedSegment::new(Cow::Borrowed("name")),
        ];
        let mut path = SmallVec::<[&str; 16]>::new();
        path.push("items");
        let mut value_to_set = Some(arena.alloc_str("leaf"));

        let step = visit_seq_node(
            &ctx,
            &mut items,
            &segments,
            &mut path,
            1,
            "0",
            false,
            &mut value_to_set,
        )
        .expect("visit_seq_node should succeed");

        assert_matches!(step, TraversalStep::Descend(_));
        assert_eq!(items.len(), 1);
        assert_matches!(&items[0], ArenaValue::Map { .. });
        assert!(value_to_set.is_some());
    }

    #[test]
    fn should_complete_sequence_visit_when_leaf_inserted_then_store_value() {
        let arena = ParseArena::new();
        let state = acquire_pattern_state();
        let ctx = make_ctx(&arena, &state, "items", DuplicateKeyBehavior::LastWins);
        let mut items = arena.alloc_vec();
        let segments = [
            ResolvedSegment::new(Cow::Borrowed("items")),
            ResolvedSegment::new(Cow::Borrowed("0")),
        ];
        let mut path = SmallVec::<[&str; 16]>::new();
        path.push("items");
        let mut value_to_set = Some(arena.alloc_str("leaf"));

        let step = visit_seq_node(
            &ctx,
            &mut items,
            &segments,
            &mut path,
            1,
            "0",
            true,
            &mut value_to_set,
        )
        .expect("visit_seq_node should succeed");

        assert_matches!(step, TraversalStep::Complete);
        assert_eq!(items.len(), 1);
        assert_matches!(&items[0], ArenaValue::String(value) if *value == "leaf");
        assert!(value_to_set.is_none());
    }
}

mod child_capacity_hint {
    use super::*;

    #[test]
    fn should_cap_child_capacity_hint_when_hint_exceeds_limit_then_return_maximum() {
        let mut state = acquire_pattern_state();
        for idx in 0..70 {
            let segment = format!("field{idx}");
            let path = ["root", "branch", segment.as_str()];
            resolve_segments(&mut state, &path).expect("resolve should succeed");
        }

        let hint = child_capacity_hint(&state, &["root"], "branch");
        assert_eq!(hint, 64);
    }
}

mod handle_map_segment {
    use super::*;

    #[test]
    fn should_error_when_map_segment_missing_value_in_vacant_branch_then_return_duplicate_key_error() {
        let arena = ParseArena::new();
        let state = acquire_pattern_state();
        let ctx = make_ctx(&arena, &state, "root", DuplicateKeyBehavior::LastWins);
        let mut entries = arena.alloc_vec();
        let mut index = HashMap::<&str, usize, RandomState>::with_capacity_and_hasher(
            0,
            RandomState::default(),
        );
        let segments = [ResolvedSegment::new(Cow::Borrowed("root"))];
        let mut path = SmallVec::<[&str; 16]>::new();
        let mut missing: Option<&str> = None;

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

        assert!(result.is_err(), "expected duplicate key error");
        let err = result.err().unwrap();
        expect_duplicate_key(err, "child");
    }

    #[test]
    fn should_error_when_map_segment_updates_without_value_then_signal_duplicate() {
        let arena = ParseArena::new();
        let state = acquire_pattern_state();
        let ctx = make_ctx(&arena, &state, "root", DuplicateKeyBehavior::LastWins);
        let mut entries = arena.alloc_vec();
        let mut index = HashMap::<&str, usize, RandomState>::with_capacity_and_hasher(
            0,
            RandomState::default(),
        );
        let segments = [ResolvedSegment::new(Cow::Borrowed("root"))];
        let mut path = SmallVec::<[&str; 16]>::new();
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

        assert!(result.is_err(), "expected duplicate key error");
        let err = result.err().unwrap();
        expect_duplicate_key(err, "child");
    }

    #[test]
    fn should_replace_map_entry_when_last_wins_allows_duplicate_key_then_store_latest_value() {
        let arena = ParseArena::new();
        let state = acquire_pattern_state();
        let ctx = make_ctx(&arena, &state, "root", DuplicateKeyBehavior::LastWins);
        let mut entries = arena.alloc_vec();
        let mut index = HashMap::<&str, usize, RandomState>::with_capacity_and_hasher(
            0,
            RandomState::default(),
        );
        let segments = [ResolvedSegment::new(Cow::Borrowed("root"))];
        let mut path = SmallVec::<[&str; 16]>::new();
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
        .expect("initial insert should record value");

        let mut replacement = Some(arena.alloc_str("second"));
        handle_map_segment(
            &ctx,
            &mut entries,
            &mut index,
            &segments,
            &mut SmallVec::<[&str; 16]>::new(),
            0,
            "child",
            true,
            &mut replacement,
        )
        .expect("last wins should overwrite existing entry");

        assert_eq!(entries.len(), 1);
        let (key, value) = &entries[0];
        assert_eq!(*key, "child");
        assert!(
            matches!(value, ArenaValue::String(text) if *text == "second"),
            "replacement should store latest value"
        );
    }
}

mod handle_seq_segment {
    use super::*;

    #[test]
    fn should_error_when_numeric_segment_overflows_then_report_duplicate_key() {
        let arena = ParseArena::new();
        let state = acquire_pattern_state();
        let ctx = make_ctx(&arena, &state, "items", DuplicateKeyBehavior::LastWins);
        let mut items = arena.alloc_vec();
        let overflow = "184467440737095516160";
        let segments = [ResolvedSegment::new(Cow::Borrowed(overflow))];
        let mut path = SmallVec::<[&str; 16]>::new();
        let mut value_to_set = Some(arena.alloc_str("value"));

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

        assert!(result.is_err(), "expected duplicate key error");
        let err = result.err().unwrap();
        expect_duplicate_key(err, "items");
    }

    #[test]
    fn should_error_when_last_wins_sequence_missing_value_then_signal_duplicate() {
        let arena = ParseArena::new();
        let state = acquire_pattern_state();
        let ctx = make_ctx(&arena, &state, "items", DuplicateKeyBehavior::LastWins);
        let mut items = arena.alloc_vec();
        items.push(ArenaValue::string(arena.alloc_str("existing")));
        let segments = [ResolvedSegment::new(Cow::Borrowed("0"))];
        let mut path = SmallVec::<[&str; 16]>::new();
        let mut missing: Option<&str> = None;

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

        assert!(result.is_err(), "expected duplicate key error");
        let err = result.err().unwrap();
        expect_duplicate_key(err, "0");
    }

    #[test]
    fn should_error_when_sequence_segment_is_non_numeric_then_signal_duplicate_key() {
        let arena = ParseArena::new();
        let state = acquire_pattern_state();
        let ctx = make_ctx(&arena, &state, "items", DuplicateKeyBehavior::Reject);
        let mut items = arena.alloc_vec();
        let segments = [ResolvedSegment::new(Cow::Borrowed("alpha"))];
        let mut path = SmallVec::<[&str; 16]>::new();
        let mut value_to_set = Some(arena.alloc_str("value"));

        let result = handle_seq_segment(
            &ctx,
            &mut items,
            &segments,
            &mut path,
            0,
            "alpha",
            true,
            &mut value_to_set,
        );

        assert!(result.is_err(), "expected duplicate key error");
        let err = result.err().unwrap();
        expect_duplicate_key(err, "items");
    }

    #[test]
    fn should_error_when_sequence_append_missing_value_then_report_duplicate_key() {
        let arena = ParseArena::new();
        let state = acquire_pattern_state();
        let ctx = make_ctx(&arena, &state, "items", DuplicateKeyBehavior::LastWins);
        let mut items = arena.alloc_vec();
        let segments = [ResolvedSegment::new(Cow::Borrowed("0"))];
        let mut path = SmallVec::<[&str; 16]>::new();
        let mut value_to_set: Option<&str> = None;

        let result = handle_seq_segment(
            &ctx,
            &mut items,
            &segments,
            &mut path,
            0,
            "0",
            true,
            &mut value_to_set,
        );

        assert!(result.is_err(), "expected duplicate key error");
        let err = result.err().unwrap();
        expect_duplicate_key(err, "0");
    }

    #[test]
    fn should_error_when_sequence_placeholder_missing_value_then_report_duplicate_key() {
        let arena = ParseArena::new();
        let state = acquire_pattern_state();
        let ctx = make_ctx(&arena, &state, "items", DuplicateKeyBehavior::LastWins);
        let mut items = arena.alloc_vec();
        items.push(ArenaValue::string(arena.alloc_str("")));
        let segments = [ResolvedSegment::new(Cow::Borrowed("0"))];
        let mut path = SmallVec::<[&str; 16]>::new();
        let mut value_to_set: Option<&str> = None;

        let result = handle_seq_segment(
            &ctx,
            &mut items,
            &segments,
            &mut path,
            0,
            "0",
            true,
            &mut value_to_set,
        );

        assert!(result.is_err(), "expected duplicate key error");
        let err = result.err().unwrap();
        expect_duplicate_key(err, "0");
    }
}

mod try_insert_or_duplicate {
    use super::*;

    #[test]
    fn should_convert_duplicate_error_when_helper_insert_fails_then_return_parse_error() {
        let error = try_insert_or_duplicate("token", || Err(()))
            .expect_err("helper should convert duplicate into ParseError");

        expect_duplicate_key(error, "token");
    }

    #[test]
    fn should_return_ok_when_helper_insert_succeeds_then_propagate_success() {
        try_insert_or_duplicate("token", || Ok(()))
            .expect("helper should propagate successful insertion");
    }
}

mod get_root_value {
    use super::*;

    #[test]
    fn should_return_duplicate_key_when_root_value_missing_then_return_error() {
        let arena = ParseArena::new();
        let mut map = map_with_capacity(&arena, 0);

        let result = get_root_value(&mut map, "profile", "profile");
        assert!(result.is_err(), "expected duplicate key error");
        let err = result.err().unwrap();
        expect_duplicate_key(err, "profile");
    }
}

mod unexpected_nested_string {
    use super::*;

    #[test]
    fn should_surface_duplicate_key_error_when_nested_string_encountered_then_return_parse_error() {
        let error = unexpected_nested_string("profile");
        expect_duplicate_key(error, "profile");
    }
}

mod resolve_segments {
    use super::*;

    #[test]
    fn should_increment_indices_when_array_segment_repeats_then_update_sequence_indices() {
        let mut state = acquire_pattern_state();
        let path = ["items", "", "name"];

        let first = resolve_segments(&mut state, &path).expect("first resolve");
        let second = resolve_segments(&mut state, &path).expect("second resolve");

        assert_eq!(first[1].as_str(), "0");
        assert_eq!(second[1].as_str(), "1");
    }

    #[test]
    fn should_return_original_segments_when_path_is_literal_then_preserve_path_segments() {
        let mut state = acquire_pattern_state();
        let path = ["profile", "name"];

        let resolved = resolve_segments(&mut state, &path).expect("resolve");

        assert_eq!(resolved.len(), path.len());
        assert_eq!(resolved[0].as_str(), "profile");
        assert_eq!(resolved[1].as_str(), "name");
    }

    #[test]
    fn should_return_single_segment_when_path_contains_only_root_segment_then_preserve_root_segment() {
        let mut state = acquire_pattern_state();
        let path = ["token"];

        let resolved = resolve_segments(&mut state, &path).expect("resolve");

        assert_eq!(resolved.len(), 1);
        assert_eq!(resolved[0].as_str(), "token");
    }
}
