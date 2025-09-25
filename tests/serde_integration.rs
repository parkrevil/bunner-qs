#[cfg(feature = "serde")]
mod serde_tests {
    use bunner_qs::*;
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct SimpleStruct {
        name: String,
        age: u32,
    }

    #[test]
    fn test_simple_struct_roundtrip() {
        let original = SimpleStruct {
            name: "John".to_string(),
            age: 30,
        };

        // Struct -> QueryMap -> Struct
        let query_map = to_query_map(&original).unwrap();
        let restored: SimpleStruct = from_query_map(&query_map).unwrap();

        assert_eq!(original, restored);
    }

    #[test]
    fn test_hashmap_roundtrip() {
        let mut original = HashMap::new();
        original.insert("name".to_string(), "Alice".to_string());
        original.insert("city".to_string(), "Seoul".to_string());

        let query_map = to_query_map(&original).unwrap();
        let restored: HashMap<String, String> = from_query_map(&query_map).unwrap();

        assert_eq!(restored, original);
    }

    #[test]
    fn test_manual_query_map_to_struct() {
        let mut query_map = QueryMap::new();
        query_map.insert("name".to_string(), Value::String("Bob".to_string()));
        query_map.insert("age".to_string(), Value::String("25".to_string()));

        let result: SimpleStruct = from_query_map(&query_map).unwrap();

        assert_eq!(result.name, "Bob");
        assert_eq!(result.age, 25);
    }
}
