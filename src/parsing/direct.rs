use ahash::AHashSet;
use serde::de::DeserializeOwned;

use crate::error::{ParseError, ParseResult};

use super::key_path::estimate_param_capacity;
use super::runtime::ParseRuntime;

pub(crate) fn try_parse_direct<T>(trimmed: &str, runtime: &ParseRuntime) -> Option<ParseResult<T>>
where
    T: DeserializeOwned + Default,
{
    if !runtime.serde_fastpath {
        return None;
    }

    let bytes = trimmed.as_bytes();
    if bytes.iter().any(|b| matches!(b, b'[' | b']' | b'%')) {
        return None;
    }

    if !runtime.space_as_plus && bytes.contains(&b'+') {
        return None;
    }

    let mut pairs = 0usize;
    let mut seen: AHashSet<&str> = AHashSet::with_capacity(estimate_param_capacity(trimmed));

    for segment in trimmed.split('&') {
        if segment.is_empty() {
            return None;
        }

        pairs += 1;
        if let Some(limit) = runtime.max_params
            && pairs > limit
        {
            return Some(Err(ParseError::TooManyParameters {
                limit,
                actual: pairs,
            }));
        }

        let (key, _) = match segment.split_once('=') {
            Some((key, value)) => (key, value),
            None => (segment, ""),
        };

        if key.is_empty() {
            return None;
        }

        if !seen.insert(key) {
            return Some(Err(ParseError::DuplicateKey {
                key: key.to_string(),
            }));
        }
    }

    match serde_urlencoded::from_str::<T>(trimmed) {
        Ok(value) => Some(Ok(value)),
        Err(_) => None,
    }
}
