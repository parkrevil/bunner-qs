#[path = "common/asserts.rs"]
mod asserts;
#[path = "common/json.rs"]
mod json;
#[path = "common/options.rs"]
mod options;
#[path = "common/proptest_profiles.rs"]
mod proptest_profiles;
#[path = "common/serde_data.rs"]
mod serde_data;
#[path = "common/serde_error_fixtures.rs"]
mod serde_error_fixtures;
#[path = "common/serde_helpers.rs"]
mod serde_helpers;
#[path = "common/stringify_options.rs"]
mod stringify_options;

use asserts::{assert_str_path, assert_string_array_path, expect_path};
use bunner_qs::{
    ParseError, ParseOptions, SerdeQueryError, SerdeStringifyError, StringifyOptions, parse,
    parse_with, stringify, stringify_with,
};
use json::json_from_pairs;
use options::try_build_parse_options;
use proptest::prelude::*;
use proptest_profiles::{RandomProfileData, random_profile_strategy};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_data::{
    ContactForm, DesiredContact, DesiredPhone, DesiredProfile, FlattenedContact, FlattenedName,
    FlattenedProfile, NetworkPeer, NotificationPreference, ProfileForm, SimpleUser, TaggedRecord,
    TaggedSettings,
};
use serde_helpers::{
    assert_encoded_contains, assert_parse_roundtrip, assert_stringify_roundtrip,
    assert_stringify_roundtrip_with_options, roundtrip_via_public_api,
};
use serde_json::{Value, json};
use std::borrow::Cow;
use std::collections::BTreeMap;
use std::error::Error;
use stringify_options::try_build_stringify_options;

use serde_error_fixtures::{BoolField, NestedWrapper, UnitHolder};

const STRINGIFY_BUILD_OK: &str = "stringify options builder should succeed";

type ParseOptionsBuilder = bunner_qs::ParseOptionsBuilder;
type StringifyOptionsBuilder = bunner_qs::StringifyOptionsBuilder;

fn build_parse_options<F>(configure: F) -> ParseOptions
where
    F: FnOnce(ParseOptionsBuilder) -> ParseOptionsBuilder,
{
    try_build_parse_options(configure).expect("parse options builder should succeed")
}

fn build_stringify_options<F>(configure: F) -> StringifyOptions
where
    F: FnOnce(StringifyOptionsBuilder) -> StringifyOptionsBuilder,
{
    try_build_stringify_options(configure).expect(STRINGIFY_BUILD_OK)
}

fn parse_serde_error_message<T>(query: &str) -> String
where
    T: DeserializeOwned + Default + std::fmt::Debug + 'static,
{
    let err = parse::<T>(query).expect_err("expected serde error");
    let message = err.to_string();
    match err {
        ParseError::Serde(_) => message,
        other => panic!("expected serde error, got {other:?}"),
    }
}

mod struct_roundtrip_tests {
    use super::*;

    #[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
    struct PrimitiveScalars {
        small_signed: i8,
        medium_signed: i16,
        big_signed: i128,
        small_unsigned: u8,
        medium_unsigned: u16,
        big_unsigned: u128,
        decimal: f32,
        symbol: char,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
    #[serde(rename_all = "camelCase")]
    struct CamelCaseUser {
        first_name: String,
        last_name: String,
        is_active: bool,
    }

    #[derive(Debug, Default)]
    struct BorrowedPayload {
        title: Cow<'static, str>,
        note: Option<Cow<'static, str>>,
    }

    impl<'de> Deserialize<'de> for BorrowedPayload {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            #[derive(Deserialize)]
            struct BorrowedPayloadHelper<'a> {
                #[serde(borrow)]
                title: Cow<'a, str>,
                #[serde(borrow)]
                note: Option<Cow<'a, str>>,
            }

