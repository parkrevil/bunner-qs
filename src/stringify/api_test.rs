use crate::{SerdeStringifyError, StringifyError, StringifyOptions};
use serde::Serialize;

#[derive(Serialize)]
struct Profile<'a> {
    name: &'a str,
    city: &'a str,
}

#[derive(Serialize)]
struct Message<'a> {
    body: &'a str,
}

fn make_profile() -> Profile<'static> {
    Profile {
        name: "Alice",
        city: "Seattle",
    }
}

fn make_message(body: &'static str) -> Message<'static> {
    Message { body }
}

mod stringify {
    use super::*;

    #[test]
    fn should_use_default_options_when_stringifying_struct_then_return_default_encoding() {
        // Arrange
        let profile = make_profile();

        // Act
        let result = crate::stringify(&profile).expect("stringify should succeed");

        // Assert
        assert_eq!(result, "name=Alice&city=Seattle");
    }

    #[test]
    fn should_return_error_when_value_contains_control_characters_then_return_invalid_value_error() {
        // Arrange
        let message = make_message("line1\nline2");

        // Act
        let error = crate::stringify(&message).expect_err("control characters should fail");

        // Assert
        match error {
            SerdeStringifyError::Stringify(StringifyError::InvalidValue { key }) => {
                assert_eq!(key, "body")
            }
            other => panic!("expected stringify error, got {other:?}"),
        }
    }
}

mod stringify_with {
    use super::*;

    #[test]
    fn should_encode_spaces_as_plus_when_option_is_enabled_then_replace_spaces_with_plus() {
        // Arrange
        let message = make_message("hello world");
        let options = StringifyOptions {
            space_as_plus: true,
        };

        // Act
        let result =
            crate::stringify_with(&message, &options).expect("stringify_with should succeed");

        // Assert
        assert_eq!(result, "body=hello+world");
    }
}
