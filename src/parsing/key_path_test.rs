use super::{duplicate_key_label, estimate_param_capacity, validate_brackets};
use crate::parsing::errors::ParseError;
use assert_matches::assert_matches;

mod validate_brackets {
    use super::*;

    #[test]
    fn should_return_ok_when_brackets_are_balanced_then_allow_nested_segments() {
        let key = "user[address][street]";

        let result = validate_brackets(key, Some(3));

        assert!(result.is_ok());
    }

    #[test]
    fn should_return_error_when_closing_bracket_is_unmatched_then_return_unmatched_bracket_error() {
        let key = "user]";

        let result = validate_brackets(key, None);

        assert_matches!(
            result,
            Err(ParseError::UnmatchedBracket { key: ref error_key }) if error_key == key
        );
    }

    #[test]
    fn should_return_error_when_open_brackets_are_unmatched_then_report_unmatched_open_bracket() {
        let key = "user[address";

        let result = validate_brackets(key, None);

        assert_matches!(
            result,
            Err(ParseError::UnmatchedBracket { key: ref error_key }) if error_key == key
        );
    }

    #[test]
    fn should_report_depth_exceeded_when_pair_limit_is_exceeded_then_return_depth_exceeded_error() {
        let key = "a[][][]";

        let result = validate_brackets(key, Some(2));

        assert_matches!(
            result,
            Err(ParseError::DepthExceeded {
                key: ref error_key,
                limit
            }) if error_key == key && limit == 2
        );
    }
}

mod duplicate_key_label {
    use super::*;

    #[test]
    fn should_clone_input_when_duplicate_key_label_requested_then_return_owned_string() {
        let key = "session";

        let mut cloned = duplicate_key_label(key);

        assert_eq!(cloned, key);

        cloned.push_str("_id");
        assert_eq!(key, "session");
        assert_eq!(cloned, "session_id");
    }
}

mod estimate_param_capacity {
    use super::*;

    #[test]
    fn should_count_zero_parameters_when_input_is_empty_then_return_zero_capacity() {
        let query = "";

        let capacity = estimate_param_capacity(query);

        assert_eq!(capacity, 0);
    }

    #[test]
    fn should_count_multiple_parameters_when_query_contains_three_entries_then_return_three_capacity()
     {
        let query = "a=1&b=2&c=3";

        let capacity = estimate_param_capacity(query);

        assert_eq!(capacity, 3);
    }

    #[test]
    fn should_count_trailing_empty_parameter_when_query_has_trailing_separator_then_include_trailing_capacity()
     {
        let query = "token=&";

        let capacity = estimate_param_capacity(query);

        assert_eq!(capacity, 2);
    }
}
