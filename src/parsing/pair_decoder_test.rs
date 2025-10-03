use super::*;
use crate::parsing::ParseError;
use std::borrow::Cow;

mod decode_pair {
    use super::*;

    #[test]
    fn should_return_borrowed_components_when_plain_ascii_then_avoid_allocation() {
        let options = ParseOptions::default();
        let mut scratch = Vec::new();

        let (key, value) =
            decode_pair("foo", "bar", 0, 4, &options, &mut scratch).expect("decode succeeds");

        assert!(matches!(key, Cow::Borrowed("foo")));
        assert!(matches!(value, Cow::Borrowed("bar")));
        assert!(scratch.is_empty());
    }

    #[test]
    fn should_decode_plus_signs_when_space_as_plus_enabled_then_replace_with_spaces() {
        let options = ParseOptions {
            space_as_plus: true,
            ..ParseOptions::default()
        };
        let mut scratch = Vec::new();

        let (key, value) = decode_pair("hello+world", "value+here", 0, 17, &options, &mut scratch)
            .expect("decode succeeds");

        assert_eq!(key.as_ref(), "hello world");
        assert_eq!(value.as_ref(), "value here");
    }

    #[test]
    fn should_return_error_when_brackets_unmatched_then_return_unmatched_bracket_error() {
        let options = ParseOptions::default();
        let mut scratch = Vec::new();

        let error =
            decode_pair("foo[", "bar", 0, 4, &options, &mut scratch).expect_err("decode fails");

        assert!(matches!(error, ParseError::UnmatchedBracket { ref key } if key == "foo["));
    }

    #[test]
    fn should_return_invalid_percent_error_when_key_percent_encoding_is_invalid_then_report_index()
    {
        let options = ParseOptions::default();
        let mut scratch = Vec::new();

        let error = decode_pair("%2Z", "value", 3, 9, &options, &mut scratch)
            .expect_err("invalid key percent encoding should fail");

        match error {
            ParseError::InvalidPercentEncoding { index } => assert_eq!(index, 3),
            other => panic!("expected InvalidPercentEncoding error, got {other:?}"),
        }
    }
}
