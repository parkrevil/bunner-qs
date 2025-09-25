#[cfg(feature = "serde")]
mod serde_tests {
    use bunner_qs::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct TestStruct {
        name: String,
        age: u32,
        tags: Vec<String>,
    }

    #[test]
    fn test_serde_roundtrip() {
        let original = TestStruct {
            name: "John".to_string(),
            age: 30,
            tags: vec!["rust".to_string(), "programming".to_string()],
        };

        // Struct -> QueryMap -> Struct
        let query_map = to_query_map(&original).unwrap();
        let restored: TestStruct = from_query_map(&query_map).unwrap();

        assert_eq!(original, restored);
    }

    #[test]
    fn test_query_map_to_struct() {
        let mut query_map = QueryMap::new();
        query_map.insert("name".to_string(), Value::String("Alice".to_string()));
        query_map.insert("age".to_string(), Value::String("25".to_string()));
        query_map.insert(
            "tags".to_string(),
            Value::Array(vec![
                Value::String("rust".to_string()),
                Value::String("web".to_string()),
            ]),
        );

        let result: TestStruct = from_query_map(&query_map).unwrap();

        assert_eq!(result.name, "Alice");
        assert_eq!(result.age, 25);
        assert_eq!(result.tags, vec!["rust", "web"]);
    }

    #[test]
    fn test_struct_to_query_map() {
        let data = TestStruct {
            name: "Bob".to_string(),
            age: 35,
            tags: vec!["developer".to_string()],
        };

        let query_map = to_query_map(&data).unwrap();

        // Check that the QueryMap contains expected values
        assert!(query_map.contains_key("name"));
        assert!(query_map.contains_key("age"));
        assert!(query_map.contains_key("tags"));
    }
}
