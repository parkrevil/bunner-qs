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
use bunner_qs::{ParseError, SerdeQueryError, parse, parse_with, stringify, stringify_with};
use json::json_from_pairs;
use options::build_parse_options;
use proptest::prelude::*;
use proptest_profiles::{RandomProfileData, random_profile_strategy};
use serde::Serialize;
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
use std::collections::BTreeMap;
use std::error::Error;
use stringify_options::build_stringify_options;

use serde_error_fixtures::{BoolField, NestedWrapper, UnitHolder};

#[test]
fn parse_into_struct_returns_default_for_empty_input() -> Result<(), Box<dyn Error>> {
    let parsed: SimpleUser = parse("")?;
    assert_eq!(parsed, SimpleUser::default());
    Ok(())
}

#[test]
fn parse_into_struct_with_scalars() -> Result<(), Box<dyn Error>> {
    let peer: NetworkPeer = parse("host=edge.example&port=8080&secure=true")?;
    assert_eq!(peer.host, "edge.example");
    assert_eq!(peer.port, 8080);
    assert!(peer.secure);
    Ok(())
}

#[test]
fn parse_into_json_value_coerces_scalars() -> Result<(), Box<dyn Error>> {
    let value: Value = parse("post%5Btitle%5D=Hello&post%5Bviews%5D=42&post%5Bpublished%5D=false")?;
    let expected = json!({
        "post": {
            "title": "Hello",
            "views": "42",
            "published": "false",
        }
    });
    assert_eq!(value, expected);
    Ok(())
}

#[test]
fn parse_into_struct_surfaces_deserialize_errors() {
    let err = parse::<NetworkPeer>("host=delta&port=not-a-number&secure=maybe").unwrap_err();
    assert!(matches!(err, ParseError::Serde(_)));
}

#[test]
fn struct_roundtrip_preserves_data() -> Result<(), Box<dyn Error>> {
    let profile = ProfileForm {
        username: "ada".into(),
        age: 35,
        active: true,
        contact: ContactForm {
            email: "ada@example.com".into(),
            primary_phone: "+44 123".into(),
            secondary_phone: Some("+44 987".into()),
        },
        nickname: Some("Countess".into()),
    };

    let reparsed = roundtrip_via_public_api(&profile)?;
    assert_eq!(reparsed, profile);
    Ok(())
}

