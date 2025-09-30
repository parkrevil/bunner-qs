use bunner_qs::parsing::arena::ArenaValue;
use bunner_qs::parsing::builder::with_arena_query_map;
use bunner_qs::{ParseError, ParseOptions};

mod with_arena_query_map_tests {
    use super::*;

    #[test]
    fn when_query_contains_pairs_it_should_store_strings_in_map() {
        // Arrange
        let options = ParseOptions::default();

        // Act
        with_arena_query_map("name=alice&flag", 0, &options, |_, arena_map| {
            // Assert
            assert_eq!(arena_map.len(), 2);

            let entries = arena_map.entries_slice();
            assert!(entries.iter().any(|(key, value)| {
                *key == "name" && matches!(*value, ArenaValue::String(text) if text == "alice")
            }));
            assert!(entries.iter().any(|(key, value)| {
                *key == "flag" && matches!(*value, ArenaValue::String(text) if text.is_empty())
            }));

            Ok(())
        })
        .expect("builder should parse into arena map");
    }

    #[test]
    fn when_parameter_limit_is_exceeded_it_should_return_error() {
        // Arrange
        let options = ParseOptions::builder()
            .max_params(1)
            .build()
            .expect("builder should construct options");

        // Act
        let error = with_arena_query_map("a=1&b=2", 0, &options, |_, _| Ok(()))
            .expect_err("expected parameter limit error");

        // Assert
        match error {
            ParseError::TooManyParameters { limit, actual } => {
                assert_eq!(limit, 1);
                assert_eq!(actual, 2);
            }
            other => panic!("unexpected error: {other:?}"),
        }
    }
}
