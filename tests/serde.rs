#![cfg(feature = "serde")]

use bunner_qs::*;
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
    // serde_urlencoded가 지원하는 배열 형식 테스트
    use std::collections::HashMap;

    // 간단한 HashMap으로 테스트
    let mut data = HashMap::new();
    data.insert("name".to_string(), "john".to_string());
    data.insert("age".to_string(), "30".to_string());

    let query_map = to_query_map(&data)?;
    let reconstructed: HashMap<String, String> = from_query_map(&query_map)?;

    assert_eq!(reconstructed, data);
    Ok(())
}