            let helper = BorrowedPayloadHelper::deserialize(deserializer)?;
            let title = Cow::Owned(helper.title.into_owned());
            let note = helper.note.map(|cow| Cow::Owned(cow.into_owned()));
            Ok(BorrowedPayload { title, note })
        }
    }

    fn profile_form() -> ProfileForm {
        ProfileForm {
            username: "ada".into(),
            age: 35,
            active: true,
            contact: ContactForm {
                email: "ada@example.com".into(),
                primary_phone: "+44 123".into(),
                secondary_phone: Some("+44 987".into()),
            },
            nickname: Some("Countess".into()),
        }
    }

    fn flattened_profile() -> FlattenedProfile {
        FlattenedProfile {
            name: FlattenedName {
                first: "Ada".into(),
                last: "Lovelace".into(),
            },
            contact: FlattenedContact {
                email: "ada@example.com".into(),
                phone: "+44 123".into(),
            },
            active: true,
            note: Some("First programmer".into()),
        }
    }

    fn desired_profile() -> DesiredProfile {
        DesiredProfile {
            username: "ada".into(),
            age: 36,
            contact: DesiredContact {
                email: "ada@example.com".into(),
                phones: vec!["+44 123".into(), "+44 987".into()],
                numbers: vec![
                    DesiredPhone {
                        kind: "mobile".into(),
                        number: "+44 123".into(),
                        preferred: true,
                    },
                    DesiredPhone {
                        kind: "office".into(),
                        number: "+44 987".into(),
                        preferred: false,
                    },
                ],
                tags: vec!["pioneer".into(), "math".into()],
            },
            bio: Some("Analytical Engine operator".into()),
        }
    }

    fn tagged_settings() -> TaggedSettings {
        TaggedSettings {
            preference: NotificationPreference::Email {
                address: "ada@example.com".into(),
            },
            token: "SECRET".into(),
        }
    }

    fn complex_profile() -> ProfileForm {
        ProfileForm {
            username: "Complex User".into(),
            age: 54,
            active: true,
            contact: ContactForm {
                email: "complex@example.com".into(),
                primary_phone: "+41 555 0000".into(),
                secondary_phone: Some("+41 555 1111".into()),
            },
            nickname: Some("Cipher".into()),
        }
    }

    fn plus_space_stringify_options() -> StringifyOptions {
        build_stringify_options(|builder| builder.space_as_plus(true))
    }

    fn relaxed_parse_options() -> ParseOptions {
        build_parse_options(|builder| {
            builder
                .space_as_plus(true)
                .max_params(1024)
                .max_length(16 * 1024)
                .max_depth(64)
        })
    }

    #[test]
    fn should_use_default_struct_when_parsing_empty_input() -> Result<(), Box<dyn Error>> {
        // Arrange
        let query = "";

        // Act
        let parsed: SimpleUser = parse(query)?;

        // Assert
        assert_eq!(parsed, SimpleUser::default());
        Ok(())
    }

    #[test]
    fn should_populate_fields_when_scalars_struct_is_parsed() -> Result<(), Box<dyn Error>> {
        // Arrange
        let query = "host=edge.example&port=8080&secure=true";

        // Act
        let peer: NetworkPeer = parse(query)?;

        // Assert
        assert_eq!(peer.host, "edge.example");
        assert_eq!(peer.port, 8080);
        assert!(peer.secure);
        Ok(())
    }

    #[test]
    fn should_preserve_values_when_primitive_scalars_roundtrip() -> Result<(), Box<dyn Error>> {
        // Arrange
        let payload = PrimitiveScalars {
            small_signed: -12,
            medium_signed: -32000,
            big_signed: -9_223_372_036_854_775_808_i128,
            small_unsigned: 12,
            medium_unsigned: 65000,
            big_unsigned: 18_446_744_073_709_551_615_u128,
            decimal: 1.5,
            symbol: 'Ω',
        };

        // Act
        let encoded = stringify(&payload)?;
        let reparsed: PrimitiveScalars = parse(&encoded)?;

        // Assert
        assert_eq!(reparsed, payload);
        Ok(())
    }

    #[test]
    fn should_preserve_case_when_camel_case_struct_roundtrips() -> Result<(), Box<dyn Error>> {
        // Arrange
        let user = CamelCaseUser {
            first_name: "Ada".into(),
            last_name: "Lovelace".into(),
            is_active: true,
        };

        // Act
        let encoded = stringify(&user)?;
        assert_encoded_contains(
            &encoded,
            &["firstName=Ada", "lastName=Lovelace", "isActive=true"],
        );
        let reparsed: CamelCaseUser = parse(&encoded)?;

        // Assert
        assert_eq!(reparsed, user);
        Ok(())
    }

    #[test]
    fn should_preserve_fields_when_struct_roundtrips_via_public_api() -> Result<(), Box<dyn Error>>
    {
        // Arrange
        let profile = profile_form();

        // Act
        let reparsed = roundtrip_via_public_api(&profile)?;

        // Assert
        assert_eq!(reparsed, profile);
        Ok(())
    }

    #[test]
    fn should_keep_flat_fields_when_flattened_struct_roundtrips() -> Result<(), Box<dyn Error>> {
        // Arrange
        let profile = flattened_profile();

        // Act
        let encoded = stringify(&profile)?;
        assert_encoded_contains(
            &encoded,
            &["first_name=Ada", "contact_email=ada%40example.com"],
        );
        let reparsed: FlattenedProfile = parse(&encoded)?;

        // Assert
        assert_eq!(reparsed, profile);
        Ok(())
    }

    #[test]
    fn should_preserve_variant_when_tagged_enum_roundtrips() -> Result<(), Box<dyn Error>> {
        // Arrange
        let settings = tagged_settings();

        // Act
        let encoded = stringify(&settings)?;
        assert_encoded_contains(
            &encoded,
            &[
                "notification_kind=Email",
                "notification%5Baddress%5D=ada%40example.com",
                "access_token=SECRET",
            ],
        );
        let reparsed: TaggedSettings = parse(&encoded)?;

        // Assert
        assert_eq!(reparsed, settings);
        Ok(())
    }

    #[test]
    fn should_remove_padding_when_custom_deserializer_trims_whitespace()
    -> Result<(), Box<dyn Error>> {
        // Arrange
        let raw = concat!(
            "notification_kind=Sms&",
            "notification%5Bnumber%5D=010-0000&",
            "access_token=%20TRIM%20"
        );

        // Act
        let parsed: TaggedSettings = parse(raw)?;

        // Assert
        assert_eq!(parsed.token, "TRIM");
        Ok(())
    }

    #[test]
    fn should_preserve_entries_when_btree_map_roundtrips() -> Result<(), Box<dyn Error>> {
        // Arrange
        let mut data = BTreeMap::new();
        data.insert("city".to_string(), "Seoul".to_string());
        data.insert("country".to_string(), "KR".to_string());

        // Act
        let restored = roundtrip_via_public_api(&data)?;

        // Assert
        assert_eq!(restored, data);
        Ok(())
    }

    #[test]
    fn should_retain_values_when_sequence_field_roundtrips() -> Result<(), Box<dyn Error>> {
        // Arrange
        let record = TaggedRecord {
            name: "release".into(),
            tags: vec!["stable".into(), "serde".into()],
        };

        // Act
        let restored = roundtrip_via_public_api(&record)?;

        // Assert
        assert_eq!(restored, record);
        Ok(())
    }

    #[test]
    fn should_copy_values_when_borrowed_cow_fields_are_parsed() -> Result<(), Box<dyn Error>> {
        // Arrange
        let query = "title=Bonjour&note=Monde";

        // Act
        let parsed: BorrowedPayload = parse(query)?;

        // Assert
        assert_eq!(parsed.title.as_ref(), "Bonjour");
        assert_eq!(parsed.note.as_deref(), Some("Monde"));
        Ok(())
    }

    #[test]
    fn should_coerce_scalars_when_json_value_is_parsed() -> Result<(), Box<dyn Error>> {
        // Arrange
        let query = "post%5Btitle%5D=Hello&post%5Bviews%5D=42&post%5Bpublished%5D=false";

        // Act
        let value: Value = parse(query)?;

        // Assert
        let expected = json!({
            "post": {"title": "Hello", "views": "42", "published": "false"}
        });
        assert_eq!(value, expected);
        Ok(())
    }

    #[test]
    fn should_succeed_in_deep_roundtrip_when_custom_options_used() -> Result<(), Box<dyn Error>> {
        // Arrange
        let profile = complex_profile();
        let stringify_options = plus_space_stringify_options();
        let parse_options = relaxed_parse_options();

        // Act
        let encoded = stringify_with(&profile, &stringify_options)?;
        let reparsed: ProfileForm = parse_with(&encoded, &parse_options)?;
        let profile_value =
            serde_json::to_value(&profile).expect("profile should convert to Value");
        let _ = assert_stringify_roundtrip_with_options(
            &profile_value,
            &stringify_options,
            &parse_options,
        );

        // Assert
        assert_eq!(reparsed, profile);
        Ok(())
    }

    #[test]
    fn should_preserve_contact_fields_when_to_json_style_roundtrip_runs()
    -> Result<(), SerdeQueryError> {
        // Arrange
        let profile = profile_form();

        // Act
        let encoded = stringify(&profile).expect("stringify should succeed");
        let value: Value = parse(&encoded).expect("parse should succeed");

        // Assert
        assert_str_path(&value, &["contact📞", "email📧"], "ada@example.com");
        Ok(())
    }

    #[test]
    fn should_surface_profile_when_stringify_shapes_nested_data() -> Result<(), Box<dyn Error>> {
        // Arrange
        let profile = desired_profile();

        // Act
        let encoded = stringify(&profile)?;
        assert_parse_roundtrip(&encoded);
        let parsed: Value = parse(&encoded)?;
        let profile_value =
            serde_json::to_value(&profile).expect("profile should convert to Value");
        let _ = assert_stringify_roundtrip(&profile_value);

        // Assert
        assert_str_path(&parsed, &["profile✨name"], "ada");
        assert_str_path(&parsed, &["age✨"], "36");
        assert_str_path(&parsed, &["contact", "email📮"], "ada@example.com");
        assert_string_array_path(&parsed, &["contact", "phones📱"], &["+44 123", "+44 987"]);
        let numbers = expect_path(&parsed, &["contact", "numbers📇"])
            .as_array()
            .expect("numbers should be array");
        assert_eq!(numbers.len(), 2);
        assert_str_path(&numbers[0], &["kind🥇"], "mobile");
        assert_str_path(&numbers[0], &["preferred✔"], "true");
        Ok(())
    }
}

