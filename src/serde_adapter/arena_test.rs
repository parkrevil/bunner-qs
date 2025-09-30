use super::*;
use crate::parsing::arena::{ArenaQueryMap, ArenaValue, ParseArena};
use serde_json::json;

mod arena_value_to_json {
    use super::*;

    #[test]
    fn when_converting_string_value_it_should_return_json_string() {
        // Arrange
        let value = ArenaValue::string("hello");

        // Act
        let json_value = arena_value_to_json(&value);

        // Assert
        assert_eq!(json_value, json!("hello"));
    }

    #[test]
    fn when_converting_sequence_it_should_preserve_ordered_elements() {
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
    fn when_converting_map_it_should_preserve_nested_structure() {
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
    fn when_converting_query_map_it_should_produce_object_value() {
        // Arrange
        let arena = ParseArena::new();
        let mut query_map = ArenaQueryMap::with_capacity(&arena, 2);
        query_map
            .try_insert_str(
                &arena,
                "name",
                ArenaValue::string(arena.alloc_str("Jane")),
            )
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
