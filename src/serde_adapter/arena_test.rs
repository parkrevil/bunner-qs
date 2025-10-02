use super::*;
use crate::arena_helpers::map_with_capacity;
use crate::parsing::arena::{ArenaValue, ParseArena};
use serde_json::json;

mod arena_value_to_json {
    use super::*;

    #[test]
    fn should_convert_string_value_into_json_string_when_value_is_string_then_return_json_string() {
        // Arrange
        let value = ArenaValue::string("hello");

        // Act
        let json_value = arena_value_to_json(&value);

        // Assert
        assert_eq!(json_value, json!("hello"));
    }

    #[test]
    fn should_preserve_order_when_converting_sequence_into_json_when_sequence_contains_multiple_values_then_preserve_sequence_order()
     {
        // Arrange
        let arena = ParseArena::new();
        let mut values = arena.alloc_vec();
        values.push(ArenaValue::string(arena.alloc_str("alpha")));
        values.push(ArenaValue::string(arena.alloc_str("beta")));
        let sequence = ArenaValue::Seq(values);

        // Act
        let json_value = arena_value_to_json(&sequence);

        // Assert
        assert_eq!(json_value, json!(["alpha", "beta"]));
    }

    #[test]
    fn should_preserve_nested_structure_when_converting_map_into_json_when_map_contains_nested_entries_then_produce_nested_json_structure()
     {
        // Arrange
        let arena = ParseArena::new();
        let mut entries = arena.alloc_vec();
        entries.push((
            arena.alloc_str("city"),
            ArenaValue::string(arena.alloc_str("Seoul")),
        ));
        entries.push((
            arena.alloc_str("country"),
            ArenaValue::string(arena.alloc_str("Korea")),
        ));
        let map_value = ArenaValue::Map {
            entries,
            index: Default::default(),
        };

        // Act
        let json_value = arena_value_to_json(&map_value);

        // Assert
        assert_eq!(json_value, json!({"city": "Seoul", "country": "Korea"}));
    }
}

mod arena_map_to_json_value {
    use super::*;

    #[test]
    fn should_produce_object_value_when_converting_query_map_into_json_when_map_contains_sequences_then_create_json_object()
     {
        // Arrange
        let arena = ParseArena::new();
        let mut query_map = map_with_capacity(&arena, 2);
        query_map
            .try_insert_str(&arena, "name", ArenaValue::string(arena.alloc_str("Jane")))
            .expect("unique key should insert");

        let mut hobbies_items = arena.alloc_vec();
        hobbies_items.push(ArenaValue::string(arena.alloc_str("tea")));
        hobbies_items.push(ArenaValue::string(arena.alloc_str("hiking")));
        let hobbies = ArenaValue::Seq(hobbies_items);

        query_map
            .try_insert_str(&arena, "hobbies", hobbies)
            .expect("unique key should insert");

        // Act
        let json_value = arena_map_to_json_value(&query_map);

        // Assert
        assert_eq!(
            json_value,
            json!({
                "name": "Jane",
                "hobbies": ["tea", "hiking"]
            })
        );
    }
}
