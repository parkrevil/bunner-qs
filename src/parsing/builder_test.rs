use super::*;
use crate::arena_helpers::map_with_capacity;
use crate::config::{DuplicateKeyBehavior, ParseOptions};
use crate::parsing::ParseError;
use crate::parsing::arena::ArenaValue;

mod with_arena_query_map {
    use super::*;

    #[test]
    fn should_populate_map_with_unique_pairs_when_all_keys_are_unique_then_store_all_pairs() {
        let trimmed = "foo=bar&baz=qux";
        let options = ParseOptions::default();

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

        result.expect("unique pairs should parse");
    }

    #[test]
    fn should_return_error_when_duplicate_key_appears_then_include_conflicting_key() {
        let trimmed = "foo=one&foo=two";
        let options = ParseOptions::default();

        let error =
            with_arena_query_map(trimmed, 0, &options, |_, _| Ok(())).expect_err("duplicate key");

        match error {
            ParseError::DuplicateKey { key } => assert_eq!(key, "foo"),
            other => panic!("expected duplicate key error, got {other:?}"),
        }
    }

    #[test]
    fn should_allow_duplicate_keys_when_first_wins_enabled_then_preserve_initial_value() {
        let trimmed = "foo=one&foo=two";
        let options = ParseOptions::builder()
            .duplicate_keys(DuplicateKeyBehavior::FirstWins)
            .build()
            .expect("builder should succeed");

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

        result.expect("first wins should allow duplicates");
    }

    #[test]
    fn should_overwrite_duplicate_keys_when_last_wins_enabled_then_store_latest_value() {
        let trimmed = "foo=one&foo=two";
        let options = ParseOptions::builder()
            .duplicate_keys(DuplicateKeyBehavior::LastWins)
            .build()
            .expect("builder should succeed");

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

        result.expect("last wins should replace duplicate values");
    }

    #[test]
    fn should_return_too_many_parameters_when_parameter_limit_exceeded_then_report_limit_and_actual()
     {
        let options = ParseOptions::builder()
            .max_params(1)
            .build()
            .expect("builder should succeed");

        let error =
            with_arena_query_map("a=1&b=2", 0, &options, |_, _| Ok(())).expect_err("param limit");

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

    #[test]
    fn should_ignore_trailing_ampersand_when_query_ends_with_separator_then_parse_existing_pairs() {
        let options = ParseOptions::default();

        let result = with_arena_query_map("foo=bar&", 0, &options, |_, map| {
            let entries = map.entries_slice();
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].0, "foo");
            match &entries[0].1 {
                ArenaValue::String(value) => assert_eq!(*value, "bar"),
                _ => panic!("expected string value"),
            }
            Ok(())
        });

        result.expect("trailing separator should be ignored");
    }

    #[test]
    fn should_return_invalid_percent_error_when_value_contains_non_hex_digits_then_report_index() {
        let options = ParseOptions::default();

        let error = with_arena_query_map("foo=%GG", 0, &options, |_, _| Ok(()))
            .expect_err("invalid percent encoding should error");

        match error {
            ParseError::InvalidPercentEncoding { index } => assert_eq!(index, 4),
            other => panic!("expected InvalidPercentEncoding error, got {other:?}"),
        }
    }

    #[test]
    fn should_store_key_without_value_when_equals_missing_then_use_empty_value() {
        let options = ParseOptions::default();
        let trimmed = "flag";

        let result = with_arena_query_map(trimmed, 12, &options, |_, map| {
            let entries = map.entries_slice();
            assert_eq!(entries.len(), 1);
            let (key, value) = &entries[0];
            assert_eq!(*key, "flag");
            match value {
                ArenaValue::String(text) => assert!(text.is_empty()),
                _ => panic!("expected string value"),
            }
            Ok(())
        });

        result.expect("missing equals should produce empty value");
    }

    #[test]
    fn should_skip_empty_segments_when_multiple_separators_appear_consecutively() {
        let options = ParseOptions::default();
        let trimmed = "&&foo=bar";

        let result = with_arena_query_map(trimmed, 0, &options, |_, map| {
            let entries = map.entries_slice();
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].0, "foo");
            match &entries[0].1 {
                ArenaValue::String(value) => assert_eq!(*value, "bar"),
                _ => panic!("expected string"),
            }
            Ok(())
        });

        result.expect("consecutive separators should be ignored");
    }

    #[test]
    fn should_include_additional_equals_in_value_when_present_then_keep_full_segment() {
        let options = ParseOptions::default();
        let trimmed = "token==value";

        let result = with_arena_query_map(trimmed, 3, &options, |_, map| {
            let entries = map.entries_slice();
            assert_eq!(entries.len(), 1);
            let (key, value) = &entries[0];
            assert_eq!(*key, "token");
            match value {
                ArenaValue::String(text) => assert_eq!(*text, "=value"),
                _ => panic!("expected string value"),
            }
            Ok(())
        });

        result.expect("additional equals should remain in value");
    }
}

mod parse_segments_into_map {
    use super::*;

    #[test]
    fn should_parse_segments_into_map_when_query_contains_multiple_pairs_then_populate_map() {
        let arena = ParseArena::new();
        let mut map = map_with_capacity(&arena, 2);
        let mut pattern_state = acquire_pattern_state();
        let options = ParseOptions::default();
        let trimmed = "foo=bar&baz=qux";
        let mut scratch = Vec::new();

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

        let entries = map.entries_slice();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].0, "foo");
        assert_eq!(entries[1].0, "baz");
    }

    #[test]
    fn should_return_too_many_parameters_error_when_pairs_exceed_limit_then_report_counts() {
        let arena = ParseArena::new();
        let mut map = map_with_capacity(&arena, 4);
        let mut pattern_state = acquire_pattern_state();
        let options = ParseOptions::builder()
            .max_params(1)
            .build()
            .expect("builder should succeed");
        let trimmed = "a=1&b=2";
        let mut scratch = Vec::new();

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

        match error {
            ParseError::TooManyParameters { limit, actual } => {
                assert_eq!(limit, 1);
                assert_eq!(actual, 2);
            }
            other => panic!("expected TooManyParameters error, got {other:?}"),
        }
    }
}
