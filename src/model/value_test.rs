use super::*;

mod new {
    use super::*;

    #[test]
    fn should_create_empty_query_map_when_new_is_called_then_return_empty_map() {
        // Arrange
        // Act
        let map = QueryMap::new();

        // Assert
        assert!(map.is_empty());
    }
}

mod with_capacity {
    use super::*;

    #[test]
    fn should_match_default_when_reserving_zero_capacity_then_behave_like_empty_map() {
        // Arrange
        let baseline = QueryMap::new();

        // Act
        let map = QueryMap::with_capacity(0);

        // Assert
        assert_eq!(map.len(), baseline.len());
        assert!(map.is_empty());
    }

    #[test]
    fn should_store_items_when_reserving_capacity_then_store_inserted_pairs() {
        // Arrange
        let mut map = QueryMap::with_capacity(2);

        // Act
        map.insert(String::from("alpha"), Value::from("one"));
        map.insert(String::from("beta"), Value::from("two"));

        // Assert
        assert_eq!(map.len(), 2);
    }
}

mod from_iter {
    use super::*;

    #[test]
    fn should_preserve_insertion_order_when_collecting_pairs_then_collect_pairs_in_order() {
        // Arrange
        let pairs = [
            (String::from("first"), Value::from("one")),
            (String::from("second"), Value::from("two")),
        ];

        // Act
        let collected: QueryMap = pairs.clone().into_iter().collect();

        // Assert
        let items: Vec<_> = collected.into_iter().collect();
        assert_eq!(items, pairs);
    }
}

mod into_iterator {
    use super::*;

    #[test]
    fn should_yield_borrowed_values_when_iterating_immutably_then_iterate_over_entries() {
        // Arrange
        let mut map = QueryMap::new();
        map.insert(String::from("flag"), Value::from("on"));

        // Act
        let mut iter = (&map).into_iter();

        // Assert
        let (key, value) = iter.next().expect("entry should exist");
        assert_eq!(key, &String::from("flag"));
        assert_eq!(value, &Value::from("on"));
        assert!(iter.next().is_none());
    }

    #[test]
    fn should_allow_updates_when_iterating_mutably_then_mutate_entries() {
        // Arrange
        let mut map = QueryMap::new();
        map.insert(String::from("flag"), Value::from("off"));

        // Act
        for (_key, value) in (&mut map).into_iter() {
            *value = Value::from("on");
        }

        // Assert
        assert_eq!(map.get("flag"), Some(&Value::from("on")));
    }
}

mod value_from {
    use super::*;

    #[test]
    fn should_wrap_owned_string_when_constructing_value_string_then_store_owned_string() {
        // Arrange
        let source = String::from("hello");

        // Act
        let value = Value::from(source.clone());

        // Assert
        assert_eq!(value, Value::String(source));
    }

    #[test]
    fn should_clone_str_slice_when_constructing_value_string_then_store_cloned_string() {
        // Arrange
        let source = "world";

        // Act
        let value = Value::from(source);

        // Assert
        assert_eq!(value, Value::String(String::from(source)));
    }
}

mod conversions {
    use super::*;

    #[test]
    fn should_construct_query_map_from_ordered_map_then_preserve_entries() {
        // Arrange
        let mut ordered = OrderedMap::default();
        ordered.insert(String::from("id"), Value::from("42"));

        // Act
        let query_map = QueryMap::from(ordered.clone());

        // Assert
        assert_eq!(query_map.len(), 1);
        assert_eq!(query_map.get("id"), ordered.get("id"));
    }

    #[test]
    fn should_convert_query_map_back_into_ordered_map_then_yield_same_contents() {
        // Arrange
        let mut query_map = QueryMap::new();
        query_map.insert(String::from("role"), Value::from("admin"));

        // Act
        let ordered: OrderedMap<_, _> = query_map.clone().into();

        // Assert
        assert_eq!(ordered.len(), 1);
        assert_eq!(ordered.get("role"), Some(&Value::from("admin")));
        assert!(
            query_map.get("role").is_some(),
            "original map should remain intact"
        );
    }

    #[test]
    fn should_consume_query_map_when_using_from_then_preserve_entries() {
        // Arrange
        let mut query_map = QueryMap::new();
        query_map.insert(String::from("token"), Value::from("abc123"));

        // Act
        let ordered = OrderedMap::from(query_map);

        // Assert
        assert_eq!(ordered.len(), 1);
        assert_eq!(ordered.get("token"), Some(&Value::from("abc123")));
    }
}

