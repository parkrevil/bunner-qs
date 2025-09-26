mod common;

use bunner_qs::{
    ParseError, ParseOptions, SerdeQueryError, StringifyOptions, parse, parse_with, stringify,
    stringify_with,
};
use common::{assert_str_entry, assert_string_array, expect_object, json_from_pairs};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::error::Error;

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
struct ContactForm {
    #[serde(rename = "email📧", alias = "email_address")]
    email: String,
    #[serde(rename = "primary-phone")]
    primary_phone: String,
    #[serde(
        default,
        alias = "secondaryPhone",
        skip_serializing_if = "Option::is_none"
    )]
    secondary_phone: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
struct ProfileForm {
    #[serde(rename = "user_name")]
    username: String,
    #[serde(rename = "age-years", alias = "user_age")]
    age: u8,
    #[serde(rename = "active?")]
    active: bool,
    #[serde(rename = "contact📞")]
    contact: ContactForm,
    #[serde(
        default,
        rename = "nickname🎭",
        alias = "alias🎭",
        skip_serializing_if = "Option::is_none"
    )]
    nickname: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
struct SimpleUser {
    username: String,
    age: u8,
    active: bool,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
struct TaggedRecord {
    name: String,
    tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Default, Clone)]
struct DesiredPhone {
    #[serde(rename = "kind🥇")]
    kind: String,
    #[serde(rename = "number#")]
    number: String,
    #[serde(rename = "preferred✔")]
    preferred: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Default, Clone)]
struct DesiredContact {
    #[serde(rename = "email📮")]
    email: String,
    #[serde(rename = "phones📱")]
    phones: Vec<String>,
    #[serde(rename = "numbers📇")]
    numbers: Vec<DesiredPhone>,
    #[serde(rename = "tags🔥", alias = "tag_list", default)]
    tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Default, Clone)]
struct DesiredProfile {
    #[serde(rename = "profile✨name")]
    username: String,
    #[serde(rename = "age✨", alias = "alt_age")]
    age: u8,
    contact: DesiredContact,
    bio: Option<String>,
}

#[test]
fn parse_into_struct_returns_default_for_empty_input() -> Result<(), Box<dyn Error>> {
    let parsed: SimpleUser = parse("")?;
    assert_eq!(parsed, SimpleUser::default());
    Ok(())
}

#[test]
fn parse_into_struct_with_scalars() -> Result<(), Box<dyn Error>> {
    #[allow(dead_code)]
    #[derive(Debug, Deserialize, PartialEq, Default)]
    struct NetworkPeer {
        host: String,
        port: u16,
        secure: bool,
    }

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
    #[allow(dead_code)]
    #[derive(Debug, Deserialize, Default)]
    struct NetworkPeer {
        host: String,
        port: u16,
        secure: bool,
    }

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

    let encoded = stringify(&profile).expect("stringify should succeed");
    let reparsed: ProfileForm = parse(&encoded)?;
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
    let parsed: Value = parse(&encoded)?;
    let root = expect_object(&parsed);

    assert_str_entry(root, "profile✨name", "ada");
    assert_str_entry(root, "age✨", "36");

    let contact = expect_object(root.get("contact").expect("missing contact"));
    assert_str_entry(contact, "email📮", "ada@example.com");

    let phones = contact.get("phones📱").expect("missing phones");
    assert_string_array(phones, &["+44 123", "+44 987"]);

    let numbers = contact
        .get("numbers📇")
        .expect("missing numbers")
        .as_array()
        .expect("numbers should be array");
    assert_eq!(numbers.len(), 2);
    let first = expect_object(&numbers[0]);
    assert_str_entry(first, "kind🥇", "mobile");
    assert_str_entry(first, "preferred✔", "true");

    Ok(())
}

#[test]
fn tighten_parse_options_detects_violations() {
    let options = ParseOptions {
        max_params: Some(2),
        ..ParseOptions::default()
    };

    let err = parse_with::<SimpleUser>("username=ada&age=36&active=true", &options).unwrap_err();
    assert!(matches!(err, ParseError::TooManyParameters { .. }));
}

#[test]
fn stringify_options_control_space_encoding() -> Result<(), Box<dyn Error>> {
    let value = json_from_pairs(&[("note", "hello world")]);
    let options = StringifyOptions::builder()
        .space_as_plus(true)
        .build()
        .expect("builder should succeed");
    let encoded = stringify_with(&value, &options)?;
    assert_eq!(encoded, "note=hello+world");
    Ok(())
}

#[test]
fn btree_map_roundtrip_via_public_api() -> Result<(), Box<dyn Error>> {
    let mut data = BTreeMap::new();
    data.insert("city".to_string(), "Seoul".to_string());
    data.insert("country".to_string(), "KR".to_string());

    let encoded = stringify(&data)?;
    let restored: BTreeMap<String, String> = parse(&encoded)?;
    assert_eq!(restored, data);
    Ok(())
}

#[test]
fn sequence_field_roundtrip_preserves_values() -> Result<(), Box<dyn Error>> {
    let record = TaggedRecord {
        name: "release".into(),
        tags: vec!["stable".into(), "serde".into()],
    };

    let encoded = stringify(&record)?;
    let restored: TaggedRecord = parse(&encoded)?;
    assert_eq!(restored, record);
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

    let stringify_options = StringifyOptions::builder()
        .space_as_plus(true)
        .build()
        .expect("builder should succeed");
    let parse_options = ParseOptions::builder()
        .space_as_plus(true)
        .max_params(1024)
        .max_length(16 * 1024)
        .max_depth(64)
        .build()
        .expect("parse builder should succeed");

    let encoded = stringify_with(&profile, &stringify_options)?;
    let reparsed: ProfileForm = parse_with(&encoded, &parse_options)?;
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
    let contact = expect_object(value.get("contact📞").expect("missing contact"));
    assert_str_entry(contact, "email📧", "json@example.com");
    Ok(())
}
