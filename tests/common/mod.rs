use bunner_qs::{
    ParseOptions, ParseResult, QsValue, StringifyError, StringifyOptions, StringifyResult, parse,
    parse_with_options, stringify, stringify_with_options,
};
use indexmap::IndexMap;
use serde_json::Value;
use std::time::SystemTime;

pub fn parse_default(input: &str) -> ParseResult<QsValue> {
    parse(input)
}

pub fn parse_with(input: &str, options: ParseOptions) -> ParseResult<QsValue> {
    parse_with_options(input, options)
}

pub fn build_options(configure: impl FnOnce(&mut ParseOptions)) -> ParseOptions {
    let mut options = ParseOptions::default();
    configure(&mut options);
    options
}

pub fn make_object(entries: Vec<(&str, QsValue)>) -> QsValue {
    let mut map: IndexMap<String, QsValue> = IndexMap::new();
    for (key, value) in entries {
        map.insert(key.to_string(), value);
    }
    QsValue::Object(map)
}

pub fn make_array(values: Vec<QsValue>) -> QsValue {
    QsValue::Array(values)
}

pub fn from_json(value: Value) -> QsValue {
    match value {
        Value::Null => QsValue::Null,
        Value::Bool(b) => QsValue::Bool(b),
        Value::Number(n) => QsValue::Number(n.as_f64().unwrap_or_default()),
        Value::String(s) => QsValue::String(s),
        Value::Array(arr) => QsValue::Array(arr.into_iter().map(from_json).collect()),
        Value::Object(obj) => {
            let mut map = IndexMap::new();
            for (key, value) in obj {
                map.insert(key, from_json(value));
            }
            QsValue::Object(map)
        }
    }
}

pub fn bytes(data: &[u8]) -> QsValue {
    QsValue::Bytes(data.to_vec())
}

pub fn js_date(time: SystemTime) -> QsValue {
    QsValue::Date(time)
}

pub fn assert_parse(input: &str, options: Option<ParseOptions>, expected: QsValue) {
    let result = match options {
        Some(opts) => parse_with(input, opts),
        None => parse_default(input),
    };

    match result {
        Ok(actual) => assert_eq!(actual, expected, "unexpected parse result for {input:?}"),
        Err(error) => panic!("parse({input:?}) returned error: {error:?}"),
    }
}

pub fn assert_parse_default(input: &str, expected: QsValue) {
    assert_parse(input, None, expected)
}

pub fn stringify_default(value: &QsValue) -> StringifyResult<String> {
    stringify(value)
}

pub fn stringify_with(value: &QsValue, options: StringifyOptions) -> StringifyResult<String> {
    stringify_with_options(value, options)
}

pub fn build_stringify_options(configure: impl FnOnce(&mut StringifyOptions)) -> StringifyOptions {
    let mut options = StringifyOptions::default();
    configure(&mut options);
    options
}

pub fn assert_stringify_unimplemented(result: StringifyResult<String>) {
    match result {
        Err(StringifyError::Unimplemented) => {}
        Err(error) => panic!("expected stringify to be unimplemented, got error {error:?}"),
        Ok(value) => panic!("expected stringify to be unimplemented, got result {value:?}"),
    }
}
