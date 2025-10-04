use super::{StringifyError, StringifyOptions, stringify, stringify_with};
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
        let profile = make_profile();

        let result = stringify(&profile).expect("stringify should succeed");

        assert_eq!(result, "name=Alice&city=Seattle");
    }

    #[test]
    fn should_return_error_when_value_contains_control_characters_then_return_invalid_value_error() {
        let message = make_message("line1\nline2");

        let error = stringify(&message).expect_err("control characters should fail");

        match error {
            StringifyError::InvalidValue { key } => assert_eq!(key, "body"),
            other => panic!("expected stringify error, got {other:?}"),
        }
    }
}

mod stringify_with {
    use super::*;

    #[test]
    fn should_encode_spaces_as_plus_when_option_is_enabled_then_replace_spaces_with_plus() {
        let message = make_message("hello world");
        let options = StringifyOptions {
            space_as_plus: true,
        };

        let result = stringify_with(&message, &options).expect("stringify_with should succeed");

        assert_eq!(result, "body=hello+world");
    }
}