mod enum_roundtrip_tests {
    use super::*;

    #[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
    #[serde(tag = "kind", content = "payload")]
    enum InternalMessage {
        #[default]
        Ping,
        Text {
            text: String,
        },
        Metrics {
            value: i32,
        },
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
    struct InternalEnvelope {
        message: InternalMessage,
        priority: u8,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    #[serde(untagged)]
    enum UntaggedValue {
        Word(String),
        Pair { left: i32, right: i32 },
    }

    impl Default for UntaggedValue {
        fn default() -> Self {
            UntaggedValue::Word(String::new())
        }
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
    struct UntaggedEnvelope {
        alias: String,
        value: UntaggedValue,
    }

    #[test]
    fn should_show_deserialize_error_when_internally_tagged_enum_stringified()
    -> Result<(), Box<dyn Error>> {
        // Arrange
        let envelope = InternalEnvelope {
            message: InternalMessage::Text {
                text: "pong".into(),
            },
            priority: 9,
        };

        // Act
        let encoded = stringify(&envelope)?;
        assert_encoded_contains(
            &encoded,
            &[
                "message%5Bkind%5D=Text",
                "message%5Bpayload%5D%5Btext%5D=pong",
                "priority=9",
            ],
        );

        // Assert
        asserts::assert_err_matches!(
            parse::<InternalEnvelope>(&encoded),
            ParseError::Serde(SerdeQueryError::Deserialize(_)) => |message| {
                assert!(message.contains("enum"), "expected enum error: {message}");
            }
        );
        Ok(())
    }

