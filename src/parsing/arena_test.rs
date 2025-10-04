use super::*;
use crate::arena_helpers::{map, map_with_capacity};
use assert_matches::assert_matches;

mod parse_arena_new {
    use super::*;

    #[test]
    fn should_allocate_string_when_using_alloc_str_then_return_same_reference() {
        let arena = ParseArena::new();

        let stored = arena.alloc_str("hello");

        assert_eq!(stored, "hello");
    }

    #[test]
    fn should_return_empty_vector_when_alloc_vec_is_called_then_provide_zero_length_slice() {
        let arena = ParseArena::new();

        let values: ArenaVec<'_, i32> = arena.alloc_vec();

        assert!(values.is_empty());
    }

    #[test]
    fn should_delegate_zero_capacity_to_new_arena_when_with_capacity_called_with_zero_then_return_default_instance()
     {
        let zero = std::hint::black_box(0usize);

        let arena = ParseArena::with_capacity(zero);

        assert_eq!(arena.capacity_hint(), 0);
    }

    #[test]
    fn should_shrink_arena_when_runtime_hint_is_small_then_reallocate_with_new_capacity() {
        let mut arena = ParseArena::with_capacity(512 * 1024);
        let requested = std::hint::black_box(8 * 1024);

        arena.prepare(requested);

        assert_eq!(arena.capacity_hint(), requested);
    }

    #[test]
    fn should_reset_arena_when_prepare_called_with_zero_capacity_then_clear_entries() {
        let mut arena = ParseArena::with_capacity(1024);
        arena.alloc_str("buffered");

        arena.prepare(0);

        assert_eq!(arena.capacity_hint(), 1024);
    }

    #[test]
    fn should_allocate_capacity_hint_when_constructed_with_non_zero_capacity_then_reserve_internal_buffer()
     {
        let arena = ParseArena::with_capacity(4096);

        assert_eq!(arena.capacity_hint(), 4096);
    }

    #[test]
    fn should_reallocate_arena_when_prepare_requests_more_capacity_than_current_then_expand_allocation()
     {
        let mut arena = ParseArena::with_capacity(1024);

        arena.prepare(4096);

        assert_eq!(arena.capacity_hint(), 4096);
    }

    #[test]
    fn should_reset_without_shrinking_when_capacity_below_threshold_and_minimum_is_smaller_then_preserve_existing_capacity()
     {
        let mut arena = ParseArena::with_capacity(64 * 1024);
        arena.alloc_str("primed");

        arena.prepare(32 * 1024);

        assert_eq!(arena.capacity_hint(), 64 * 1024);
        let stored = arena.alloc_str("reset");
        assert_eq!(stored, "reset");
    }
}

mod parse_arena_pooling {
    use super::*;

    #[test]
    fn should_reuse_pooled_arena_when_acquired_multiple_times_then_return_same_instance() {
        {
            let lease = acquire_parse_arena(2048);
            lease.alloc_str("warmup");
            assert_eq!(lease.capacity_hint(), 2048);
        }

        let lease = acquire_parse_arena(0);

        assert_eq!(lease.capacity_hint(), 2048);
    }
}

mod arena_query_map_insert {
    use super::*;

    #[test]
    fn should_store_value_when_inserting_unique_key_then_increase_map_length() {
        let lease = acquire_parse_arena(0);
        let arena: &ParseArena = &lease;
        let mut map = map_with_capacity(arena, 1);
        let value = ArenaValue::string(arena.alloc_str("value"));

        let result = map.try_insert_str(arena, "key", value);

        assert!(result.is_ok());
        assert_eq!(map.len(), 1);
        assert_eq!(map.entries_slice()[0].0, "key");
    }

    #[test]
    fn should_return_error_when_inserting_duplicate_key_then_prevent_overwrite() {
        let lease = acquire_parse_arena(0);
        let arena: &ParseArena = &lease;
        let mut map = map_with_capacity(arena, 1);
        let value = ArenaValue::string(arena.alloc_str("first"));
        map.try_insert_str(arena, "key", value)
            .expect("first insert");
        let duplicate = ArenaValue::string(arena.alloc_str("second"));

        let result = map.try_insert_str(arena, "key", duplicate);

        assert!(result.is_err());
    }
}

mod arena_query_map_iter {
    use super::*;

    #[test]
    fn should_iterate_in_insertion_order_when_entries_exist_then_preserve_sequence_order() {
        let lease = acquire_parse_arena(0);
        let arena: &ParseArena = &lease;
        let mut map = map_with_capacity(arena, 2);
        map.try_insert_str(arena, "first", ArenaValue::string(arena.alloc_str("1")))
            .expect("insert first");
        map.try_insert_str(arena, "second", ArenaValue::string(arena.alloc_str("2")))
            .expect("insert second");

        let collected: Vec<(&str, &str)> = map
            .iter()
            .map(|(key, value)| match value {
                ArenaValue::String(text) => Ok((key, *text)),
                other => Err(other),
            })
            .collect::<Result<_, _>>()
            .expect("entries should store string values");

        assert_eq!(collected, vec![("first", "1"), ("second", "2")]);
    }
}

mod arena_query_map_zero_capacity {
    use super::*;

    #[test]
    fn should_initialize_query_map_without_preallocating_when_capacity_is_zero_then_allocate_empty_map()
     {
        let lease = acquire_parse_arena(0);
        let arena: &ParseArena = &lease;

        let map = map(arena);

        assert!(map.is_empty());
        assert!(!map.contains_key("missing"));
    }
}

