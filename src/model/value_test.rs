use super::*;
use assert_matches::assert_matches;

mod new {
    use super::*;

    #[test]
    fn should_create_empty_query_map_when_new_is_called_then_return_empty_map() {
        let map = QueryMap::new();

        assert!(map.is_empty());
    }
}

mod with_capacity {
    use super::*;

    #[test]
    fn should_match_default_when_reserving_zero_capacity_then_behave_like_empty_map() {
        let baseline = QueryMap::new();

        let map = QueryMap::with_capacity(0);

        assert_eq!(map.len(), baseline.len());
        assert!(map.is_empty());
    }

    #[test]
    fn should_store_items_when_reserving_capacity_then_store_inserted_pairs() {
        let mut map = QueryMap::with_capacity(2);

        map.insert(String::from("alpha"), Value::from("one"));
        map.insert(String::from("beta"), Value::from("two"));

        assert_eq!(map.len(), 2);
    }
}

#[test]
fn should_reserve_capacity_when_cloning_large_array_then_clone_all_items() {
    let arena = ParseArena::new();
    let value = Value::Array(vec![
        Value::from("one"),
        Value::from("two"),
        Value::from("three"),
        Value::from("four"),
        Value::from("five"),
        Value::from("six"),
    ]);

    let cloned = clone_value_into_arena_for_test(&arena, &value);

    assert_matches!(cloned, ArenaValue::Seq(items) => {
        assert_eq!(items.len(), 6);
        let mut collected = Vec::new();
        for item in items {
            assert_matches!(item, ArenaValue::String(text) => {
                collected.push(text.to_string());
            });
        }
        assert_eq!(collected, ["one", "two", "three", "four", "five", "six"]);
    });
}

mod from_iter {
    use super::*;

    #[test]
    fn should_preserve_insertion_order_when_collecting_pairs_then_collect_pairs_in_order() {
        let pairs = [
            (String::from("first"), Value::from("one")),
            (String::from("second"), Value::from("two")),
        ];

        let collected: QueryMap = pairs.clone().into_iter().collect();

        let items: Vec<_> = collected.into_iter().collect();
        assert_eq!(items, pairs);
    }
}

mod into_iterator {
    use super::*;

    #[test]
    fn should_yield_borrowed_values_when_iterating_immutably_then_iterate_over_entries() {
        let mut map = QueryMap::new();
        map.insert(String::from("flag"), Value::from("on"));

        let mut iter = (&map).into_iter();

        let (key, value) = iter.next().expect("entry should exist");
        assert_eq!(key, &String::from("flag"));
        assert_eq!(value, &Value::from("on"));
        assert!(iter.next().is_none());
    }

    #[test]
    fn should_allow_updates_when_iterating_mutably_then_mutate_entries() {
        let mut map = QueryMap::new();
        map.insert(String::from("flag"), Value::from("off"));

        for (_key, value) in (&mut map).into_iter() {
            *value = Value::from("on");
        }

        assert_eq!(map.get("flag"), Some(&Value::from("on")));
    }
}

mod value_from {
    use super::*;

    #[test]
    fn should_wrap_owned_string_when_constructing_value_string_then_store_owned_string() {
        let source = String::from("hello");

        let value = Value::from(source.clone());

        assert_eq!(value, Value::String(source));
    }

    #[test]
    fn should_clone_str_slice_when_constructing_value_string_then_store_cloned_string() {
        let source = "world";

        let value = Value::from(source);

        assert_eq!(value, Value::String(String::from(source)));
    }
}

mod conversions {
    use super::*;

    #[test]
    fn should_construct_query_map_from_ordered_map_then_preserve_entries() {
        let mut ordered = OrderedMap::default();
        ordered.insert(String::from("id"), Value::from("42"));

        let query_map = QueryMap::from(ordered.clone());

        assert_eq!(query_map.len(), 1);
        assert_eq!(query_map.get("id"), ordered.get("id"));
    }

    #[test]
    fn should_convert_query_map_back_into_ordered_map_then_yield_same_contents() {
        let mut query_map = QueryMap::new();
        query_map.insert(String::from("role"), Value::from("admin"));

        let ordered: OrderedMap<_, _> = query_map.clone().into();

        assert_eq!(ordered.len(), 1);
        assert_eq!(ordered.get("role"), Some(&Value::from("admin")));
        assert!(
            query_map.get("role").is_some(),
            "original map should remain intact"
        );
    }

    #[test]
    fn should_consume_query_map_when_using_from_then_preserve_entries() {
        let mut query_map = QueryMap::new();
        query_map.insert(String::from("token"), Value::from("abc123"));

        let ordered = OrderedMap::from(query_map);

        assert_eq!(ordered.len(), 1);
        assert_eq!(ordered.get("token"), Some(&Value::from("abc123")));
    }
}

mod value_accessors {
    use super::*;

    #[test]
    fn should_return_borrowed_str_when_value_is_string_then_expose_slice() {
        let value = Value::from("access");

        let result = value.as_str();

        assert_eq!(result, Some("access"));
        assert!(value.is_string());
        assert!(!value.is_array());
        assert!(!value.is_object());
    }

    #[test]
    fn should_return_none_when_calling_as_str_on_non_string_then_reject_conversion() {
        let value = Value::Array(vec![]);

        assert!(value.as_str().is_none());
        assert!(!value.is_string());
    }

    #[test]
    fn should_return_slice_when_value_is_array_then_expose_elements() {
        let value = Value::Array(vec![Value::from("a"), Value::from("b")]);

        let slice = value.as_array().expect("array should be exposed as slice");

        assert_eq!(slice.len(), 2);
        assert!(value.is_array());
        assert!(!value.is_object());
    }

