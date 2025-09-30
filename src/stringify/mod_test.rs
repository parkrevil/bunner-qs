use super::{SerdeStringifyError, StringifyError, StringifyOptions, stringify, stringify_with};
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

mod stringify_tests {
    use super::*;

    #[test]
    fn when_stringify_is_called_it_should_use_default_options() {
        // Arrange
        let profile = make_profile();

        // Act
        let result = stringify(&profile).expect("stringify should succeed");

        // Assert
        assert_eq!(result, "name=Alice&city=Seattle");
    }

    #[test]
    fn when_stringify_encounters_control_characters_it_should_return_error() {
        // Arrange
        let message = make_message("line1\nline2");

        // Act
        let error = stringify(&message).expect_err("control characters should fail");

        // Assert
        match error {
            SerdeStringifyError::Stringify(StringifyError::InvalidValue { key }) => {
                assert_eq!(key, "body")
            }
            other => panic!("expected stringify error, got {other:?}"),
        }
    }
}

mod stringify_with_tests {
    use super::*;

    #[test]
    fn when_stringify_with_has_space_as_plus_enabled_it_should_replace_spaces() {
        // Arrange
        let message = make_message("hello world");
        let options = StringifyOptions {
            space_as_plus: true,
        };

        // Act
        let result = stringify_with(&message, &options).expect("stringify_with should succeed");

        // Assert
        assert_eq!(result, "body=hello+world");
    }
}