mod arena_query_map_get_mut {
    use super::*;

    #[test]
    fn should_store_values_when_mutating_sequence_entry_then_append_new_item() {
        let lease = acquire_parse_arena(0);
        let arena: &ParseArena = &lease;
        let mut map = map_with_capacity(arena, 1);
        let sequence = ArenaValue::seq_with_capacity(arena, 0);
        map.try_insert_str(arena, "items", sequence)
            .expect("insert sequence");

        let entry = map.get_mut("items").expect("sequence entry");
        assert_matches!(entry, ArenaValue::Seq(values) => {
            values.push(ArenaValue::string(arena.alloc_str("one")));
        });

        let stored = map.entries_slice()[0]
            .1
            .as_seq_slice()
            .expect("sequence slice");
        assert_eq!(stored.len(), 1);
        assert_matches!(stored[0], ArenaValue::String("one"));
    }
}

mod arena_value_accessors {
    use super::*;

    #[test]
    fn should_create_empty_map_when_requested_capacity_provided_then_return_empty_entries() {
        let lease = acquire_parse_arena(0);
        let arena: &ParseArena = &lease;

        let value = ArenaValue::map_with_capacity(arena, 8);

        let entries = value.as_map_slice().expect("map slice");
        assert!(entries.is_empty());
    }

    #[test]
    fn should_create_empty_sequence_when_requested_capacity_provided_then_return_empty_entries() {
        let lease = acquire_parse_arena(0);
        let arena: &ParseArena = &lease;

        let value = ArenaValue::seq_with_capacity(arena, 5);

        let entries = value.as_seq_slice().expect("seq slice");
        assert!(entries.is_empty());
    }

    #[test]
    fn should_create_map_using_small_capacity_path_when_capacity_is_low_then_allocate_small_map() {
        let lease = acquire_parse_arena(0);
        let arena: &ParseArena = &lease;

        let value = ArenaValue::map_with_capacity(arena, 2);

        let entries = value.as_map_slice().expect("map slice");
        assert!(entries.is_empty());
    }

    #[test]
    fn should_create_sequence_without_reserve_when_capacity_is_small_then_allocate_small_sequence()
    {
        let lease = acquire_parse_arena(0);
        let arena: &ParseArena = &lease;

        let value = ArenaValue::seq_with_capacity(arena, 2);

        let entries = value.as_seq_slice().expect("seq slice");
        assert!(entries.is_empty());
    }

    #[test]
    fn should_return_none_from_as_seq_slice_when_value_is_map_then_report_absence() {
        let lease = acquire_parse_arena(0);
        let arena: &ParseArena = &lease;

        let value = ArenaValue::map_with_capacity(arena, 1);

        assert!(value.as_seq_slice().is_none());
    }

    #[test]
    fn should_return_none_from_as_map_slice_when_value_is_sequence_then_report_absence() {
        let lease = acquire_parse_arena(0);
        let arena: &ParseArena = &lease;

        let value = ArenaValue::seq_with_capacity(arena, 1);

        assert!(value.as_map_slice().is_none());
    }

    #[test]
    fn should_expose_entries_and_index_when_map_parts_mut_called_on_map_then_return_mutable_parts()
    {
        let lease = acquire_parse_arena(0);
        let arena: &ParseArena = &lease;

        let mut value = ArenaValue::map_with_capacity(arena, 0);

        let (entries, index) = value
            .map_parts_mut()
            .expect("map_parts_mut should return map internals");

        assert!(entries.is_empty());
        assert!(index.is_empty());
    }

    #[test]
    fn should_return_none_from_map_parts_mut_when_value_is_not_map_then_report_absence() {
        let arena = ParseArena::new();
        let mut value = ArenaValue::string(arena.alloc_str("plain"));

        assert!(value.map_parts_mut().is_none());
    }
}

mod arena_value_debug {
    use super::*;

    #[test]
    fn should_format_string_variant_then_display_contents() {
        let arena = ParseArena::new();
        let value = ArenaValue::string(arena.alloc_str("debug"));

        assert_eq!(format!("{:?}", value), "String(\"debug\")");
    }

    #[test]
    fn should_format_sequence_variant_then_render_children() {
        let arena = ParseArena::new();
        let mut sequence = ArenaValue::seq_with_capacity(&arena, 0);
        if let ArenaValue::Seq(items) = &mut sequence {
            items.push(ArenaValue::string(arena.alloc_str("alpha")));
            items.push(ArenaValue::string(arena.alloc_str("beta")));
        } else {
            panic!("expected sequence variant");
        }

        let formatted = format!("{:?}", sequence);
        assert!(formatted.starts_with("Seq(["));
        assert!(formatted.contains("String(\"alpha\")"));
        assert!(formatted.contains("String(\"beta\")"));
    }

    #[test]
    fn should_format_map_variant_then_render_entries_field() {
        let arena = ParseArena::new();
        let mut map_value = ArenaValue::map(&arena);
        if let ArenaValue::Map { entries, index } = &mut map_value {
            let key = arena.alloc_str("name");
            let value = ArenaValue::string(arena.alloc_str("neo"));
            entries.push((key, value));
            index.insert(key, 0);
        } else {
            panic!("expected map variant");
        }

        let formatted = format!("{:?}", map_value);
        assert!(formatted.starts_with("Map {"));
        assert!(formatted.contains("entries"));
        assert!(formatted.contains("name"));
        assert!(formatted.contains("neo"));
    }
}
