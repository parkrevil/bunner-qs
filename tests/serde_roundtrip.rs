#![cfg(feature = "serde")]

mod common;

use bunner_qs::{QueryMap, SerdeQueryError, Value};
use common::{assert_str_entry, expect_array, expect_object};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct ContactForm {
    email: String,
    primary_phone: String,
    #[serde(default)]
    secondary_phone: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct ProfileForm {
    username: String,
    age: u8,
    active: bool,
    contact: ContactForm,
    #[serde(default)]
    nickname: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct SimpleUser {
    username: String,
    age: u8,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct TaggedRecord {
    name: String,
    tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct DesiredPhone {
    kind: String,
    number: String,
    preferred: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct DesiredContact {
    email: String,
    phones: Vec<String>,
    numbers: Vec<DesiredPhone>,
    tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct DesiredProfile {
    username: String,
    age: u8,
    contact: DesiredContact,
    bio: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct NetworkPeer {
    host: String,
    port: u16,
    secure: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct LocaleSettings {
    language: String,
    description: String,
    greetings: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct Metrics {
    load: f32,
    requests: u64,
    trend: Vec<f64>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct LevelFive {
    message: String,
    ordinal: i32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct LevelFour {
    code: String,
    depth: LevelFive,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct LevelThree {
    token: String,
    depth: LevelFour,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct LevelTwo {
    key: String,
    depth: LevelThree,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct LevelOneDeep {
    namespace: String,
    depth: LevelTwo,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct DeepEnvelope {
    id: u64,
    checksum: String,
    profile: ProfileForm,
    history: Vec<ProfileForm>,
    level: LevelOneDeep,
    peers: Vec<NetworkPeer>,
    locales: BTreeMap<String, LocaleSettings>,
    breadcrumbs: Vec<Vec<String>>,
    matrix: Vec<Vec<i32>>,
    feature_flags: Vec<bool>,
    tags: Vec<String>,
    metrics: Metrics,
    extra_notes: Option<Vec<String>>,
}

fn build_deep_envelope() -> DeepEnvelope {
    let base_profile = ProfileForm {
        username: "deeproot".into(),
        age: 41,
        active: true,
        contact: ContactForm {
            email: "root@example.com".into(),
            primary_phone: "+82 010-555-0100".into(),
            secondary_phone: Some("+1 999-9999".into()),
        },
        nickname: Some("Root 🔑".into()),
    };

    let history = vec![
        base_profile.clone(),
        ProfileForm {
            username: "legacy-user".into(),
            age: 39,
            active: false,
            contact: ContactForm {
                email: "legacy@example.com".into(),
                primary_phone: "+44 20 7946 0958".into(),
                secondary_phone: None,
            },
            nickname: Some("Archivist".into()),
        },
    ];

    let mut locales = BTreeMap::new();
    locales.insert(
        "en-US".into(),
        LocaleSettings {
            language: "English (US)".into(),
            description: "primary locale".into(),
            greetings: vec!["Hello".into(), "Howdy".into()],
        },
    );
    locales.insert(
        "ko-KR".into(),
        LocaleSettings {
            language: "한국어".into(),
            description: "기본 로케일".into(),
            greetings: vec!["안녕하세요".into(), "반가워요".into()],
        },
    );
    locales.insert(
        "emoji🌐".into(),
        LocaleSettings {
            language: "Emoji Tongue".into(),
            description: "experimental 🌐 locale".into(),
            greetings: vec!["👋".into(), "🙌🔥".into()],
        },
    );

    DeepEnvelope {
        id: 42,
        checksum: "🔥-hash-✓".into(),
        profile: base_profile,
        history,
        level: LevelOneDeep {
            namespace: "core::services::auth".into(),
            depth: LevelTwo {
                key: "region:asia-pacific".into(),
                depth: LevelThree {
                    token: "branch/☕️".into(),
                    depth: LevelFour {
                        code: "lf-Δ".into(),
                        depth: LevelFive {
                            message: "depth-five 🚀".into(),
                            ordinal: -7,
                        },
                    },
                },
            },
        },
        peers: vec![
            NetworkPeer {
                host: "alpha.example.com".into(),
                port: 443,
                secure: true,
            },
            NetworkPeer {
                host: "[2001:db8::1]".into(),
                port: 8443,
                secure: false,
            },
        ],
        locales,
        breadcrumbs: vec![
            vec!["root".into(), "auth".into(), "callbacks".into()],
            vec!["fallback".into(), "β-branch".into(), "🚀".into()],
        ],
        matrix: vec![vec![-5, -1, 0], vec![0, 1, 2], vec![9, 13, 21]],
        feature_flags: vec![true, false, true, true, false],
        tags: vec![
            " primary ".into(),
            "🔥hot🔥".into(),
            "unicode-✓".into(),
            "line\nbreak".into(),
        ],
        metrics: Metrics {
            load: 73.5,
            requests: 120_045,
            trend: vec![0.1, -0.25, 1.75],
        },
        extra_notes: Some(vec![
            "first line".into(),
            "line two with newline\nsplit".into(),
            "emoji ☕️🚀".into(),
        ]),
    }
}

#[test]
fn struct_roundtrip_preserves_data() -> Result<(), SerdeQueryError> {
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

    let map = QueryMap::from_struct(&profile)?;
    let restored: ProfileForm = map.to_struct()?;
    assert_eq!(restored, profile);
    Ok(())
}

#[test]
fn query_map_contains_expected_nested_values() -> Result<(), SerdeQueryError> {
    let profile = ProfileForm {
        username: "grace".into(),
        age: 29,
        active: false,
        contact: ContactForm {
            email: "grace@example.com".into(),
            primary_phone: "+1 555-0100".into(),
            secondary_phone: None,
        },
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
fn sequence_field_roundtrip_preserves_values() -> Result<(), SerdeQueryError> {
    let record = TaggedRecord {
        name: "release".into(),
        tags: vec!["stable".into(), "serde".into()],
    };

    let map = QueryMap::from_struct(&record)?;
    let restored: TaggedRecord = map.to_struct()?;
    assert_eq!(restored, record);
    Ok(())
}

#[test]
fn serialize_sequence_field_creates_array() -> Result<(), SerdeQueryError> {
    let record = TaggedRecord {
        name: "release".into(),
        tags: vec!["stable".into(), "serde".into()],
    };

    let map = QueryMap::from_struct(&record)?;
    let tags = map.get("tags").expect("tags field should exist");
    let tags_array = expect_array(tags);
    assert_eq!(tags_array.len(), 2, "should keep both tags");
    assert_eq!(tags_array[0].as_str(), Some("stable"));
    assert_eq!(tags_array[1].as_str(), Some("serde"));
    Ok(())
}

#[test]
fn to_struct_rejects_unknown_field() {
    let mut map = QueryMap::new();
    map.insert("username".into(), Value::String("ada".into()));
    map.insert("age".into(), Value::String("36".into()));
    map.insert("unexpected".into(), Value::String("boom".into()));

    let result: Result<SimpleUser, SerdeQueryError> = map.to_struct();
    match result {
        Err(SerdeQueryError::Deserialize(_)) => {}
        other => panic!("expected Deserialize error, got {other:?}"),
    }
}

#[test]
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

#[test]
fn deeply_nested_struct_roundtrip_preserves_all_fields() -> Result<(), SerdeQueryError> {
    let envelope = build_deep_envelope();
    let map = QueryMap::from_struct(&envelope)?;
    let restored: DeepEnvelope = map.to_struct()?;
    assert_eq!(restored, envelope);
    Ok(())
}

#[test]
fn deeply_nested_query_map_shape_matches_struct() -> Result<(), SerdeQueryError> {
    let envelope = build_deep_envelope();
    let map = QueryMap::from_struct(&envelope)?;

    assert_str_entry(&map, "checksum", "🔥-hash-✓");

    let level = expect_object(map.get("level").expect("missing level object"));
    assert_str_entry(level, "namespace", "core::services::auth");

    let level_two = expect_object(level.get("depth").expect("missing level two"));
    assert_str_entry(level_two, "key", "region:asia-pacific");

    let level_three = expect_object(level_two.get("depth").expect("missing level three"));
    assert_str_entry(level_three, "token", "branch/☕️");

    let level_four = expect_object(level_three.get("depth").expect("missing level four"));
    assert_str_entry(level_four, "code", "lf-Δ");

    let level_five = expect_object(level_four.get("depth").expect("missing level five"));
    assert_str_entry(level_five, "message", "depth-five 🚀");
    assert_str_entry(level_five, "ordinal", "-7");

    let peers = expect_array(map.get("peers").expect("missing peers"));
    assert!(
        peers.len() >= 2,
        "peers array should contain at least two entries"
    );
    let first_peer = expect_object(&peers[0]);
    assert_str_entry(first_peer, "host", "alpha.example.com");
    assert_str_entry(first_peer, "port", "443");
    assert_str_entry(first_peer, "secure", "true");

    let matrix = expect_array(map.get("matrix").expect("missing matrix"));
    assert_eq!(matrix.len(), 3);
    let second_row = expect_array(&matrix[1]);
    assert_eq!(second_row.len(), 3);
    assert_eq!(second_row[0].as_str(), Some("0"));
    assert_eq!(second_row[2].as_str(), Some("2"));

    let locales = expect_object(map.get("locales").expect("missing locales"));
    let korean = expect_object(locales.get("ko-KR").expect("missing ko-KR locale"));
    assert_str_entry(korean, "language", "한국어");
    let greetings = expect_array(korean.get("greetings").expect("missing greetings"));
    assert_eq!(greetings[0].as_str(), Some("안녕하세요"));
    assert_eq!(greetings[1].as_str(), Some("반가워요"));

    let notes = expect_array(map.get("extra_notes").expect("missing extra_notes"));
    assert_eq!(notes.len(), 3);
    assert_eq!(notes[1].as_str(), Some("line two with newline\nsplit"));

    Ok(())
}

#[test]
fn deep_struct_to_struct_rejects_unknown_nested_field() -> Result<(), SerdeQueryError> {
    let envelope = build_deep_envelope();
    let mut map = QueryMap::from_struct(&envelope)?;

    if let Some(Value::Object(level_obj)) = map.get_mut("level") {
        if let Some(Value::Object(level_two)) = level_obj.get_mut("depth") {
            level_two.insert("phantom".into(), Value::String("ghost".into()));
        } else {
            panic!("expected level two object");
        }
    } else {
        panic!("expected level object");
    }

    let result: Result<DeepEnvelope, SerdeQueryError> = map.to_struct();
    match result {
        Err(SerdeQueryError::Deserialize(_)) => Ok(()),
        other => panic!("expected Deserialize error, got {other:?}"),
    }
}

#[test]
fn deep_struct_to_struct_detects_type_mismatch() -> Result<(), SerdeQueryError> {
    let envelope = build_deep_envelope();
    let mut map = QueryMap::from_struct(&envelope)?;

    if let Some(Value::Object(level_obj)) = map.get_mut("level") {
        level_obj.insert(
            "namespace".into(),
            Value::Array(vec![Value::String("oops".into())]),
        );
    } else {
        panic!("expected level object");
    }

    let result: Result<DeepEnvelope, SerdeQueryError> = map.to_struct();
    match result {
        Err(SerdeQueryError::Deserialize(_)) => Ok(()),
        other => panic!("expected Deserialize error, got {other:?}"),
    }
}

#[test]
fn serialize_rejects_non_map_top_level() {
    let primitive: i32 = 42;
    let result: Result<QueryMap, SerdeQueryError> = QueryMap::from_struct(&primitive);
    match result {
        Err(SerdeQueryError::Serialize(_)) => {} // TopLevel error
        other => panic!("expected Serialize error for non-map top-level, got {other:?}"),
    }
}

#[test]
fn serialize_rejects_unsupported_enum_newtype_variant() {
    #[derive(Serialize)]
    enum UnsupportedEnum {
        Variant(String),
    }

    let data = UnsupportedEnum::Variant("test".into());
    let result: Result<QueryMap, SerdeQueryError> = QueryMap::from_struct(&data);
    match result {
        Err(SerdeQueryError::Serialize(_)) => {} // Unsupported error
        other => panic!("expected Serialize error for unsupported enum, got {other:?}"),
    }
}
