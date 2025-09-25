#![cfg(feature = "serde")]

use bunner_qs::*;
use bunner_qs::Value;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct Account {
    username: String,
    age: u8,
    active: bool,
}

#[test]
fn round_trips_struct() -> Result<(), SerdeQueryError> {
    let account = Account {
        username: "jill".into(),
        age: 27,
        active: true,
    };

    let map = to_query_map(&account)?;
    let decoded: Account = from_query_map(&map)?;

    assert_eq!(decoded, account);
    Ok(())
}

#[test]
fn handles_sequences() -> Result<(), SerdeQueryError> {
    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct Tags {
        tag: Vec<String>,
    }

    let source = QueryMap::from([(
        "tag".to_string(),
        Value::Array(vec![
            Value::String("rust".to_string()),
            Value::String("serde".to_string()),
        ]),
    )]);

    let tags: Tags = from_query_map(&source)?;
    assert_eq!(tags.tag, vec!["rust", "serde"]);

    let rebuilt = to_query_map(&tags)?;
    assert_eq!(
        rebuilt.get("tag"),
        Some(&Value::Array(vec![
            Value::String("rust".into()),
            Value::String("serde".into())
        ]))
    );

    Ok(())
}
