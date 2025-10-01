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
