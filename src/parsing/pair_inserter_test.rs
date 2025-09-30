use super::*;
use crate::DuplicateKeyBehavior;
use crate::nested::pattern_state::acquire_pattern_state;
use crate::parsing::arena::{ArenaQueryMap, ArenaValue};
use crate::parsing_helpers::expect_duplicate_key;

mod insert_pair_arena {
    use super::*;

    #[test]
    fn when_flat_key_unique_should_insert_string_value() {
        // Arrange
        let arena = ParseArena::new();
        let mut map = ArenaQueryMap::with_capacity(&arena, 2);
        let mut pattern_state = acquire_pattern_state();

        // Act
        insert_pair_arena(
            &arena,
            &mut map,
            &mut pattern_state,
            Cow::Borrowed("foo"),
            Cow::Borrowed("bar"),
            DuplicateKeyBehavior::Reject,
        )
        .expect("insert succeeds");

        // Assert
        let entries = map.entries_slice();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, "foo");
        if let ArenaValue::String(value) = entries[0].1 {
            assert_eq!(value, "bar");
        } else {
            panic!("expected string value");
        }
    }

    #[test]
    fn when_flat_key_repeats_should_return_duplicate_key_error() {
        // Arrange
        let arena = ParseArena::new();
        let mut map = ArenaQueryMap::with_capacity(&arena, 2);
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

        // Act
        let error = insert_pair_arena(
            &arena,
            &mut map,
            &mut pattern_state,
            Cow::Borrowed("foo"),
            Cow::Borrowed("second"),
            DuplicateKeyBehavior::Reject,
        )
        .expect_err("duplicate key error");

        // Assert
        expect_duplicate_key(error, "foo");
    }

    #[test]
    fn when_flat_key_repeats_and_first_wins_should_keep_initial_value() {
        // Arrange
        let arena = ParseArena::new();
        let mut map = ArenaQueryMap::with_capacity(&arena, 2);
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

        // Act
        insert_pair_arena(
            &arena,
            &mut map,
            &mut pattern_state,
            Cow::Borrowed("foo"),
            Cow::Borrowed("second"),
            DuplicateKeyBehavior::FirstWins,
        )
        .expect("duplicate insert ignored");

        // Assert
        let entries = map.entries_slice();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, "foo");
        if let ArenaValue::String(value) = entries[0].1 {
            assert_eq!(value, "first");
        } else {
            panic!("expected string value");
        }
    }

    #[test]
    fn when_flat_key_repeats_and_last_wins_should_replace_value() {
        // Arrange
        let arena = ParseArena::new();
        let mut map = ArenaQueryMap::with_capacity(&arena, 2);
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

        // Act
        insert_pair_arena(
            &arena,
            &mut map,
            &mut pattern_state,
            Cow::Borrowed("foo"),
            Cow::Borrowed("second"),
            DuplicateKeyBehavior::LastWins,
        )
        .expect("duplicate insert overwrites");

        // Assert
        let entries = map.entries_slice();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, "foo");
        if let ArenaValue::String(value) = entries[0].1 {
            assert_eq!(value, "second");
        } else {
            panic!("expected string value");
        }
    }

    #[test]
    fn when_key_is_empty_should_store_under_empty_label() {
        // Arrange
        let arena = ParseArena::new();
        let mut map = ArenaQueryMap::with_capacity(&arena, 2);
        let mut pattern_state = acquire_pattern_state();

        // Act
        insert_pair_arena(
            &arena,
            &mut map,
            &mut pattern_state,
            Cow::Borrowed(""),
            Cow::Borrowed("value"),
            DuplicateKeyBehavior::Reject,
        )
        .expect("empty key insert succeeds");

        // Assert
        let entries = map.entries_slice();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].0, "");
        if let ArenaValue::String(value) = entries[0].1 {
            assert_eq!(value, "value");
        } else {
            panic!("expected string value");
        }
    }
}
