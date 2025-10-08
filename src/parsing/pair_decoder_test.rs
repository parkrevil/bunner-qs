use super::*;
use crate::parsing::errors::ParseError;
use assert_matches::assert_matches;
use std::borrow::Cow;

mod decode_pair {
    use super::*;

    #[test]
    fn should_return_borrowed_components_when_plain_ascii_then_avoid_allocation() {
        let options = ParseOptions::default();
        let mut scratch = Vec::new();

        let (key, value) =
            decode_pair("foo", "bar", 0, 4, &options, &mut scratch).expect("decode succeeds");

        assert_matches!(key, Cow::Borrowed("foo"));
        assert_matches!(value, Cow::Borrowed("bar"));
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

        assert_matches!(
            error,
            ParseError::UnmatchedBracket { ref key } if key == "foo["
        );
    }

    #[test]
    fn should_return_invalid_percent_error_when_key_percent_encoding_is_invalid_then_report_index()
    {
        let options = ParseOptions::default();
        let mut scratch = Vec::new();

        let error = decode_pair("%2Z", "value", 3, 9, &options, &mut scratch)
            .expect_err("invalid key percent encoding should fail");

        assert_matches!(
            error,
            ParseError::InvalidPercentEncoding { index } => {
                assert_eq!(index, 3);
            }
        );
    }

    #[test]
    fn should_return_depth_exceeded_error_when_bracket_depth_exceeds_limit_then_report_limit() {
        let options = ParseOptions::new().max_depth(1);
        options.validate().expect("configuration should succeed");
        let mut scratch = Vec::new();
        let key = "user[address][city]";

        let error = decode_pair(key, "seattle", 0, key.len() + 1, &options, &mut scratch)
            .expect_err("depth should exceed limit");

        assert_matches!(
            error,
            ParseError::DepthExceeded { ref key, limit }
                if key == "user[address][city]" && limit == 1
        );
    }

    #[test]
    fn should_return_invalid_percent_error_when_value_percent_encoding_is_invalid_then_report_index()
     {
        let options = ParseOptions::default();
        let mut scratch = Vec::new();

        let error = decode_pair("flag", "%GG", 0, 5, &options, &mut scratch)
            .expect_err("invalid value percent encoding should fail");

        assert_matches!(
            error,
            ParseError::InvalidPercentEncoding { index } if index == 5
        );
    }

    #[test]
    fn should_return_owned_value_when_percent_decoding_allocates_then_produce_owned_value() {
        let options = ParseOptions::default();
        let mut scratch = Vec::new();

        let (key, value) = decode_pair("token", "caf%25C3%25A9", 0, 6, &options, &mut scratch)
            .expect("percent-decoded value should succeed");

        assert_matches!(key, Cow::Borrowed("token"));
        assert_matches!(value, Cow::Owned(ref owned) if owned == "caf%C3%A9");
    }
}
