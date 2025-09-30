use super::*;

mod parse_options_builder {
    use super::*;

    #[test]
    fn when_building_with_defaults_it_should_succeed() {
        // Arrange
        let builder = ParseOptions::builder();

        // Act
        let options = builder.build().expect("defaults should be valid");

        // Assert
        assert!(!options.space_as_plus);
        assert!(options.max_params.is_none());
        assert!(options.max_length.is_none());
        assert!(options.max_depth.is_none());
    }

    #[test]
    fn when_setting_positive_limits_it_should_store_values() {
        // Arrange
        let builder = ParseOptions::builder();

        // Act
        let options = builder
            .space_as_plus(true)
            .max_params(100)
            .max_length(2048)
            .max_depth(32)
            .build()
            .expect("positive limits should be valid");

        // Assert
        assert!(options.space_as_plus);
        assert_eq!(options.max_params, Some(100));
        assert_eq!(options.max_length, Some(2048));
        assert_eq!(options.max_depth, Some(32));
    }

    #[test]
    fn when_setting_zero_max_params_it_should_fail() {
        // Arrange
        let builder = ParseOptions::builder();

        // Act
        let error = builder
            .max_params(0)
            .build()
            .expect_err("zero max_params should be rejected");

        // Assert
        assert_eq!(error.to_string(), "max_params must be greater than 0 when using the builder");
    }

    #[test]
    fn when_setting_zero_max_length_it_should_fail() {
        // Arrange
        let builder = ParseOptions::builder();

        // Act
        let error = builder
            .max_length(0)
            .build()
            .expect_err("zero max_length should be rejected");

        // Assert
        assert_eq!(error.to_string(), "max_length must be greater than 0 when using the builder");
    }

    #[test]
    fn when_setting_zero_max_depth_it_should_fail() {
        // Arrange
        let builder = ParseOptions::builder();

        // Act
        let error = builder
            .max_depth(0)
            .build()
            .expect_err("zero max_depth should be rejected");

        // Assert
        assert_eq!(error.to_string(), "max_depth must be greater than 0 when using the builder");
    }
}

mod stringify_options_builder {
    use super::*;

    #[test]
    fn when_building_stringify_defaults_it_should_succeed() {
        // Arrange
        let builder = StringifyOptions::builder();

        // Act
        let options = builder.build().expect("defaults should be valid");

        // Assert
        assert!(!options.space_as_plus);
    }

    #[test]
    fn when_enabling_space_as_plus_it_should_store_flag() {
        // Arrange
        let builder = StringifyOptions::builder();

        // Act
        let options = builder
            .space_as_plus(true)
            .build()
            .expect("flag should be valid");

        // Assert
        assert!(options.space_as_plus);
    }
}
