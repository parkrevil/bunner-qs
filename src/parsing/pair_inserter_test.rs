use super::*;
use crate::DuplicateKeyBehavior;
use crate::arena_helpers::map_with_capacity;
use crate::nested::pattern_state::acquire_pattern_state;
use crate::parsing::arena::ArenaValue;
use crate::parsing_helpers::expect_duplicate_key;
use assert_matches::assert_matches;

mod insert_pair_arena {
    use super::*;

    #[test]
    fn should_insert_string_value_when_flat_key_is_unique_then_store_entry_once() {
        let arena = ParseArena::new();
        let mut map = map_with_capacity(&arena, 2);
        let mut pattern_state = acquire_pattern_state();

        insert_pair_arena(
            &arena,
            &mut map,
            &mut pattern_state,
            Cow::Borrowed("foo"),
            Cow::Borrowed("bar"),
            DuplicateKeyBehavior::Reject,
        )
        .expect("insert succeeds");

        let entries = map.entries_slice();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, "foo");
        assert_matches!(&entries[0].1, ArenaValue::String(value) if *value == "bar");
    }

    #[test]
    fn should_return_duplicate_key_error_when_flat_key_repeats_then_include_conflicting_key() {
        let arena = ParseArena::new();
        let mut map = map_with_capacity(&arena, 2);
        let mut pattern_state = acquire_pattern_state();
        insert_pair_arena(
            &arena,
            &mut map,
            &mut pattern_state,
            Cow::Borrowed("foo"),
            Cow::Borrowed("first"),
            DuplicateKeyBehavior::Reject,
        )
        .expect("initial insert succeeds");

        let error = insert_pair_arena(
            &arena,
            &mut map,
            &mut pattern_state,
            Cow::Borrowed("foo"),
            Cow::Borrowed("second"),
            DuplicateKeyBehavior::Reject,
        )
        .expect_err("duplicate key error");

        expect_duplicate_key(error, "foo");
    }

    #[test]
    fn should_keep_initial_value_when_flat_key_repeats_and_first_wins_then_preserve_original_value()
    {
        let arena = ParseArena::new();
        let mut map = map_with_capacity(&arena, 2);
        let mut pattern_state = acquire_pattern_state();
        insert_pair_arena(
            &arena,
            &mut map,
            &mut pattern_state,
            Cow::Borrowed("foo"),
            Cow::Borrowed("first"),
            DuplicateKeyBehavior::FirstWins,
        )
        .expect("initial insert succeeds");

        insert_pair_arena(
            &arena,
            &mut map,
            &mut pattern_state,
            Cow::Borrowed("foo"),
            Cow::Borrowed("second"),
            DuplicateKeyBehavior::FirstWins,
        )
        .expect("duplicate insert ignored");

        let entries = map.entries_slice();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, "foo");
        assert_matches!(&entries[0].1, ArenaValue::String(value) if *value == "first");
    }

    #[test]
    fn should_replace_value_when_flat_key_repeats_and_last_wins_then_store_latest_value() {
        let arena = ParseArena::new();
        let mut map = map_with_capacity(&arena, 2);
        let mut pattern_state = acquire_pattern_state();
        insert_pair_arena(
            &arena,
            &mut map,
            &mut pattern_state,
            Cow::Borrowed("foo"),
            Cow::Borrowed("first"),
            DuplicateKeyBehavior::LastWins,
        )
        .expect("initial insert succeeds");

        insert_pair_arena(
            &arena,
            &mut map,
            &mut pattern_state,
            Cow::Borrowed("foo"),
            Cow::Borrowed("second"),
            DuplicateKeyBehavior::LastWins,
        )
        .expect("duplicate insert overwrites");

        let entries = map.entries_slice();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, "foo");
        assert_matches!(&entries[0].1, ArenaValue::String(value) if *value == "second");
    }

    #[test]
    fn should_store_under_empty_label_when_key_is_empty_then_use_empty_key() {
        let arena = ParseArena::new();
        let mut map = map_with_capacity(&arena, 2);
        let mut pattern_state = acquire_pattern_state();

        insert_pair_arena(
            &arena,
            &mut map,
            &mut pattern_state,
            Cow::Borrowed(""),
            Cow::Borrowed("value"),
            DuplicateKeyBehavior::Reject,
        )
        .expect("empty key insert succeeds");

        let entries = map.entries_slice();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, "");
        assert_matches!(&entries[0].1, ArenaValue::String(value) if *value == "value");
    }

    #[test]
    fn should_return_duplicate_key_error_when_empty_key_repeats_then_signal_root_duplicate() {
        let arena = ParseArena::new();
        let mut map = map_with_capacity(&arena, 2);
        let mut pattern_state = acquire_pattern_state();
        insert_pair_arena(
            &arena,
            &mut map,
            &mut pattern_state,
            Cow::Borrowed(""),
            Cow::Borrowed("first"),
            DuplicateKeyBehavior::Reject,
        )
        .expect("initial insert succeeds");

        let error = insert_pair_arena(
            &arena,
            &mut map,
            &mut pattern_state,
            Cow::Borrowed(""),
            Cow::Borrowed("second"),
            DuplicateKeyBehavior::Reject,
        )
        .expect_err("duplicate root key should error");

        expect_duplicate_key(error, "");
    }

    #[test]
    fn should_insert_nested_segments_when_key_contains_brackets_then_defer_to_nested_inserter() {
        let arena = ParseArena::new();
        let mut map = map_with_capacity(&arena, 4);
        let mut pattern_state = acquire_pattern_state();

        insert_pair_arena(
            &arena,
            &mut map,
            &mut pattern_state,
            Cow::Borrowed("user[profile][name]"),
            Cow::Borrowed("neo"),
            DuplicateKeyBehavior::Reject,
        )
        .expect("nested insert should succeed");

        let entries = map.entries_slice();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, "user");

        assert_matches!(&entries[0].1, ArenaValue::Map { entries: profile_entries, .. } => {
            assert_eq!(profile_entries.len(), 1);
            assert_eq!(profile_entries[0].0, "profile");

            assert_matches!(
                &profile_entries[0].1,
                ArenaValue::Map { entries: inner_entries, .. } => {
                    assert_eq!(inner_entries.len(), 1);
                    assert_eq!(inner_entries[0].0, "name");
                    assert_matches!(
                        &inner_entries[0].1,
                        ArenaValue::String(text) if *text == "neo"
                    );
                }
            );
        });
    }
}
