use crate::serde_adapter::SerializeError;
use crate::{SerdeAdapterError, StringifyError, StringifyOptions};
use assert_matches::assert_matches;
use serde::Serialize;
use serde::ser::Error as _;

#[derive(Serialize)]
struct Profile<'a> {
    name: &'a str,
    city: &'a str,
}

#[derive(Serialize)]
struct Message<'a> {
    body: &'a str,
}

#[derive(Serialize)]
enum Command<'a> {
    Tuple(&'a str, &'a str),
}

struct BrokenPayload;

impl Serialize for BrokenPayload {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Err(S::Error::custom("broken payload"))
    }
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

        let result = crate::stringify(&profile).expect("stringify should succeed");

        assert_eq!(result, "name=Alice&city=Seattle");
    }

    #[test]
    fn should_return_error_when_value_contains_control_characters_then_return_invalid_value_error()
    {
        let message = make_message("line1\nline2");

        let error = crate::stringify(&message).expect_err("control characters should fail");

        assert_matches!(error, StringifyError::InvalidValue { key } if key == "body");
    }

    #[test]
    fn should_wrap_serialize_error_when_top_level_is_not_map_then_return_serde_error() {
        let error = crate::stringify(&"plain").expect_err("non-map top level should fail");

        assert_matches!(
            error,
            StringifyError::Serialize(SerdeAdapterError::Serialize(
                SerializeError::TopLevel(kind)
            )) if kind == "string"
        );
    }

    #[test]
    fn should_wrap_unexpected_skip_error_when_option_is_none_then_return_serde_error() {
        let data: Option<Profile> = None;

        let error = crate::stringify(&data).expect_err("option none should be rejected");

        assert_matches!(
            error,
            StringifyError::Serialize(SerdeAdapterError::Serialize(SerializeError::UnexpectedSkip))
        );
    }

    #[test]
    fn should_wrap_unsupported_error_when_enum_contains_tuple_variant_then_return_serde_error() {
        let command = Command::Tuple("run", "now");

        let error = crate::stringify(&command).expect_err("tuple variant should be unsupported");

        assert_matches!(
            error,
            StringifyError::Serialize(SerdeAdapterError::Serialize(
                SerializeError::Unsupported(kind)
            )) if kind == "tuple variant"
        );
    }

    #[test]
    fn should_wrap_custom_message_error_when_serializer_returns_custom_error_then_propagate_message()
     {
        let error = crate::stringify(&BrokenPayload)
            .expect_err("custom serialization error should propagate");

        assert_matches!(
            error,
            StringifyError::Serialize(SerdeAdapterError::Serialize(
                SerializeError::Message(message)
            )) if message == "broken payload"
        );
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

        let result =
            crate::stringify_with(&message, &options).expect("stringify_with should succeed");

        assert_eq!(result, "body=hello+world");
    }

    #[test]
    fn should_percent_encode_spaces_when_option_disabled_then_preserve_percent_encoding() {
        let message = make_message("hello world");
        let options = StringifyOptions {
            space_as_plus: false,
        };

        let result =
            crate::stringify_with(&message, &options).expect("stringify_with should succeed");

        assert_eq!(result, "body=hello%20world");
    }

    #[test]
    fn should_wrap_serialize_error_when_tuple_variant_provided_then_return_serde_error() {
        let options = StringifyOptions::default();
        let command = Command::Tuple("run", "now");

        let error = crate::stringify_with(&command, &options)
            .expect_err("tuple variant should be unsupported");

        assert_matches!(
            error,
            StringifyError::Serialize(SerdeAdapterError::Serialize(
                SerializeError::Unsupported(kind)
            )) if kind == "tuple variant"
        );
    }

    #[test]
    fn should_wrap_runtime_error_when_value_contains_control_characters_then_return_invalid_value_error()
     {
        let message = make_message("line1\nline2");
        let options = StringifyOptions {
            space_as_plus: true,
        };

        let error = crate::stringify_with(&message, &options)
            .expect_err("control characters should fail even with custom options");

        assert_matches!(error, StringifyError::InvalidValue { key } if key == "body");
    }
}