mod value_accessors {
    use super::*;

    #[test]
    fn should_return_borrowed_str_when_value_is_string_then_expose_slice() {
        // Arrange
        let value = Value::from("access");

        // Act
        let result = value.as_str();

        // Assert
        assert_eq!(result, Some("access"));
        assert!(value.is_string());
        assert!(!value.is_array());
        assert!(!value.is_object());
    }

    #[test]
    fn should_return_none_when_calling_as_str_on_non_string_then_reject_conversion() {
        // Arrange
        let value = Value::Array(vec![]);

        // Act & Assert
        assert!(value.as_str().is_none());
        assert!(!value.is_string());
    }

    #[test]
    fn should_return_slice_when_value_is_array_then_expose_elements() {
        // Arrange
        let value = Value::Array(vec![Value::from("a"), Value::from("b")]);

        // Act
        let slice = value.as_array().expect("array should be exposed as slice");

        // Assert
        assert_eq!(slice.len(), 2);
        assert!(value.is_array());
        assert!(!value.is_object());
    }

    #[test]
    fn should_return_map_reference_when_value_is_object_then_expose_entries() {
        // Arrange
        let mut map = OrderedMap::default();
        map.insert(String::from("name"), Value::from("Neo"));
        let value = Value::Object(map);

        // Act
        let object = value
            .as_object()
            .expect("object should be exposed as ordered map");

        // Assert
        assert_eq!(object.get("name"), Some(&Value::from("Neo")));
        assert!(value.is_object());
        assert!(!value.is_string());
        assert!(!value.is_array());
    }
}