    #[test]
    fn should_report_variant_mismatch_when_untagged_enum_stringified() -> Result<(), Box<dyn Error>>
    {
        // Arrange
        let envelope = UntaggedEnvelope {
            alias: "coords".into(),
            value: UntaggedValue::Pair { left: -3, right: 9 },
        };

        // Act
        let encoded = stringify(&envelope)?;
        assert_encoded_contains(
            &encoded,
            &["alias=coords", "value%5Bleft%5D=-3", "value%5Bright%5D=9"],
        );

        // Assert
        asserts::assert_err_matches!(
            parse::<UntaggedEnvelope>(&encoded),
            ParseError::Serde(SerdeQueryError::Deserialize(_)) => |message| {
                assert!(
                    message.contains("did not match any variant"),
                    "unexpected untagged enum error: {message}"
                );
            }
        );
        Ok(())
    }
}

mod options_behavior_tests {
    use super::*;

    #[test]
    fn should_detect_violation_when_parse_options_are_tightened() {
        // Arrange
        let options = build_parse_options(|builder| builder.max_params(2));

        // Act
        let err =
            parse_with::<SimpleUser>("username=ada&age=36&active=true", &options).unwrap_err();

        // Assert
        assert!(matches!(err, ParseError::TooManyParameters { .. }));
    }

