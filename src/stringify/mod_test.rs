use super::{SerdeStringifyError, StringifyError, StringifyOptions, stringify, stringify_with};
use serde::Serialize;

mod stringify_mod_tests {
    use super::*;

    #[derive(Serialize)]
    struct Profile<'a> {
        name: &'a str,
        city: &'a str,
    }

    #[derive(Serialize)]
    struct Message<'a> {
        body: &'a str,
    }

    #[test]
    fn when_stringify_is_called_it_should_use_default_options() {
        let profile = Profile {
            name: "Alice",
            city: "Seattle",
        };

        let result = stringify(&profile).expect("stringify should succeed");

        assert_eq!(result, "name=Alice&city=Seattle");
    }

    #[test]
    fn when_stringify_with_has_space_as_plus_enabled_it_should_replace_spaces() {
        let message = Message {
            body: "hello world",
        };
        let options = StringifyOptions {
            space_as_plus: true,
        };

        let result = stringify_with(&message, &options).expect("stringify_with should succeed");

        assert_eq!(result, "body=hello+world");
    }

    #[test]
    fn when_stringify_encounters_control_characters_it_should_return_error() {
        let message = Message {
            body: "line1\nline2",
        };

        let error = stringify(&message).expect_err("control characters should fail");

        match error {
            SerdeStringifyError::Stringify(StringifyError::InvalidValue { key }) => {
                assert_eq!(key, "body")
            }
            other => panic!("expected stringify error, got {other:?}"),
        }
    }
}
