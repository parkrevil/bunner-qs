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
    fn uses_default_options_by_default() {
        // Arrange
        let profile = make_profile();

        // Act
        let result = crate::stringify(&profile).expect("stringify should succeed");

        // Assert
        assert_eq!(result, "name=Alice&city=Seattle");
    }

    #[test]
    fn returns_error_on_control_characters() {
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
    fn encodes_spaces_as_plus_when_enabled() {
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
