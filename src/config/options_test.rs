use super::*;

mod parse_options_builder {
    use super::*;

    #[test]
    fn should_build_successfully_when_using_defaults_then_return_default_parse_options() {
        let options = ParseOptions::new();
        options.validate().expect("defaults should be valid");

        assert!(!options.space_as_plus);
        assert_eq!(options.duplicate_keys, DuplicateKeyBehavior::Reject);
        assert!(options.max_params.is_none());
        assert!(options.max_length.is_none());
        assert!(options.max_depth.is_none());
    }

    #[test]
    fn should_store_values_when_setting_positive_limits_then_apply_configured_limits() {
        let options = ParseOptions::new()
            .space_as_plus(true)
            .duplicate_keys(DuplicateKeyBehavior::LastWins)
            .max_params(100)
            .max_length(2048)
            .max_depth(32);
        options.validate().expect("positive limits should be valid");

        assert!(options.space_as_plus);
        assert_eq!(options.duplicate_keys, DuplicateKeyBehavior::LastWins);
        assert_eq!(options.max_params, Some(100));
        assert_eq!(options.max_length, Some(2048));
        assert_eq!(options.max_depth, Some(32));
    }

    #[test]
    fn should_fail_when_setting_zero_max_params_then_return_builder_error() {
        let error = ParseOptions::new()
            .max_params(0)
            .validate()
            .expect_err("zero max_params should be rejected");

        assert_eq!(
            error,
            OptionsValidationError::NonZeroRequired {
                field: "max_params"
            }
        );
    }

    #[test]
    fn should_fail_when_setting_zero_max_length_then_return_builder_error() {
        let error = ParseOptions::new()
            .max_length(0)
            .validate()
            .expect_err("zero max_length should be rejected");

        assert_eq!(
            error,
            OptionsValidationError::NonZeroRequired {
                field: "max_length"
            }
        );
    }

    #[test]
    fn should_fail_when_setting_zero_max_depth_then_return_builder_error() {
        let error = ParseOptions::new()
            .max_depth(0)
            .validate()
            .expect_err("zero max_depth should be rejected");

        assert_eq!(
            error,
            OptionsValidationError::NonZeroRequired { field: "max_depth" }
        );
    }
}

mod stringify_options_builder {
    use super::*;

    #[test]
    fn should_build_successfully_when_using_stringify_defaults_then_return_default_stringify_options()
     {
        let options = StringifyOptions::new();
        options.validate().expect("defaults should be valid");

        assert!(!options.space_as_plus);
    }

    #[test]
    fn should_store_flag_when_enabling_space_as_plus_then_set_space_as_plus_true() {
        let options = StringifyOptions::new().space_as_plus(true);
        options.validate().expect("flag should be valid");

        assert!(options.space_as_plus);
    }
}
