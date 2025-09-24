use crate::{Charset, Format, QsValue};
use thiserror::Error;

pub fn merge(target: QsValue, source: QsValue) -> QsValue {
    match (&target, &source) {
        (QsValue::Array(t), QsValue::Array(s)) => {
            let mut result = t.clone();
            result.extend(s.clone());
            QsValue::Array(result)
        }
        (QsValue::Array(t), _) => {
            let mut result = t.clone();
            result.push(source);
            QsValue::Array(result)
        }
        (_, QsValue::Array(s)) => {
            let mut result = vec![target];
            result.extend(s.clone());
            QsValue::Array(result)
        }
        (QsValue::Object(t), QsValue::Object(s)) => {
            let mut result = t.clone();
            for (k, v) in s {
                if let Some(existing) = result.get(k) {
                    result.insert(k.clone(), QsValue::Array(vec![existing.clone(), v.clone()]));
                } else {
                    result.insert(k.clone(), v.clone());
                }
            }
            QsValue::Object(result)
        }
        _ => QsValue::Array(vec![target, source]),
    }
}

pub fn assign(target: &mut QsValue, source: &QsValue) -> QsValue {
    if let QsValue::Object(target_map) = target
        && let QsValue::Object(source_map) = source
    {
        for (key, value) in source_map.iter() {
            target_map.insert(key.clone(), value.clone());
        }
    }

    target.clone()
}

pub fn combine(first: QsValue, second: QsValue) -> QsValue {
    match (&first, &second) {
        (QsValue::Array(a), QsValue::Array(b)) => {
            let mut combined = a.clone();
            combined.extend(b.clone());
            QsValue::Array(combined)
        }
        (QsValue::Array(a), _) => {
            let mut combined = a.clone();
            combined.push(second);
            QsValue::Array(combined)
        }
        (_, QsValue::Array(b)) => {
            let mut combined = Vec::with_capacity(b.len() + 1);
            combined.push(first);
            combined.extend(b.clone());
            QsValue::Array(combined)
        }
        _ => QsValue::Array(vec![first, second]),
    }
}

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("utils::decode is not yet implemented")]
    Unimplemented,
}

pub type DecodeResult<T> = Result<T, DecodeError>;

pub fn decode(input: &str, charset: Option<Charset>) -> DecodeResult<String> {
    let _ = (input, charset);
    Err(DecodeError::Unimplemented)
}

#[derive(Debug, Error)]
pub enum EncodeError {
    #[error("utils::encode is not yet implemented")]
    Unimplemented,
    #[error("{0}")]
    TypeError(String),
}

pub type EncodeResult<T> = Result<T, EncodeError>;

pub fn encode(
    value: &QsValue,
    charset: Option<Charset>,
    format: Option<Format>,
) -> EncodeResult<QsValue> {
    let _ = (value, charset, format);
    Err(EncodeError::Unimplemented)
}

pub fn is_buffer(value: &QsValue) -> bool {
    matches!(value, QsValue::Bytes(_))
}

pub fn is_reg_exp(value: &QsValue) -> bool {
    let _ = value;
    false
}
