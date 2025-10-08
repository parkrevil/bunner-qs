use crate::StringifyOptions;
use crate::stringify::{stringify, StringifyError};
use assert_matches::assert_matches;
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

fn stringify_with_options<T>(
    value: &T,
    options: &StringifyOptions,
) -> Result<String, StringifyError>
where
    T: Serialize,
{
    stringify(value, options)
}

fn stringify_default<T>(value: &T) -> Result<String, StringifyError>
where
    T: Serialize,
{
    stringify_with_options(value, &StringifyOptions::default())
}

mod stringify {
    use super::*;

    #[test]
    fn should_encode_struct_with_default_options() {
        let result = stringify_default(&Profile {
            name: "Alice",
            city: "Seattle",
        })
        .expect("stringify should succeed");

        assert_eq!(result, "name=Alice&city=Seattle");
    }

    #[test]
    fn should_encode_spaces_as_plus_when_option_enabled() {
        let options = StringifyOptions::new().space_as_plus(true);

        let result = stringify_with_options(
            &Message {
                body: "hello world",
            },
            &options,
        )
        .expect("stringify should succeed");

        assert_eq!(result, "body=hello+world");
    }

    #[test]
    fn should_preserve_percent_encoding_when_space_as_plus_disabled() {
        let options = StringifyOptions::new().space_as_plus(false);

        let result = stringify_with_options(
            &Message {
                body: "hello world",
            },
            &options,
        )
        .expect("stringify should succeed");

        assert_eq!(result, "body=hello%20world");
    }

    #[test]
    fn should_surface_invalid_value_error_for_control_characters() {
        let error = stringify_default(&Message {
            body: "line1\nline2",
        })
        .expect_err("control characters should fail");

        assert_matches!(error, StringifyError::InvalidValue { key } if key == "body");
    }
}