#[test]
fn stringify_shapes_nested_data_for_inspection() -> Result<(), Box<dyn Error>> {
    let profile = DesiredProfile {
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
    };

    let encoded = stringify(&profile).expect("stringify should succeed");
    assert_parse_roundtrip(&encoded);
    let parsed: Value = parse(&encoded)?;
    let profile_value = serde_json::to_value(&profile).expect("profile should convert to Value");
    let _ = assert_stringify_roundtrip(&profile_value);
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

#[test]
fn tighten_parse_options_detects_violations() {
    let options = build_parse_options(|builder| builder.max_params(2));

    let err = parse_with::<SimpleUser>("username=ada&age=36&active=true", &options).unwrap_err();
    assert!(matches!(err, ParseError::TooManyParameters { .. }));
}

#[test]
fn stringify_options_control_space_encoding() -> Result<(), Box<dyn Error>> {
    let value = json_from_pairs(&[("note", "hello world")]);
    let options = build_stringify_options(|builder| builder.space_as_plus(true));
    let encoded = stringify_with(&value, &options)?;
    assert_eq!(encoded, "note=hello+world");
    Ok(())
}

#[test]
fn btree_map_roundtrip_via_public_api() -> Result<(), Box<dyn Error>> {
    let mut data = BTreeMap::new();
    data.insert("city".to_string(), "Seoul".to_string());
    data.insert("country".to_string(), "KR".to_string());

    let restored = roundtrip_via_public_api(&data)?;
    assert_eq!(restored, data);
    Ok(())
}

#[test]
fn sequence_field_roundtrip_preserves_values() -> Result<(), Box<dyn Error>> {
    let record = TaggedRecord {
        name: "release".into(),
        tags: vec!["stable".into(), "serde".into()],
    };

    let restored = roundtrip_via_public_api(&record)?;
    assert_eq!(restored, record);
    Ok(())
}

#[test]
fn flattened_struct_roundtrip_preserves_fields() -> Result<(), Box<dyn Error>> {
    let profile = FlattenedProfile {
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
    };

    let encoded = stringify(&profile)?;
    assert_encoded_contains(
        &encoded,
        &["first_name=Ada", "contact_email=ada%40example.com"],
    );
    let reparsed: FlattenedProfile = parse(&encoded)?;
    assert_eq!(reparsed, profile);
    Ok(())
}

#[test]
fn tagged_enum_roundtrip_preserves_variant_and_token() -> Result<(), Box<dyn Error>> {
    let settings = TaggedSettings {
        preference: NotificationPreference::Email {
            address: "ada@example.com".into(),
        },
        token: "SECRET".into(),
    };

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
    assert_eq!(reparsed, settings);
    Ok(())
}

#[test]
fn custom_deserializer_trims_whitespace_from_token() -> Result<(), Box<dyn Error>> {
    let raw = concat!(
        "notification_kind=Sms&",
        "notification%5Bnumber%5D=010-0000&",
        "access_token=%20TRIM%20"
    );

    let parsed: TaggedSettings = parse(raw)?;
    let expected = TaggedSettings {
        preference: NotificationPreference::Sms {
            number: "010-0000".into(),
        },
        token: "TRIM".into(),
    };
    assert_eq!(parsed, expected);
    Ok(())
}

#[test]
fn parse_rejects_unknown_field_after_serialization() {
    let mut object = json!({
        "username": "ada",
        "age": 36,
        "active": true
    });
    if let Value::Object(map) = &mut object {
        map.insert("unexpected".into(), Value::String("boom".into()));
    }
    let encoded = stringify(&object).expect("stringify should succeed");
    let result = parse::<SimpleUser>(&encoded);
    assert!(matches!(result, Err(ParseError::Serde(_))));
}

#[test]
fn deep_roundtrip_with_custom_options() -> Result<(), Box<dyn Error>> {
    let profile = ProfileForm {
        username: "Complex User".into(),
        age: 54,
        active: true,
        contact: ContactForm {
            email: "complex@example.com".into(),
            primary_phone: "+41 555 0000".into(),
            secondary_phone: Some("+41 555 1111".into()),
        },
        nickname: Some("Cipher".into()),
    };

    let stringify_options = build_stringify_options(|builder| builder.space_as_plus(true));
    let parse_options = build_parse_options(|builder| {
        builder
            .space_as_plus(true)
            .max_params(1024)
            .max_length(16 * 1024)
            .max_depth(64)
    });

    let encoded = stringify_with(&profile, &stringify_options)?;
    let reparsed: ProfileForm = parse_with(&encoded, &parse_options)?;
    let profile_value = serde_json::to_value(&profile).expect("profile should convert to Value");
    let _ =
        assert_stringify_roundtrip_with_options(&profile_value, &stringify_options, &parse_options);
    assert_eq!(reparsed, profile);
    Ok(())
}

#[test]
fn serde_errors_surface_during_parse_with_modified_value() {
    let encoded = concat!(
        "profile%E2%9C%A8name=Alias%20User&",
        "age%E2%9C%A8=not-a-number&",
        "contact[email%F0%9F%93%AE]=alias%40example.com"
    );
    let result = parse::<DesiredProfile>(encoded);
    assert!(matches!(result, Err(ParseError::Serde(_))));
}

#[test]
fn to_json_style_value_roundtrip() -> Result<(), SerdeQueryError> {
    let profile = ProfileForm {
        username: "json_user".into(),
        age: 21,
        active: true,
        contact: ContactForm {
            email: "json@example.com".into(),
            primary_phone: "+1 555".into(),
            secondary_phone: None,
        },
        nickname: None,
    };

    let encoded = stringify(&profile).expect("stringify should succeed");
    let value: Value = parse(&encoded).expect("parse should succeed");
    assert_str_path(&value, &["contact📞", "email📧"], "json@example.com");
    Ok(())
}

#[test]
fn stringify_rejects_tuple_variants() {
    #[derive(Debug, Serialize)]
    enum UnsupportedVariant {
        Tuple(String, String),
    }

    asserts::assert_err_matches!(
        stringify(&UnsupportedVariant::Tuple("lhs".into(), "rhs".into())),
        bunner_qs::SerdeStringifyError::Serialize(SerdeQueryError::Serialize(_)) => |message| {
            assert!(
                message.contains("tuple variant"),
                "unexpected serialize error: {message}"
            );
        }
    );
}

#[test]
fn stringify_rejects_invalid_map_keys() {
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

    let mut map = BTreeMap::new();
    map.insert(UnitKey, "value".to_string());

    asserts::assert_err_matches!(
        stringify(&map),
        bunner_qs::SerdeStringifyError::Serialize(SerdeQueryError::Serialize(_)) => |message| {
            assert!(
                message.contains("map key must be a string"),
                "unexpected serialize error: {message}"
            );
        }
    );
}

#[test]
fn parse_reports_invalid_bool_detail() {
    asserts::assert_err_matches!(
        parse::<BoolField>("secure=maybe"),
        ParseError::Serde(SerdeQueryError::Deserialize(_)) => |message| {
            assert!(
                message.contains("invalid boolean literal `maybe`"),
                "unexpected deserialize error: {message}"
            );
        }
    );
}

#[test]
fn parse_reports_expected_object_for_nested_struct() {
    asserts::assert_err_matches!(
        parse::<NestedWrapper>("peer=value"),
        ParseError::Serde(SerdeQueryError::Deserialize(_)) => |message| {
            assert!(
                message.contains("expected an object for struct `NestedPeer`, found string"),
                "unexpected deserialize error: {message}"
            );
        }
    );
}

#[test]
fn parse_reports_unexpected_type_for_unit_field() {
    asserts::assert_err_matches!(
        parse::<UnitHolder>("empty=value"),
        ParseError::Serde(SerdeQueryError::Deserialize(_)) => |message| {
            assert!(
                message.contains("expected empty string for unit"),
                "unexpected deserialize error: {message}"
            );
        }
    );
}

proptest! {
    #[test]
    fn random_profile_roundtrips(profile in random_profile_strategy()) {
        let encoded = stringify(&profile).expect("stringify should succeed");
        let reparsed: RandomProfileData = parse(&encoded).expect("parse should succeed");
        prop_assert_eq!(reparsed, profile);
    }
}
