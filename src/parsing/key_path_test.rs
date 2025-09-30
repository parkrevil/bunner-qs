use super::{duplicate_key_label, estimate_param_capacity, validate_brackets};
use crate::parsing::ParseError;

mod validate_brackets {
    use super::*;

    #[test]
    fn when_brackets_are_balanced_it_returns_ok() {
        // Arrange
        let key = "user[address][street]";

        // Act
        let result = validate_brackets(key, Some(3));

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn when_closing_bracket_appears_without_opening_it_returns_error() {
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
    fn when_open_brackets_remain_unmatched_it_returns_error() {
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
    fn when_total_pairs_exceed_limit_it_reports_depth_exceeded() {
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
    fn when_called_it_returns_owned_copy_of_input() {
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
    fn when_input_is_empty_it_returns_zero() {
        // Arrange
        let query = "";

        // Act
        let capacity = estimate_param_capacity(query);

        // Assert
        assert_eq!(capacity, 0);
    }

    #[test]
    fn when_input_has_multiple_parameters_it_counts_them() {
        // Arrange
        let query = "a=1&b=2&c=3";

        // Act
        let capacity = estimate_param_capacity(query);

        // Assert
        assert_eq!(capacity, 3);
    }

    #[test]
    fn when_input_contains_trailing_ampersand_it_accounts_for_empty_parameter() {
        // Arrange
        let query = "token=&";

        // Act
        let capacity = estimate_param_capacity(query);

        // Assert
        assert_eq!(capacity, 2);
    }
}