mod query_map_from_struct {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Credentials {
        username: String,
        token: String,
        active: bool,
        roles: Vec<String>,
        nickname: Option<String>,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Entry {
        id: u32,
        title: String,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Library {
        owner: Credentials,
        entries: Vec<Entry>,
    }

    #[test]
    fn should_serialize_struct_into_query_map_when_using_from_struct() {
        // Arrange
        let payload = Credentials {
            username: String::from("neo"),
            token: String::from("abc123"),
            active: true,
            roles: vec![String::from("admin"), String::from("operator")],
            nickname: None,
        };

        // Act
        let map = QueryMap::from_struct(&payload).expect("serialization should succeed");

        // Assert
        assert_eq!(map.get("username"), Some(&Value::from("neo")));
        assert_eq!(map.get("token"), Some(&Value::from("abc123")));
        assert_eq!(map.get("active"), Some(&Value::from("true")));

        let roles = map
            .get("roles")
            .and_then(Value::as_array)
            .expect("roles should serialize as array");
        assert_eq!(roles.len(), 2);
        assert!(map.get("nickname").is_none(), "None should be omitted");
    }

    #[test]
    fn should_serialize_nested_structures_and_arrays_when_using_from_struct() {
        // Arrange
        let payload = Library {
            owner: Credentials {
                username: String::from("trinity"),
                token: String::from("xyz789"),
                active: false,
                roles: vec![String::from("reader")],
                nickname: Some(String::from("Tri")),
            },
            entries: vec![
                Entry {
                    id: 1,
                    title: String::from("Matrix 101"),
                },
                Entry {
                    id: 2,
                    title: String::from("Rust Patterns"),
                },
            ],
        };

        // Act
        let map = QueryMap::from_struct(&payload).expect("serialization should succeed");

        // Assert
        let owner = map
            .get("owner")
            .and_then(Value::as_object)
            .expect("owner should be serialized as object");
        assert_eq!(owner.get("username"), Some(&Value::from("trinity")));
        assert_eq!(owner.get("active"), Some(&Value::from("false")));

        let entries = map
            .get("entries")
            .and_then(Value::as_array)
            .expect("entries should serialize as array");
        assert_eq!(entries.len(), 2);
        assert!(entries.iter().all(Value::is_object));
    }
}

mod query_map_to_struct {
    use super::*;
    use serde::{Deserialize, Serialize};
    use crate::SerdeQueryError;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Dimensions {
        width: u32,
        height: u32,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Document {
        title: String,
        cover: Dimensions,
        tags: Vec<String>,
    }

    #[test]
    fn should_deserialize_nested_struct_from_query_map_when_using_to_struct() {
        // Arrange
        let mut cover = OrderedMap::default();
        cover.insert(String::from("width"), Value::from("800"));
        cover.insert(String::from("height"), Value::from("600"));

        let mut map = QueryMap::new();
        map.insert(String::from("title"), Value::from("Rust Adventures"));
        map.insert(String::from("cover"), Value::Object(cover));
        map.insert(
            String::from("tags"),
            Value::Array(vec![Value::from("rust"), Value::from("web")]),
        );

        // Act
        let document: Document = map.to_struct().expect("deserialization should succeed");

        // Assert
        assert_eq!(document.title, "Rust Adventures");
        assert_eq!(document.cover.width, 800);
        assert_eq!(document.tags, vec!["rust", "web"]);
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Profile {
        name: String,
        age: u8,
    }

    #[test]
    fn should_return_error_when_field_type_mismatch_then_propagate_serde_failure() {
        // Arrange
        let mut map = QueryMap::new();
        map.insert(String::from("name"), Value::from("Morpheus"));
        map.insert(String::from("age"), Value::from("not-a-number"));

        // Act
        let result = map.to_struct::<Profile>();

        // Assert
        let err = result.expect_err("invalid number should fail");
        match err {
            SerdeQueryError::Deserialize(source) => {
                assert!(source.to_string().contains("invalid number"));
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}

mod clone_value_into_arena {
    use super::*;
    use super::clone_value_into_arena_for_test;

    #[test]
    fn should_clone_string_into_arena_then_produce_arena_string_value() {
        // Arrange
        let arena = ParseArena::new();
        let value = Value::from("matrix");

        // Act
        let cloned = clone_value_into_arena_for_test(&arena, &value);

        // Assert
        if let ArenaValue::String(text) = cloned {
            assert_eq!(text, "matrix");
        } else {
            panic!("expected ArenaValue::String");
        }
    }

    #[test]
    fn should_clone_array_into_arena_then_preserve_element_structure() {
        // Arrange
        let arena = ParseArena::new();
        let value = Value::Array(vec![
            Value::from("alpha"),
            Value::Object({
                let mut inner = OrderedMap::default();
                inner.insert(String::from("beta"), Value::from("bravo"));
                inner
            }),
        ]);

        // Act
        let cloned = clone_value_into_arena_for_test(&arena, &value);

        // Assert
        match cloned {
            ArenaValue::Seq(items) => {
                assert_eq!(items.len(), 2);
                match &items[0] {
                    ArenaValue::String(text) => assert_eq!(*text, "alpha"),
                    _ => panic!("expected first element to be ArenaValue::String"),
                }
                match &items[1] {
                    ArenaValue::Map { entries, .. } => {
                        assert_eq!(entries.len(), 1);
                        assert_eq!(entries[0].0, "beta");
                        match &entries[0].1 {
                            ArenaValue::String(text) => assert_eq!(*text, "bravo"),
                            _ => panic!("expected nested value to be ArenaValue::String"),
                        }
                    }
                    _ => panic!("expected second element to be ArenaValue::Map"),
                }
            }
            _ => panic!("expected cloned value to be ArenaValue::Seq"),
        }
    }

    #[test]
    fn should_clone_object_into_arena_then_build_indexed_entries() {
        // Arrange
        let arena = ParseArena::new();
        let mut payload = OrderedMap::default();
        payload.insert(String::from("gamma"), Value::from("3"));
        payload.insert(String::from("delta"), Value::Array(vec![Value::from("1")]));
        let value = Value::Object(payload);

        // Act
        let cloned = clone_value_into_arena_for_test(&arena, &value);

        // Assert
        match cloned {
            ArenaValue::Map { entries, index } => {
                assert_eq!(entries.len(), 2);
                assert_eq!(index.len(), 2);

                let &gamma_index = index.get("gamma").expect("gamma key should exist");
                match &entries[gamma_index].1 {
                    ArenaValue::String(text) => assert_eq!(*text, "3"),
                    _ => panic!("expected gamma entry to be ArenaValue::String"),
                }

                let &delta_index = index.get("delta").expect("delta key should exist");
                match &entries[delta_index].1 {
                    ArenaValue::Seq(items) => {
                        assert_eq!(items.len(), 1);
                        match &items[0] {
                            ArenaValue::String(text) => assert_eq!(*text, "1"),
                            _ => panic!("expected nested array element to be ArenaValue::String"),
                        }
                    }
                    _ => panic!("expected delta entry to be ArenaValue::Seq"),
                }
            }
            _ => panic!("expected cloned value to be ArenaValue::Map"),
        }
    }
}
