#![cfg(feature = "serde")]

mod common;

use bunner_qs::{QueryMap, SerdeQueryError, Value};
use common::{assert_str_entry, expect_array, expect_object};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[cfg(test)]
#[allow(dead_code)]
#[deprecated(
    note = "serde_urlencoded cannot yet serialize nested arrays/objects; tests are ignored until support is added"
)]
fn _serde_roundtrip_pending_warning() {}

#[cfg(test)]
#[allow(dead_code)]
#[allow(deprecated)]
fn _trigger_serde_roundtrip_warning() {
    _serde_roundtrip_pending_warning();
}

/// serde_urlencoded does not support nested struct values directly, so we expose
/// the `contact[...]` projections explicitly for query string generation.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct ProfileForm {
    username: String,
    age: u8,
    active: bool,
    #[serde(rename = "contact[email]")]
    contact_email: String,
    #[serde(rename = "contact[primary_phone]")]
    contact_primary_phone: String,
    #[serde(rename = "contact[secondary_phone]", default)]
    contact_secondary_phone: Option<String>,
    #[serde(default)]
    nickname: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct SimpleUser {
    username: String,
    age: u8,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct TaggedRecord {
    name: String,
    tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct DesiredPhone {
    kind: String,
    number: String,
    preferred: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct DesiredContact {
    email: String,
    phones: Vec<String>,
    numbers: Vec<DesiredPhone>,
    tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct DesiredProfile {
    username: String,
    age: u8,
    contact: DesiredContact,
    bio: Option<String>,
}

#[test]
#[ignore = "serde_urlencoded currently cannot serialize nested arrays/objects"]
fn struct_roundtrip_preserves_data() -> Result<(), SerdeQueryError> {
    let profile = ProfileForm {
        username: "ada".into(),
        age: 35,
        active: true,
        contact_email: "ada@example.com".into(),
        contact_primary_phone: "+44 123".into(),
        contact_secondary_phone: Some("+44 987".into()),
        nickname: Some("Countess".into()),
    };

    let map = QueryMap::from_struct(&profile)?;
    let restored: ProfileForm = map.to_struct()?;
    assert_eq!(restored, profile);
    Ok(())
}

#[test]
#[ignore = "serde_urlencoded currently cannot serialize nested arrays/objects"]
fn query_map_contains_expected_nested_values() -> Result<(), SerdeQueryError> {
    let profile = ProfileForm {
        username: "grace".into(),
        age: 29,
        active: false,
        contact_email: "grace@example.com".into(),
        contact_primary_phone: "+1 555-0100".into(),
        contact_secondary_phone: None,
        nickname: None,
    };

    let map = QueryMap::from_struct(&profile)?;

    assert_str_entry(&map, "username", "grace");
    assert_str_entry(&map, "age", "29");
    assert_str_entry(&map, "active", "false");

    let contact = map.get("contact").expect("missing contact value");
    let contact_obj = expect_object(contact);
    assert_str_entry(contact_obj, "email", "grace@example.com");

    assert_str_entry(contact_obj, "primary_phone", "+1 555-0100");
    assert!(
        !contact_obj.contains_key("secondary_phone"),
        "optional phone should be omitted"
    );

    assert!(
        !map.contains_key("nickname"),
        "unset option should be omitted"
    );

    Ok(())
}

#[test]
#[ignore = "serde_urlencoded currently cannot serialize nested arrays/objects"]
fn btree_map_roundtrip() -> Result<(), SerdeQueryError> {
    let mut data = BTreeMap::new();
    data.insert("city".to_string(), "Seoul".to_string());
    data.insert("country".to_string(), "KR".to_string());

    let query_map = QueryMap::from_struct(&data)?;
    let restored: BTreeMap<String, String> = query_map.to_struct()?;

    assert_eq!(restored, data);
    Ok(())
}

#[test]
#[ignore = "serde_urlencoded currently cannot serialize nested arrays/objects"]
fn to_struct_sequence_field_returns_error() {
    let mut map = QueryMap::new();
    map.insert("name".into(), Value::String("release".into()));
    map.insert(
        "tags".into(),
        Value::Array(vec![
            Value::String("stable".into()),
            Value::String("serde".into()),
        ]),
    );

    let result: Result<TaggedRecord, SerdeQueryError> = map.to_struct();
    match result {
        Err(SerdeQueryError::Deserialize(_)) => {}
        other => panic!("expected Deserialize error, got {other:?}"),
    }
}

#[test]
#[ignore = "serde_urlencoded currently cannot serialize nested arrays/objects"]
fn from_struct_sequence_field_returns_error() {
    let record = TaggedRecord {
        name: "release".into(),
        tags: vec!["stable".into(), "serde".into()],
    };

    let result = QueryMap::from_struct(&record);
    match result {
        Err(SerdeQueryError::Serialize(_)) => {}
        other => panic!("expected Serialize error, got {other:?}"),
    }
}

#[test]
#[ignore = "serde_urlencoded currently cannot serialize nested arrays/objects"]
fn to_struct_missing_required_field_returns_error() {
    let mut map = QueryMap::new();
    map.insert("username".into(), Value::String("no-age".into()));

    let result: Result<SimpleUser, SerdeQueryError> = map.to_struct();
    match result {
        Err(SerdeQueryError::Deserialize(_)) => {}
        other => panic!("expected Deserialize error, got {other:?}"),
    }
}

#[test]
#[ignore = "serde_urlencoded currently cannot serialize nested arrays/objects"]
fn to_struct_invalid_number_returns_error() {
    let mut map = QueryMap::new();
    map.insert("username".into(), Value::String("invalid".into()));
    map.insert("age".into(), Value::String("not-a-number".into()));

    let result: Result<SimpleUser, SerdeQueryError> = map.to_struct();
    match result {
        Err(SerdeQueryError::Deserialize(_)) => {}
        other => panic!("expected Deserialize error, got {other:?}"),
    }
}

#[test]
#[ignore = "serde_urlencoded currently cannot serialize nested arrays/objects"]
fn desired_nested_struct_roundtrip_should_succeed() {
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

    let map = QueryMap::from_struct(&profile)
        .expect("QueryMap::from_struct should support nested structs");
    let restored: DesiredProfile = map
        .to_struct()
        .expect("QueryMap::to_struct should deserialize nested structs");
    assert_eq!(
        restored, profile,
        "round-trip through QueryMap should preserve data"
    );
}

#[test]
#[ignore = "serde_urlencoded currently cannot serialize nested arrays/objects"]
fn desired_nested_struct_shape_should_include_arrays() {
    let profile = DesiredProfile {
        username: "grace".into(),
        age: 30,
        contact: DesiredContact {
            email: "grace@example.com".into(),
            phones: vec!["+1 555-0100".into()],
            numbers: vec![DesiredPhone {
                kind: "mobile".into(),
                number: "+1 555-0100".into(),
                preferred: true,
            }],
            tags: vec!["compiler".into(), "navy".into()],
        },
        bio: None,
    };

    let map = QueryMap::from_struct(&profile).expect("serializing nested arrays should work");

    let contact = map.get("contact").expect("contact key should exist");
    let contact_obj = expect_object(contact);
    assert_str_entry(contact_obj, "email", "grace@example.com");

    let phones = contact_obj
        .get("phones")
        .expect("phones array should exist");
    let phones_array = expect_array(phones);
    assert_eq!(
        phones_array.len(),
        1,
        "phones array should contain one entry"
    );
    assert_eq!(phones_array[0].as_str(), Some("+1 555-0100"));

    let tags = contact_obj.get("tags").expect("tags array should exist");
    let tags_array = expect_array(tags);
    assert_eq!(
        tags_array
            .iter()
            .map(|value| value.as_str().map(|s| s.to_string()))
            .collect::<Option<Vec<_>>>(),
        Some(vec!["compiler".into(), "navy".into()]),
        "tags array should preserve insertion order"
    );
}

#[test]
#[ignore = "serde_urlencoded currently cannot serialize nested arrays/objects"]
fn desired_struct_should_support_array_of_objects() {
    let profile = DesiredProfile {
        username: "linus".into(),
        age: 33,
        contact: DesiredContact {
            email: "linus@example.com".into(),
            phones: vec![],
            numbers: vec![
                DesiredPhone {
                    kind: "mobile".into(),
                    number: "+46 111".into(),
                    preferred: true,
                },
                DesiredPhone {
                    kind: "home".into(),
                    number: "+46 222".into(),
                    preferred: false,
                },
            ],
            tags: vec!["kernel".into()],
        },
        bio: None,
    };

    let map =
        QueryMap::from_struct(&profile).expect("serializing array-of-object values should work");

    let contact = map.get("contact").expect("contact key should exist");
    let contact_obj = expect_object(contact);
    let numbers = contact_obj
        .get("numbers")
        .expect("numbers array should exist");
    let numbers_array = expect_array(numbers);
    assert_eq!(numbers_array.len(), 2, "should keep both phone entries");

    let first = expect_object(&numbers_array[0]);
    assert_str_entry(first, "kind", "mobile");
    assert_str_entry(first, "number", "+46 111");
    assert_str_entry(first, "preferred", "true");

    let second = expect_object(&numbers_array[1]);
    assert_str_entry(second, "kind", "home");
    assert_str_entry(second, "number", "+46 222");
    assert_str_entry(second, "preferred", "false");
}