    #[test]
    fn should_emit_plus_when_stringify_options_control_space_encoding() -> Result<(), Box<dyn Error>>
    {
        // Arrange
        let value = json_from_pairs(&[("note", "hello world")]);
        let options = build_stringify_options(|builder| builder.space_as_plus(true));

        // Act
        let encoded = stringify_with(&value, &options)?;

        // Assert
        assert_eq!(encoded, "note=hello+world");
        Ok(())
    }
}

mod error_reporting_tests {
    use super::*;

    #[test]
    fn should_surface_deserialize_error_when_struct_parse_fails() {
        // Arrange
        let query = "host=delta&port=not-a-number&secure=maybe";

        // Act
        let err = parse::<NetworkPeer>(query).unwrap_err();

        // Assert
        assert!(matches!(err, ParseError::Serde(_)));
    }

    #[test]
    fn should_fail_to_parse_when_unknown_field_is_added() {
        // Arrange
        let mut object = json!({ "username": "ada", "age": 36, "active": true });
        if let Value::Object(map) = &mut object {
            map.insert("unexpected".into(), Value::String("boom".into()));
        }

        // Act
        let encoded = stringify(&object).expect("stringify should succeed");
        let result = parse::<SimpleUser>(&encoded);

        // Assert
        assert!(matches!(result, Err(ParseError::Serde(_))));
    }

    #[test]
    fn should_surface_serde_error_when_encoded_value_modified() {
        // Arrange
        let encoded = concat!(
            "profile%E2%9C%A8name=Alias%20User&",
            "age%E2%9C%A8=not-a-number&",
            "contact[email%F0%9F%93%AE]=alias%40example.com"
        );

        // Act
        let result = parse::<DesiredProfile>(encoded);

        // Assert
        assert!(matches!(result, Err(ParseError::Serde(_))));
    }

    #[test]
    fn should_report_detail_when_invalid_bool_provided() {
        // Arrange
        let message = parse_serde_error_message::<BoolField>("secure=maybe");

        // Act
        let contains = message.contains("invalid boolean literal `maybe`");

        // Assert
        assert!(contains, "unexpected deserialize error: {message}");
    }

    #[test]
    fn should_report_expected_object_when_nested_struct_receives_string() {
        // Arrange
        let message = parse_serde_error_message::<NestedWrapper>("peer=value");

        // Act
        let contains = message.contains("expected an object for struct `NestedPeer`");

        // Assert
        assert!(contains, "unexpected deserialize error: {message}");
    }

    #[test]
    fn should_report_unexpected_type_when_unit_field_receives_value() {
        // Arrange
        let message = parse_serde_error_message::<UnitHolder>("empty=value");

        // Act
        let contains = message.contains("expected empty string for unit");

        // Assert
        assert!(contains, "unexpected deserialize error: {message}");
    }