    #[test]
    fn should_return_none_when_calling_as_array_on_non_array_then_reject_conversion() {
        let value = Value::from("not-an-array");

        assert!(value.as_array().is_none());
        assert!(!value.is_array());
    }

    #[test]
    fn should_return_map_reference_when_value_is_object_then_expose_entries() {
        let mut map = OrderedMap::default();
        map.insert(String::from("name"), Value::from("Neo"));
        let value = Value::Object(map);

        let object = value
            .as_object()
            .expect("object should be exposed as ordered map");

        assert_eq!(object.get("name"), Some(&Value::from("Neo")));
        assert!(value.is_object());
        assert!(!value.is_string());
        assert!(!value.is_array());
    }

    #[test]
    fn should_return_none_when_calling_as_object_on_non_object_then_reject_conversion() {
        let value = Value::from("plain");

        assert!(value.as_object().is_none());
        assert!(!value.is_object());
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
        let payload = Credentials {
            username: String::from("neo"),
            token: String::from("abc123"),
            active: true,
            roles: vec![String::from("admin"), String::from("operator")],
            nickname: None,
        };

        let map = QueryMap::from_struct(&payload).expect("serialization should succeed");

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

        let map = QueryMap::from_struct(&payload).expect("serialization should succeed");

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
    use crate::SerdeQueryError;
    use serde::{Deserialize, Serialize};

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

        let document: Document = map.to_struct().expect("deserialization should succeed");

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
        let mut map = QueryMap::new();
        map.insert(String::from("name"), Value::from("Morpheus"));
        map.insert(String::from("age"), Value::from("not-a-number"));

        let result = map.to_struct::<Profile>();

        let err = result.expect_err("invalid number should fail");
        assert_matches!(
            err,
            SerdeQueryError::Deserialize(source) => {
                assert!(source.to_string().contains("invalid number"));
            }
        );
    }
}

mod clone_value_into_arena {
    use super::clone_value_into_arena_for_test;
    use super::*;

    #[test]
    fn should_clone_string_into_arena_then_produce_arena_string_value() {
        let arena = ParseArena::new();
        let value = Value::from("matrix");

        let cloned = clone_value_into_arena_for_test(&arena, &value);

        assert_matches!(cloned, ArenaValue::String(text) => {
            assert_eq!(text, "matrix");
        });
    }

    #[test]
    fn should_clone_array_into_arena_then_preserve_element_structure() {
        let arena = ParseArena::new();
        let value = Value::Array(vec![
            Value::from("alpha"),
            Value::Object({
                let mut inner = OrderedMap::default();
                inner.insert(String::from("beta"), Value::from("bravo"));
                inner
            }),
        ]);

        let cloned = clone_value_into_arena_for_test(&arena, &value);

        assert_matches!(cloned, ArenaValue::Seq(items) => {
            assert_eq!(items.len(), 2);
            assert_matches!(&items[0], ArenaValue::String(text) if *text == "alpha");
            assert_matches!(&items[1], ArenaValue::Map { entries, .. } => {
                assert_eq!(entries.len(), 1);
                assert_eq!(entries[0].0, "beta");
                assert_matches!(&entries[0].1, ArenaValue::String(text) if *text == "bravo");
            });
        });
    }

    #[test]
    fn should_clone_object_into_arena_then_build_indexed_entries() {
        let arena = ParseArena::new();
        let mut payload = OrderedMap::default();
        payload.insert(String::from("gamma"), Value::from("3"));
        payload.insert(String::from("delta"), Value::Array(vec![Value::from("1")]));
        let value = Value::Object(payload);

        let cloned = clone_value_into_arena_for_test(&arena, &value);

        assert_matches!(cloned, ArenaValue::Map { entries, index } => {
            assert_eq!(entries.len(), 2);
            assert_eq!(index.len(), 2);

            let &gamma_index = index.get("gamma").expect("gamma key should exist");
            assert_matches!(&entries[gamma_index].1, ArenaValue::String(text) if *text == "3");

            let &delta_index = index.get("delta").expect("delta key should exist");
            assert_matches!(&entries[delta_index].1, ArenaValue::Seq(items) => {
                assert_eq!(items.len(), 1);
                assert_matches!(&items[0], ArenaValue::String(text) if *text == "1");
            });
        });
    }

    #[test]
    fn should_clone_empty_object_into_arena_then_allocate_map_without_entries() {
        let arena = ParseArena::new();
        let value = Value::Object(OrderedMap::default());

        let cloned = clone_value_into_arena_for_test(&arena, &value);

        assert_matches!(cloned, ArenaValue::Map { entries, index } => {
            assert!(entries.is_empty());
            assert!(index.is_empty());
        });
    }
}

mod insert_value_into_arena_map {
    use super::*;
    use crate::SerdeQueryError;

    #[test]
    fn should_convert_duplicate_insertion_into_serde_query_error() {
        let arena = ParseArena::new();
        let mut map = ArenaQueryMap::with_capacity(&arena, 1);
        let value = Value::from("alpha");

        super::insert_value_into_arena_map(&arena, &mut map, "token", &value)
            .expect("first insert should succeed");

        let error = super::insert_value_into_arena_map(&arena, &mut map, "token", &value)
            .expect_err("duplicate insert should return error");

        match error {
            SerdeQueryError::Deserialize(inner) => {
                let message = inner.to_string();
                assert!(message.contains("duplicate field"));
                assert!(message.contains("token"));
            }
            other => panic!("expected deserialize error, got {other:?}"),
        }
        assert_eq!(map.len(), 1, "duplicate insert should not grow map");
    }
}
