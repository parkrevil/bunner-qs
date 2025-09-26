use crate::encoding::{encode_key, encode_value};
use crate::error::{SerdeStringifyError, SerdeStringifyResult, StringifyError, StringifyResult};
use crate::options::StringifyOptions;
use crate::value::{QueryMap, Value};
use serde::Serialize;

pub fn stringify<T>(data: &T) -> SerdeStringifyResult<String>
where
    T: Serialize,
{
    stringify_with(data, &StringifyOptions::default())
}

pub fn stringify_with<T>(data: &T, options: &StringifyOptions) -> SerdeStringifyResult<String>
where
    T: Serialize,
{
    let map = QueryMap::from_struct(data).map_err(SerdeStringifyError::from)?;
    stringify_query_map_with(&map, options).map_err(SerdeStringifyError::from)
}

pub(crate) fn stringify_query_map_with(
    map: &QueryMap,
    options: &StringifyOptions,
) -> StringifyResult<String> {
    if map.is_empty() {
        return Ok(String::new());
    }

    let mut pairs = Vec::new();

    for (key, value) in map.iter() {
        ensure_no_control(key).map_err(|_| StringifyError::InvalidKey { key: key.clone() })?;

        flatten_value(key, value, &mut pairs, options.space_as_plus)?;
    }

    Ok(pairs.join("&"))
}

fn flatten_value(
    base_key: &str,
    value: &Value,
    pairs: &mut Vec<String>,
    space_as_plus: bool,
) -> StringifyResult<()> {
    match value {
        Value::String(s) => {
            ensure_no_control(s).map_err(|_| StringifyError::InvalidValue {
                key: base_key.to_string(),
            })?;
            let encoded_key = encode_key(base_key, space_as_plus);
            let encoded_value = encode_value(s, space_as_plus);
            pairs.push(format!("{}={}", encoded_key, encoded_value));
        }
        Value::Array(arr) => {
            for (idx, item) in arr.iter().enumerate() {
                let key = format!("{}[{}]", base_key, idx);
                flatten_value(&key, item, pairs, space_as_plus)?;
            }
        }
        Value::Object(obj) => {
            for (sub_key, sub_value) in obj.iter() {
                let key = format!("{}[{}]", base_key, sub_key);
                flatten_value(&key, sub_value, pairs, space_as_plus)?;
            }
        }
    }
    Ok(())
}

fn ensure_no_control(value: &str) -> Result<(), ()> {
    if value
        .chars()
        .any(|ch| matches!(ch, '\u{0000}'..='\u{001F}' | '\u{007F}'))
    {
        Err(())
    } else {
        Ok(())
    }
}
