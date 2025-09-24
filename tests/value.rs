use bunner_qs::{QueryMap, SingleValueError, from_single_map, to_single_map};
use std::collections::HashMap;

fn map(entries: &[(&str, &[&str])]) -> QueryMap {
    let mut result = QueryMap::new();
    for (key, values) in entries {
        result.insert(
            (*key).to_string(),
            values.iter().map(|v| (*v).to_string()).collect(),
        );
    }
    result
}

#[test]
fn converts_to_single_map() {
    let source = map(&[("a", &["1"]), ("b", &["apple"])]);
    let single = to_single_map(&source).expect("single-value conversion should succeed");
    assert_eq!(single.get("a"), Some(&"1".to_string()));
    assert_eq!(single.get("b"), Some(&"apple".to_string()));
}

#[test]
fn rejects_multiple_values() {
    let source = map(&[("a", &["1", "2"])]);
    let error = to_single_map(&source).expect_err("multiple values should fail conversion");
    match error {
        SingleValueError::MultipleValues { key, count } => {
            assert_eq!(key, "a");
            assert_eq!(count, 2);
        }
    }
}

#[test]
fn builds_from_single_map() {
    let mut single = HashMap::new();
    single.insert("a".to_string(), "1".to_string());
    single.insert("b".to_string(), "two".to_string());

    let map = from_single_map(single);
    assert_eq!(map.get("a"), Some(&vec!["1".to_string()]));
    assert_eq!(map.get("b"), Some(&vec!["two".to_string()]));
}
