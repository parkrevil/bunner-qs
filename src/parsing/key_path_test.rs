use super::{duplicate_key_label, estimate_param_capacity, validate_brackets};
use crate::parsing::ParseError;

mod validate_brackets {
    use super::*;

    #[test]
    fn should_return_ok_when_brackets_are_balanced_then_allow_nested_segments() {
        // Arrange
        let key = "user[address][street]";

        // Act
        let result = validate_brackets(key, Some(3));

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn should_return_error_when_closing_bracket_is_unmatched_then_return_unmatched_bracket_error() {
        // Arrange
        let key = "user]";

        // Act
        let result = validate_brackets(key, None);

        // Assert
        assert!(matches!(
            result,
            Err(ParseError::UnmatchedBracket { key: ref error_key }) if error_key == key
        ));
    }

    #[test]
    fn should_return_error_when_open_brackets_are_unmatched_then_report_unmatched_open_bracket() {
        // Arrange
        let key = "user[address";

        // Act
        let result = validate_brackets(key, None);

        // Assert
        assert!(matches!(
            result,
            Err(ParseError::UnmatchedBracket { key: ref error_key }) if error_key == key
        ));
    }

    #[test]
    fn should_report_depth_exceeded_when_pair_limit_is_exceeded_then_return_depth_exceeded_error() {
        // Arrange
        let key = "a[][][]";

        // Act
        let result = validate_brackets(key, Some(2));

        // Assert
        assert!(matches!(
            result,
            Err(ParseError::DepthExceeded {
                key: ref error_key,
                limit
            }) if error_key == key && limit == 2
        ));
    }
}

mod duplicate_key_label {
    use super::*;

    #[test]
    fn should_clone_input_when_duplicate_key_label_requested_then_return_owned_string() {
        // Arrange
        let key = "session";

        // Act
        let mut cloned = duplicate_key_label(key);

        // Assert
        assert_eq!(cloned, key);

        // Ensure the result owns its data by mutating it
        cloned.push_str("_id");
        assert_eq!(key, "session");
        assert_eq!(cloned, "session_id");
    }
}

mod estimate_param_capacity {
    use super::*;

    #[test]
    fn should_count_zero_parameters_when_input_is_empty_then_return_zero_capacity() {
        // Arrange
        let query = "";

        // Act
        let capacity = estimate_param_capacity(query);

        // Assert
        assert_eq!(capacity, 0);
    }

    #[test]
    fn should_count_multiple_parameters_when_query_contains_three_entries_then_return_three_capacity()
     {
        // Arrange
        let query = "a=1&b=2&c=3";

        // Act
        let capacity = estimate_param_capacity(query);

        // Assert
        assert_eq!(capacity, 3);
    }

    #[test]
    fn should_count_trailing_empty_parameter_when_query_has_trailing_separator_then_include_trailing_capacity()
     {
        // Arrange
        let query = "token=&";

        // Act
        let capacity = estimate_param_capacity(query);

        // Assert
        assert_eq!(capacity, 2);
    }
}
