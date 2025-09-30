use super::*;
use crate::config::ParseOptions;
use crate::parsing::arena::ArenaValue;
use crate::parsing::ParseError;

mod with_arena_query_map {
    use super::*;

    #[test]
    fn populates_map_from_unique_pairs() {
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
    fn returns_error_when_duplicate_key_appears() {
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
    fn returns_too_many_parameters_when_limit_exceeded() {
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
}
