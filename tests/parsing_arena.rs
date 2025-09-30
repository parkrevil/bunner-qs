use bunner_qs::parsing::arena::{ArenaQueryMap, ArenaValue, ParseArena, acquire_parse_arena};

mod parse_arena {
    use super::*;

    #[test]
    fn alloc_str_should_return_interned_text() {
        // Arrange
        let arena = ParseArena::new();

        // Act
        let stored = arena.alloc_str("alpha");

        // Assert
        assert_eq!(stored, "alpha");
    }

    #[test]
    fn alloc_vec_should_allow_pushing_items() {
        // Arrange
        let arena = ParseArena::new();

        // Act
        let mut values = arena.alloc_vec();
        values.push(1);
        values.push(2);

        // Assert
        assert_eq!(values.as_slice(), &[1, 2]);
    }

    #[test]
    fn prepare_should_allow_reuse_after_growing_capacity() {
        // Arrange
        let mut arena = ParseArena::with_capacity(32);
        arena.alloc_str("warmup");

        // Act
        arena.prepare(256);
        let stored = arena.alloc_str("beta");

        // Assert
        assert_eq!(stored, "beta");
    }
}

mod parse_arena_guard {
    use super::*;

    #[test]
    fn acquire_parse_arena_should_return_guard_capable_of_allocating() {
        // Arrange & Act
        let guard = acquire_parse_arena(128);
        let stored = guard.alloc_str("session");

        // Assert
        assert_eq!(stored, "session");
    }
}

mod arena_query_map {
    use super::*;

    #[test]
    fn try_insert_str_should_store_entries_and_indices() {
        // Arrange
        let arena = ParseArena::new();
        let mut map = ArenaQueryMap::with_capacity(&arena, 4);
        let value = ArenaValue::string(arena.alloc_str("first"));

        // Act
        map.try_insert_str(&arena, "profile", value)
            .expect("first insert should succeed");
        let another = ArenaValue::string(arena.alloc_str("second"));
        map.try_insert_str(&arena, "settings", another)
            .expect("second insert should succeed");

        // Assert
        assert_eq!(map.len(), 2);
        assert!(map.contains_key("profile"));
        assert!(map.contains_key("settings"));
        let collected: Vec<_> = map.iter().collect();
        assert_eq!(collected.len(), 2);
        assert!(
            map.entries_slice()
                .iter()
                .any(|(key, value)| *key == "profile"
                    && matches!(value, ArenaValue::String(text) if *text == "first"))
        );
    }

    #[test]
    fn try_insert_str_should_reject_duplicate_keys() {
        // Arrange
        let arena = ParseArena::new();
        let mut map = ArenaQueryMap::with_capacity(&arena, 2);
        let first = ArenaValue::string(arena.alloc_str("one"));
        map.try_insert_str(&arena, "status", first)
            .expect("initial insert should succeed");
        let duplicate = ArenaValue::string(arena.alloc_str("two"));

        // Act
        let result = map.try_insert_str(&arena, "status", duplicate);

        // Assert
        assert_eq!(result, Err(()));
        assert_eq!(map.len(), 1);
    }
}

mod arena_value {
    use super::*;

    #[test]
    fn string_should_wrap_text_reference() {
        // Arrange
        let arena = ParseArena::new();
        let text = arena.alloc_str("payload");

        // Act
        let value = ArenaValue::string(text);

        // Assert
        match value {
            ArenaValue::String(inner) => assert_eq!(inner, "payload"),
            _ => panic!("expected string variant"),
        }
    }

    #[test]
    fn map_should_start_with_empty_entries() {
        // Arrange
        let arena = ParseArena::new();

        // Act
        let value = ArenaValue::map(&arena);

        // Assert
        match value {
            ArenaValue::Map { entries, .. } => assert!(entries.is_empty()),
            _ => panic!("expected map variant"),
        }
    }

    #[test]
    fn map_with_capacity_should_reserve_space_without_items() {
        // Arrange
        let arena = ParseArena::new();

        // Act
        let value = ArenaValue::map_with_capacity(&arena, 10);

        // Assert
        match value {
            ArenaValue::Map { entries, .. } => assert!(entries.is_empty()),
            _ => panic!("expected map variant"),
        }
    }

    #[test]
    fn seq_with_capacity_should_create_mutable_sequence() {
        // Arrange
        let arena = ParseArena::new();

        // Act
        let mut value = ArenaValue::seq_with_capacity(&arena, 3);
        if let ArenaValue::Seq(ref mut items) = value {
            items.push(ArenaValue::string(arena.alloc_str("first")));
            items.push(ArenaValue::string(arena.alloc_str("second")));
        } else {
            panic!("expected sequence variant");
        }

        // Assert
        match value {
            ArenaValue::Seq(items) => assert_eq!(items.len(), 2),
            _ => panic!("expected sequence variant"),
        }
    }

    #[test]
    fn as_seq_slice_should_return_some_for_sequences() {
        // Arrange
        let arena = ParseArena::new();
        let mut value = ArenaValue::seq_with_capacity(&arena, 2);
        if let ArenaValue::Seq(ref mut items) = value {
            items.push(ArenaValue::string(arena.alloc_str("alpha")));
        }

        // Act
        let slice = value.as_seq_slice();

        // Assert
        assert_eq!(slice.unwrap().len(), 1);
    }

    #[test]
    fn as_map_slice_should_return_some_for_maps() {
        // Arrange
        let arena = ParseArena::new();
        let mut map_value = ArenaValue::map(&arena);
        if let ArenaValue::Map {
            ref mut entries, ..
        } = map_value
        {
            entries.push((
                arena.alloc_str("key"),
                ArenaValue::string(arena.alloc_str("value")),
            ));
        }

        // Act
        let slice = map_value.as_map_slice();

        // Assert
        assert_eq!(slice.unwrap().len(), 1);
    }
}
