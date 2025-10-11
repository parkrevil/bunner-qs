use crate::StringifyOptions;
use crate::serde_adapter::SerializeError;
use crate::stringify::{StringifyError, stringify};
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
    use serde::ser::{SerializeMap, Serializer};

    #[test]
    fn given_struct_when_stringify_with_defaults_then_encodes_pairs() {
        let result = stringify_default(&Profile {
            name: "Alice",
            city: "Seattle",
        })
        .expect("stringify should succeed");

        assert_eq!(result, "name=Alice&city=Seattle");
    }

    #[test]
    fn given_space_as_plus_option_when_stringify_then_uses_plus_separator() {
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
    fn given_space_as_plus_disabled_when_stringify_then_preserves_percent_encoding() {
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
    fn given_control_characters_when_stringify_then_returns_invalid_value_error() {
        let error = stringify_default(&Message {
            body: "line1\nline2",
        })
        .expect_err("control characters should fail");

        assert_matches!(
            error,
            StringifyError::InvalidValue { key, value }
                if key == "body" && value == "line1\nline2"
        );
    }

    #[test]
    fn given_top_level_scalar_when_stringify_then_returns_top_level_error() {
        let error = stringify_default(&"plain string")
            .expect_err("top-level scalars should fail serialization");

        assert_matches!(
            error,
            StringifyError::Serialize(SerializeError::TopLevel(found))
                if found == "string"
        );
    }

    #[test]
    fn given_option_none_when_stringify_then_reports_unexpected_skip() {
        let error = stringify_default(&Option::<u8>::None)
            .expect_err("none should trigger unexpected skip error");

        assert_matches!(
            error,
            StringifyError::Serialize(SerializeError::UnexpectedSkip)
        );
    }

    #[derive(Debug)]
    struct InvalidKeyPayload;

    impl Serialize for InvalidKeyPayload {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut map = serializer.serialize_map(Some(1))?;
            map.serialize_entry("bad\nkey", "value")?;
            map.end()
        }
    }

    #[test]
    fn given_invalid_key_when_stringify_then_returns_invalid_key_error() {
        let error = stringify_default(&InvalidKeyPayload).expect_err("invalid key should fail");

        assert_matches!(
            error,
            StringifyError::InvalidKey { key } if key == "bad\nkey"
        );
    }
}
