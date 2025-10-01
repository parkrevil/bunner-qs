use super::*;

mod parse_arena_new {
    use super::*;

    #[test]
    fn should_allocate_string_when_using_alloc_str_then_return_same_reference() {
        // Arrange
        let arena = ParseArena::new();

        // Act
        let stored = arena.alloc_str("hello");

        // Assert
        assert_eq!(stored, "hello");
    }

    #[test]
    fn should_return_empty_vector_when_alloc_vec_is_called_then_provide_zero_length_slice() {
        // Arrange
        let arena = ParseArena::new();

        // Act
        let values: ArenaVec<'_, i32> = arena.alloc_vec();

        // Assert
        assert!(values.is_empty());
    }
}

mod arena_query_map_insert {
    use super::*;

    #[test]
    fn should_store_value_when_inserting_unique_key_then_increase_map_length() {
        // Arrange
        let lease = acquire_parse_arena(0);
        let arena: &ParseArena = &lease;
        let mut map = ArenaQueryMap::with_capacity(arena, 1);
        let value = ArenaValue::string(arena.alloc_str("value"));

        // Act
        let result = map.try_insert_str(arena, "key", value);

        // Assert
        assert!(result.is_ok());
        assert_eq!(map.len(), 1);
        assert_eq!(map.entries_slice()[0].0, "key");
    }

    #[test]
    fn should_return_error_when_inserting_duplicate_key_then_prevent_overwrite() {
        // Arrange
        let lease = acquire_parse_arena(0);
        let arena: &ParseArena = &lease;
        let mut map = ArenaQueryMap::with_capacity(arena, 1);
        let value = ArenaValue::string(arena.alloc_str("first"));
        map.try_insert_str(arena, "key", value)
            .expect("first insert");
        let duplicate = ArenaValue::string(arena.alloc_str("second"));

        // Act
        let result = map.try_insert_str(arena, "key", duplicate);

        // Assert
        assert!(result.is_err());
    }
}

mod arena_query_map_get_mut {
    use super::*;

    #[test]
    fn should_store_values_when_mutating_sequence_entry_then_append_new_item() {
        // Arrange
        let lease = acquire_parse_arena(0);
        let arena: &ParseArena = &lease;
        let mut map = ArenaQueryMap::with_capacity(arena, 1);
        let sequence = ArenaValue::seq_with_capacity(arena, 0);
        map.try_insert_str(arena, "items", sequence)
            .expect("insert sequence");

        // Act
        let entry = map.get_mut("items").expect("sequence entry");
        if let ArenaValue::Seq(values) = entry {
            values.push(ArenaValue::string(arena.alloc_str("one")));
        } else {
            panic!("expected sequence value");
        }

        // Assert
        let stored = map.entries_slice()[0]
            .1
            .as_seq_slice()
            .expect("sequence slice");
        assert_eq!(stored.len(), 1);
        assert!(matches!(stored[0], ArenaValue::String("one")));
    }
}

mod arena_value_accessors {
    use super::*;

    #[test]
    fn should_create_empty_map_when_requested_capacity_provided_then_return_empty_entries() {
        // Arrange
        let lease = acquire_parse_arena(0);
        let arena: &ParseArena = &lease;

        // Act
        let value = ArenaValue::map_with_capacity(arena, 8);

        // Assert
        let entries = value.as_map_slice().expect("map slice");
        assert!(entries.is_empty());
    }

    #[test]
    fn should_create_empty_sequence_when_requested_capacity_provided_then_return_empty_entries() {
        // Arrange
        let lease = acquire_parse_arena(0);
        let arena: &ParseArena = &lease;

        // Act
        let value = ArenaValue::seq_with_capacity(arena, 5);

        // Assert
        let entries = value.as_seq_slice().expect("seq slice");
        assert!(entries.is_empty());
    }
}