    #[test]
    fn should_report_expected_string_when_flatten_structure_mismatches() {
        // Arrange
        #[allow(dead_code)]
        #[derive(Debug, Deserialize, Default)]
        struct FlattenInner {
            suffix: String,
        }

        #[allow(dead_code)]
        #[derive(Debug, Deserialize, Default)]
        struct FlattenWrapper {
            prefix: String,
            #[serde(flatten)]
            inner: FlattenInner,
        }

        // Act
        let message = parse_serde_error_message::<FlattenWrapper>("prefix=hi&suffix[extra]=boom");

        // Assert
        assert!(
            message.contains("invalid type: map, expected a string"),
            "unexpected flatten error message: {message}"
        );
    }
}

mod adapter_behavior_tests {
    use super::*;

    #[allow(dead_code)]
    #[derive(Debug, Deserialize, Default)]
    struct UppercaseAdapter {
        #[serde(deserialize_with = "uppercase_only")]
        code: String,
    }

    fn uppercase_only<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        if raw.chars().all(|ch| ch.is_ascii_uppercase()) {
            Ok(raw)
        } else {
            Err(serde::de::Error::custom(
                "code must contain only uppercase letters",
            ))
        }
    }

    #[derive(Debug, Deserialize, Default)]
    struct UppercaseTransformAdapter {
        #[serde(deserialize_with = "uppercase_adapter")]
        code: String,
    }

    fn uppercase_adapter<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Ok(raw.to_ascii_uppercase())
    }

    #[derive(Debug, Deserialize, Default)]
    struct SkipAdapter {
        provided: String,
        #[serde(default = "default_token", deserialize_with = "ignore_and_default")]
        token: String,
    }

    fn default_token() -> String {
        "SERVER-DEFAULT".into()
    }

    fn ignore_and_default<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let _ = serde::de::IgnoredAny::deserialize(deserializer)?;
        Ok(default_token())
    }

    #[test]
    fn should_report_error_when_custom_adapter_detects_lowercase() {
        // Arrange
        let message = parse_serde_error_message::<UppercaseAdapter>("code=abc123");

        // Act
        let contains = message.contains("uppercase letters");

        // Assert
        assert!(contains, "unexpected custom adapter error: {message}");
    }

    #[test]
    fn should_uppercase_code_when_custom_adapter_transforms_value() -> Result<(), Box<dyn Error>> {
        // Arrange
        let query = "code=abc123";

        // Act
        let parsed: UppercaseTransformAdapter = parse(query)?;

        // Assert
        assert_eq!(parsed.code, "ABC123");
        Ok(())
    }

    #[test]
    fn should_use_default_when_value_is_ignored_by_adapter() -> Result<(), Box<dyn Error>> {
        // Arrange
        let query = "provided=live&token=client-overrides";

        // Act
        let parsed: SkipAdapter = parse(query)?;

        // Assert
        assert_eq!(parsed.provided, "live");
        assert_eq!(parsed.token, default_token());
        Ok(())
    }
}

mod stringify_error_tests {
    use super::*;

    #[derive(Debug, Serialize)]
    enum UnsupportedVariant {
        Tuple(String, String),
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
    struct UnitKey;

    impl Serialize for UnitKey {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            serializer.serialize_unit()
        }
    }

    #[test]
    fn should_return_error_when_tuple_variant_stringified() {
        // Arrange
        let value = UnsupportedVariant::Tuple("lhs".into(), "rhs".into());

        // Act
        asserts::assert_err_matches!(
            stringify(&value),
            SerdeStringifyError::Serialize(SerdeQueryError::Serialize(_)) => |message| {
                assert!(message.contains("tuple variant"), "unexpected serialize error: {message}");
            }
        );
    }

    #[test]
    fn should_report_error_when_map_key_not_string() {
        // Arrange
        let mut map = BTreeMap::new();
        map.insert(UnitKey, "value".to_string());

        // Act
        asserts::assert_err_matches!(
            stringify(&map),
            SerdeStringifyError::Serialize(SerdeQueryError::Serialize(_)) => |message| {
                assert!(message.contains("map key must be a string"), "unexpected serialize error: {message}");
            }
        );
    }
}

proptest! {
    #[test]
    fn should_roundtrip_random_profiles_when_generated(profile in random_profile_strategy()) {
        let encoded = stringify(&profile).expect("stringify should succeed");
        let reparsed: RandomProfileData = parse(&encoded).expect("parse should succeed");
        prop_assert_eq!(reparsed, profile);
    }
}
