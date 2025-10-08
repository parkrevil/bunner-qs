use crate::api::stringify_with_options;
use bunner_qs_rs::stringify::StringifyError;
use bunner_qs_rs::{QsStringifyError, StringifyOptions};
use proptest::prelude::*;
use serde_json::{Map as JsonMap, Value};

fn stringify_with<T>(value: &T, options: &StringifyOptions) -> Result<String, StringifyError>
where
    T: serde::Serialize,
{
    match stringify_with_options(value, options) {
        Ok(encoded) => Ok(encoded),
        Err(QsStringifyError::Stringify(err)) => Err(err),
        Err(QsStringifyError::MissingStringifyOptions) => {
            unreachable!("stringify options must be configured before stringifying")
        }
    }
}

pub fn allowed_char() -> impl Strategy<Value = char> {
    prop::char::range('\u{0020}', '\u{10FFFF}').prop_filter("exclude DEL", |c| *c != '\u{007F}')
}

pub fn unicode_value_string() -> impl Strategy<Value = String> {
    prop_oneof![
        prop::collection::vec(allowed_char(), 0..8),
        prop::collection::vec(allowed_char(), 8..24),
        prop::collection::vec(allowed_char(), 24..96),
    ]
    .prop_map(|chars: Vec<char>| chars.into_iter().collect())
}

pub fn string_with_spaces() -> impl Strategy<Value = String> {
    prop::collection::vec(prop::collection::vec(allowed_char(), 1..12), 2..6).prop_map(|segments| {
        segments
            .into_iter()
            .map(|chars| chars.into_iter().collect::<String>())
            .collect::<Vec<String>>()
            .join(" ")
    })
}

pub fn percent_encode(input: &str) -> String {
    input
        .as_bytes()
        .iter()
        .map(|byte| format!("%{:02X}", byte))
        .collect()
}

pub fn form_encode(input: &str) -> String {
    input
        .as_bytes()
        .iter()
        .map(|byte| match byte {
            b' ' => "+".to_string(),
            other => format!("%{:02X}", other),
        })
        .collect()
}

pub fn unicode_key_string() -> impl Strategy<Value = String> {
    prop::collection::vec(
        prop::char::range('\u{0020}', '\u{10FFFF}').prop_filter("exclude DEL and brackets", |c| {
            *c != '\u{007F}' && *c != '[' && *c != ']'
        }),
        1..5,
    )
    .prop_map(|chars| chars.into_iter().collect())
}

pub fn root_key_string() -> impl Strategy<Value = String> {
    const ROOT_CHARS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let choices: Vec<char> = ROOT_CHARS.chars().collect();
    prop::collection::vec(prop::sample::select(choices), 3..8)
        .prop_map(|chars| chars.into_iter().collect())
}

pub fn object_key_string() -> impl Strategy<Value = String> {
    const FIRST: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ";
    const REST: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_";
    let first_choices: Vec<char> = FIRST.chars().collect();
    let rest_choices: Vec<char> = REST.chars().collect();
    (
        prop::sample::select(first_choices),
        prop::collection::vec(prop::sample::select(rest_choices), 0..4),
    )
        .prop_map(|(first, rest)| {
            let mut s = String::new();
            s.push(first);
            for ch in rest {
                s.push(ch);
            }
            s
        })
}

pub fn value_depth(value: &Value) -> usize {
    match value {
        Value::String(_) => 0,
        Value::Array(items) => items.iter().map(value_depth).max().unwrap_or(0) + 1,
        Value::Object(map) => map.values().map(value_depth).max().unwrap_or(0) + 1,
        _ => 0,
    }
}

pub fn root_depth(value: &Value) -> usize {
    match value {
        Value::Object(map) => map.values().map(value_depth).max().unwrap_or(0),
        other => value_depth(other),
    }
}

pub fn estimate_params_value(value: &Value) -> usize {
    match value {
        Value::String(_) => 1,
        Value::Array(items) => items.iter().map(estimate_params_value).sum(),
        Value::Object(map) => map.values().map(estimate_params_value).sum(),
        _ => 1,
    }
}

pub fn estimate_params(value: &Value) -> usize {
    match value {
        Value::Object(map) => map.values().map(estimate_params_value).sum(),
        other => estimate_params_value(other),
    }
}

pub fn total_string_length(value: &Value) -> usize {
    match value {
        Value::String(s) => s.len(),
        Value::Array(items) => items.iter().map(total_string_length).sum(),
        Value::Object(map) => map.values().map(total_string_length).sum(),
        _ => 0,
    }
}

pub fn arb_query_value() -> impl Strategy<Value = Value> {
    let leaf = unicode_value_string().prop_map(Value::String);
    leaf.prop_recursive(5, 64, 8, |inner| {
        let arrays = prop::collection::vec(inner.clone(), 1..6).prop_map(Value::Array);
        let objects = prop::collection::vec((object_key_string(), inner), 1..6).prop_map(|pairs| {
            let mut map = JsonMap::new();
            for (key, value) in pairs {
                map.entry(key).or_insert(value);
            }
            Value::Object(map)
        });
        prop_oneof![arrays, objects]
    })
}

pub fn arb_root_value() -> impl Strategy<Value = Value> {
    prop::collection::vec((root_key_string(), arb_query_value()), 0..6).prop_map(|pairs| {
        let mut map = JsonMap::new();
        for (key, value) in pairs {
            map.entry(key).or_insert(value);
        }
        Value::Object(map)
    })
}

#[derive(Debug, Clone)]
pub struct RoundTripConfig {
    pub space_as_plus: bool,
    pub max_params: Option<usize>,
    pub max_length: Option<usize>,
    pub max_depth: Option<usize>,
}

pub fn arb_roundtrip_input() -> impl Strategy<Value = (Value, RoundTripConfig)> {
    arb_root_value().prop_flat_map(|value| {
        let depth = root_depth(&value);
        let params = estimate_params(&value);
        let seed = value.clone();
        (
            any::<bool>(),
            prop::option::of(0usize..5),
            any::<bool>(),
            any::<bool>(),
        )
            .prop_map(
                move |(space_as_plus, extra_params, use_length, use_depth)| {
                    let max_params = extra_params.map(|extra| {
                        let limit = params.saturating_add(extra);
                        limit.max(1)
                    });
                    let estimated_len = stringify_with(&seed, &StringifyOptions::default())
                        .map(|encoded| encoded.len())
                        .unwrap_or(0);
                    let max_length = if use_length {
                        Some(estimated_len + 512)
                    } else {
                        None
                    };
                    let max_depth = if use_depth { Some(depth + 2) } else { None };
                    (
                        seed.clone(),
                        RoundTripConfig {
                            space_as_plus,
                            max_params,
                            max_length,
                            max_depth,
                        },
                    )
                },
            )
    })
}
