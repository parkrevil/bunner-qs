use crate::parsing::ParseError;
use crate::parsing::arena::{ArenaValue, ParseArena};

#[track_caller]
pub fn expect_duplicate_key(error: ParseError, expected_key: &str) {
    match error {
        ParseError::DuplicateRootKey { key } => assert_eq!(key, expected_key),
        ParseError::DuplicateMapEntry { segment, .. } => assert_eq!(segment, expected_key),
        ParseError::DuplicateSequenceIndex { index, .. } => {
            assert_eq!(index.to_string(), expected_key)
        }
        ParseError::InvalidSequenceIndex { segment, .. } => assert_eq!(segment, expected_key),
        ParseError::NestedValueConflict { parent } => assert_eq!(parent, expected_key),
        ParseError::KeyPatternConflict { segment, .. } => assert_eq!(segment, expected_key),
        other => panic!("expected duplicate key error, got {other:?}"),
    }
}

pub fn make_string<'arena>(arena: &'arena ParseArena, value: &str) -> ArenaValue<'arena> {
    ArenaValue::string(arena.alloc_str(value))
}

pub fn make_sequence<'arena>(arena: &'arena ParseArena, items: &[&str]) -> ArenaValue<'arena> {
    let mut sequence = ArenaValue::seq_with_capacity(arena, items.len());
    if let ArenaValue::Seq(values) = &mut sequence {
        for item in items {
            values.push(ArenaValue::string(arena.alloc_str(item)));
        }
    }
    sequence
}
