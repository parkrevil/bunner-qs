use super::{duplicate_key_label, estimate_param_capacity, validate_brackets};
use crate::parsing::ParseError;

mod validate_brackets {
    use super::*;

    #[test]
    fn returns_ok_for_balanced_brackets() {
        // Arrange
        let key = "user[address][street]";

        // Act
        let result = validate_brackets(key, Some(3));

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn returns_error_for_unmatched_closing_bracket() {
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
    fn returns_error_for_unmatched_open_brackets() {
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
    fn reports_depth_exceeded_when_pair_limit_exceeded() {
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
    fn clones_input_when_requested() {
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
    fn counts_zero_parameters_for_empty_input() {
        // Arrange
        let query = "";

        // Act
        let capacity = estimate_param_capacity(query);

        // Assert
        assert_eq!(capacity, 0);
    }

    #[test]
    fn counts_multiple_parameters_correctly() {
        // Arrange
        let query = "a=1&b=2&c=3";

        // Act
        let capacity = estimate_param_capacity(query);

        // Assert
        assert_eq!(capacity, 3);
    }

    #[test]
    fn counts_trailing_empty_parameter() {
        // Arrange
        let query = "token=&";

        // Act
        let capacity = estimate_param_capacity(query);

        // Assert
        assert_eq!(capacity, 2);
    }
}
