use super::*;
use crate::config::{DuplicateKeyBehavior, ParseOptions};
use crate::parsing::ParseError;
use crate::parsing::arena::ArenaValue;

mod with_arena_query_map {
    use super::*;

    #[test]
    fn should_populate_map_with_unique_pairs_when_all_keys_are_unique_then_store_all_pairs() {
        // Arrange
        let trimmed = "foo=bar&baz=qux";
        let options = ParseOptions::default();

        // Act
        let result = with_arena_query_map(trimmed, 0, &options, |_, map| {
            let entries = map.entries_slice();
            assert_eq!(entries.len(), 2);

            let (first_key, first_value) = &entries[0];
            assert_eq!(*first_key, "foo");
            match first_value {
                ArenaValue::String(value) => assert_eq!(*value, "bar"),
                _ => panic!("expected string value"),
            }

            let (second_key, second_value) = &entries[1];
            assert_eq!(*second_key, "baz");
            match second_value {
                ArenaValue::String(value) => assert_eq!(*value, "qux"),
                _ => panic!("expected string value"),
            }

            Ok(())
        });

        // Assert
        result.expect("unique pairs should parse");
    }

    #[test]
    fn should_return_error_when_duplicate_key_appears_then_include_conflicting_key() {
        // Arrange
        let trimmed = "foo=one&foo=two";
        let options = ParseOptions::default();

        // Act
        let error =
            with_arena_query_map(trimmed, 0, &options, |_, _| Ok(())).expect_err("duplicate key");

        // Assert
        match error {
            ParseError::DuplicateKey { key } => assert_eq!(key, "foo"),
            other => panic!("expected duplicate key error, got {other:?}"),
        }
    }

    #[test]
    fn should_allow_duplicate_keys_when_first_wins_enabled_then_preserve_initial_value() {
        // Arrange
        let trimmed = "foo=one&foo=two";
        let options = ParseOptions::builder()
            .duplicate_keys(DuplicateKeyBehavior::FirstWins)
            .build()
            .expect("builder should succeed");

        // Act
        let result = with_arena_query_map(trimmed, 0, &options, |_, map| {
            let entries = map.entries_slice();
            assert_eq!(entries.len(), 1);
            let (key, value) = &entries[0];
            assert_eq!(*key, "foo");
            match value {
                ArenaValue::String(text) => assert_eq!(*text, "one"),
                _ => panic!("expected string value"),
            }
            Ok(())
        });

        // Assert
        result.expect("first wins should allow duplicates");
    }

    #[test]
    fn should_overwrite_duplicate_keys_when_last_wins_enabled_then_store_latest_value() {
        // Arrange
        let trimmed = "foo=one&foo=two";
        let options = ParseOptions::builder()
            .duplicate_keys(DuplicateKeyBehavior::LastWins)
            .build()
            .expect("builder should succeed");

        // Act
        let result = with_arena_query_map(trimmed, 0, &options, |_, map| {
            let entries = map.entries_slice();
            assert_eq!(entries.len(), 1);
            let (key, value) = &entries[0];
            assert_eq!(*key, "foo");
            match value {
                ArenaValue::String(text) => assert_eq!(*text, "two"),
                _ => panic!("expected string value"),
            }
            Ok(())
        });

        // Assert
        result.expect("last wins should replace duplicate values");
    }

    #[test]
    fn should_return_too_many_parameters_when_parameter_limit_exceeded_then_report_limit_and_actual()
     {
        // Arrange
        let options = ParseOptions::builder()
            .max_params(1)
            .build()
            .expect("builder should succeed");

        // Act
        let error =
            with_arena_query_map("a=1&b=2", 0, &options, |_, _| Ok(())).expect_err("param limit");

        // Assert
        match error {
            ParseError::TooManyParameters { limit, actual } => {
                assert_eq!(limit, 1);
                assert_eq!(actual, 2);
            }
            other => panic!("expected TooManyParameters error, got {other:?}"),
        }
    }

    #[test]
    fn should_decode_plus_signs_when_space_as_plus_enabled_then_convert_to_spaces() {
        let options = ParseOptions::builder()
            .space_as_plus(true)
            .build()
            .expect("builder should succeed");

        let result = with_arena_query_map("hello+world=value+here", 0, &options, |_, map| {
            let entries = map.entries_slice();
            assert_eq!(entries.len(), 1);
            let (key, value) = &entries[0];
            assert_eq!(*key, "hello world");
            match value {
                ArenaValue::String(text) => assert_eq!(*text, "value here"),
                _ => panic!("expected string value"),
            }
            Ok(())
        });

        result.expect("space-as-plus decoding should succeed");
    }

    #[test]
    fn should_report_unmatched_bracket_error_when_brackets_are_unbalanced_then_return_parse_error()
    {
        let options = ParseOptions::default();

        let error = with_arena_query_map("foo[=bar", 0, &options, |_, _| Ok(()))
            .expect_err("unmatched bracket should error");

        assert!(matches!(
            error,
            ParseError::UnmatchedBracket { ref key } if key == "foo["
        ));
    }
}

mod parse_segments_into_map {
    use super::*;

    #[test]
    fn should_parse_segments_into_map_when_query_contains_multiple_pairs_then_populate_map() {
        // Arrange
        let arena = ParseArena::new();
        let mut map = ArenaQueryMap::with_capacity(&arena, 2);
        let mut pattern_state = acquire_pattern_state();
        let options = ParseOptions::default();
        let trimmed = "foo=bar&baz=qux";
        let mut scratch = Vec::new();

        // Act
        {
            let mut context = ParseContext {
                arena: &arena,
                arena_map: &mut map,
                pattern_state: &mut pattern_state,
                options: &options,
                trimmed,
                offset: 0,
                decode_scratch: &mut scratch,
                pairs: 0,
            };

            parse_segments_into_map(&mut context, trimmed.as_bytes())
                .expect("parse should succeed");
        }

        // Assert
        let entries = map.entries_slice();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].0, "foo");
        assert_eq!(entries[1].0, "baz");
    }

    #[test]
    fn should_return_too_many_parameters_error_when_pairs_exceed_limit_then_report_counts() {
        // Arrange
        let arena = ParseArena::new();
        let mut map = ArenaQueryMap::with_capacity(&arena, 4);
        let mut pattern_state = acquire_pattern_state();
        let options = ParseOptions::builder()
            .max_params(1)
            .build()
            .expect("builder should succeed");
        let trimmed = "a=1&b=2";
        let mut scratch = Vec::new();

        // Act
        let error = {
            let mut context = ParseContext {
                arena: &arena,
                arena_map: &mut map,
                pattern_state: &mut pattern_state,
                options: &options,
                trimmed,
                offset: 0,
                decode_scratch: &mut scratch,
                pairs: 0,
            };

            parse_segments_into_map(&mut context, trimmed.as_bytes())
                .expect_err("limit should trigger error")
        };

        // Assert
        match error {
            ParseError::TooManyParameters { limit, actual } => {
                assert_eq!(limit, 1);
                assert_eq!(actual, 2);
            }
            other => panic!("expected TooManyParameters error, got {other:?}"),
        }
    }
}
