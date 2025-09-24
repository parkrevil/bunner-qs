use crate::QueryMap;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SingleValueError {
    #[error("key '{key}' has {count} values; expected exactly one")]
    MultipleValues { key: String, count: usize },
}

pub type SingleValueResult<T> = Result<T, SingleValueError>;

pub fn to_single_map(map: &QueryMap) -> SingleValueResult<HashMap<String, String>> {
    let mut single = HashMap::with_capacity(map.len());

    for (key, values) in map.iter() {
        match values.as_slice() {
            [] => {
                single.insert(key.clone(), String::new());
            }
            [value] => {
                single.insert(key.clone(), value.clone());
            }
            slice => {
                return Err(SingleValueError::MultipleValues {
                    key: key.clone(),
                    count: slice.len(),
                });
            }
        }
    }

    Ok(single)
}

pub fn from_single_map<I, K, V>(iter: I) -> QueryMap
where
    I: IntoIterator<Item = (K, V)>,
    K: Into<String>,
    V: Into<String>,
{
    iter.into_iter()
        .fold(QueryMap::new(), |mut acc, (key, value)| {
            acc.entry(key.into()).or_default().push(value.into());
            acc
        })
}
