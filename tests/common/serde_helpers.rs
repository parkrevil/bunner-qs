use crate::api::{parse_default, parse_query, stringify_default, stringify_with_options};
use bunner_qs_rs::parsing::ParseError;
use bunner_qs_rs::stringify::StringifyError;
use bunner_qs_rs::{ParseOptions, QsParseError, QsStringifyError, StringifyOptions};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Map as JsonMap, Value};

pub fn assert_encoded_contains(encoded: &str, expected: &[&str]) {
    for fragment in expected {
        assert!(
            encoded.contains(fragment),
            "encoded string `{encoded}` should contain `{fragment}`"
        );
    }
}

pub fn assert_parse_roundtrip(input: &str) -> Value {
    let parsed: Value = parse_default(input)
        .unwrap_or_else(|err| panic!("parse should succeed but got: {}", format_parse_error(err)));
    assert_stringify_roundtrip(&parsed)
}

pub fn assert_stringify_roundtrip(value: &Value) -> Value {
    let via_public_api = roundtrip_via_public_api(value).expect("Value round-trip should succeed");
    assert_eq!(
        canonicalize_query_value(&via_public_api),
        canonicalize_query_value(value),
        "public API round-trip should preserve the value"
    );

    let default_stringify = StringifyOptions::default();
    let default_parse = ParseOptions::default();
    assert_stringify_roundtrip_with_options(value, &default_stringify, &default_parse)
}

pub fn assert_stringify_roundtrip_with_options(
    value: &Value,
    stringify_options: &StringifyOptions,
    parse_options: &ParseOptions,
) -> Value {
    let encoded = stringify_with_options(value, stringify_options).unwrap_or_else(|err| {
        panic!(
            "stringify_with should succeed with provided options but got: {}",
            format_stringify_error(err)
        )
    });
    assert_encoded_contains(&encoded, &[]);
    let reparsed: Value = parse_query(&encoded, parse_options).unwrap_or_else(|err| {
        panic!(
            "parse_with should succeed with provided options but got: {}",
            format_parse_error(err)
        )
    });
    assert_eq!(
        canonicalize_query_value(&reparsed),
        canonicalize_query_value(value),
        "value should remain unchanged after round-trip with custom options"
    );
    reparsed
}

fn canonicalize_query_value(value: &Value) -> Value {
    match value {
        Value::Bool(flag) => Value::String(flag.to_string()),
        Value::Number(num) => Value::String(num.to_string()),
        Value::Array(items) => Value::Array(items.iter().map(canonicalize_query_value).collect()),
        Value::Object(map) => {
            let mut object = JsonMap::with_capacity(map.len());
            for (key, val) in map {
                object.insert(key.clone(), canonicalize_query_value(val));
            }
            Value::Object(object)
        }
        Value::Null => Value::Null,
        Value::String(text) => Value::String(text.clone()),
    }
}

pub fn roundtrip_via_public_api<T>(value: &T) -> Result<T, RoundtripError>
where
    T: Serialize + DeserializeOwned + Default + 'static,
{
    let encoded = stringify_default(value)
        .map_err(|err| RoundtripError::from_stringify(into_stringify_error(err)))?;
    let parsed =
        parse_default(&encoded).map_err(|err| RoundtripError::from_parse(into_parse_error(err)))?;
    Ok(parsed)
}

#[derive(Debug)]
pub enum RoundtripError {
    Stringify(StringifyError),
    Parse(ParseError),
}

fn format_parse_error(err: QsParseError) -> String {
    match &err {
        QsParseError::Parse(inner) => inner.to_string(),
        QsParseError::MissingParseOptions => "missing parse options".to_string(),
    }
}

fn format_stringify_error(err: QsStringifyError) -> String {
    match &err {
        QsStringifyError::Stringify(inner) => inner.to_string(),
        QsStringifyError::MissingStringifyOptions => "missing stringify options".to_string(),
    }
}

fn into_parse_error(err: QsParseError) -> ParseError {
    match err {
        QsParseError::Parse(inner) => inner,
        QsParseError::MissingParseOptions => {
            unreachable!("parse options should always be configured")
        }
    }
}

fn into_stringify_error(err: QsStringifyError) -> StringifyError {
    match err {
        QsStringifyError::Stringify(inner) => inner,
        QsStringifyError::MissingStringifyOptions => {
            unreachable!("stringify options should always be configured")
        }
    }
}

impl RoundtripError {
    fn from_parse(err: ParseError) -> Self {
        Self::Parse(err)
    }

    fn from_stringify(err: StringifyError) -> Self {
        Self::Stringify(err)
    }
}

impl std::fmt::Display for RoundtripError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RoundtripError::Stringify(err) => write!(f, "stringify error: {err}"),
            RoundtripError::Parse(err) => write!(f, "parse error: {err}"),
        }
    }
}

impl std::error::Error for RoundtripError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            RoundtripError::Stringify(err) => Some(err),
            RoundtripError::Parse(err) => Some(err),
        }
    }
}
