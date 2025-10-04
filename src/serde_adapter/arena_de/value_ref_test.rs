use super::*;
use crate::parsing::arena::{ArenaValue, ParseArena};
use assert_matches::assert_matches;

fn string_ref(reference: ArenaValueRef<'_>) -> Option<&str> {
    match reference {
        ArenaValueRef::String(value) => Some(value),
        _ => None,
    }
}

fn seq_slice(reference: ArenaValueRef<'_>) -> Option<&[ArenaValue<'_>]> {
    match reference {
        ArenaValueRef::Seq(slice) => Some(slice),
        _ => None,
    }
}

fn map_slice(reference: ArenaValueRef<'_>) -> Option<&[(&str, ArenaValue<'_>)]> {
    match reference {
        ArenaValueRef::Map(entries) => Some(entries),
        _ => None,
    }
}

mod from_value {
    use super::*;

    #[test]
    fn should_return_string_variant_when_wrapping_string_value_then_return_string_reference() {
        let value = ArenaValue::string("hello");

        let reference = ArenaValueRef::from_value(&value);

        assert_eq!(string_ref(reference), Some("hello"));
    }

    #[test]
    fn should_borrow_sequence_slice_when_wrapping_sequence_value_then_expose_sequence_slice() {
        let arena = ParseArena::new();
        let mut items = arena.alloc_vec();
        items.push(ArenaValue::string(arena.alloc_str("zero")));
        items.push(ArenaValue::string(arena.alloc_str("one")));
        let value = ArenaValue::Seq(items);

        let reference = ArenaValueRef::from_value(&value);

        let slice = seq_slice(reference).expect("expected sequence variant");
        assert_eq!(slice.len(), 2);
        assert_matches!(slice[0], ArenaValue::String("zero"));
        assert_matches!(slice[1], ArenaValue::String("one"));
    }

    #[test]
    fn should_borrow_entry_slice_when_wrapping_map_value_then_expose_map_slice() {
        let arena = ParseArena::new();
        let mut entries = arena.alloc_vec();
        entries.push((
            arena.alloc_str("name"),
            ArenaValue::string(arena.alloc_str("Jane")),
        ));
        entries.push((
            arena.alloc_str("city"),
            ArenaValue::string(arena.alloc_str("Seoul")),
        ));
        let value = ArenaValue::Map {
            entries,
            index: Default::default(),
        };

        let reference = ArenaValueRef::from_value(&value);

        let slice = map_slice(reference).expect("expected map variant");
        assert_eq!(slice.len(), 2);
        assert_eq!(slice[0].0, "name");
        assert_matches!(slice[0].1, ArenaValue::String("Jane"));
        assert_eq!(slice[1].0, "city");
        assert_matches!(slice[1].1, ArenaValue::String("Seoul"));
    }
}
