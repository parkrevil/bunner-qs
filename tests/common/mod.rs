use bunner_qs::{QueryMap, Value};use bunner_qs::{

use std::fs;    ParseOptions, ParseResult, QsValue, StringifyError, StringifyOptions, StringifyResult, parse,

use std::path::{Path, PathBuf};    parse_with_options, stringify, stringify_with_options,

};

/// Build a `QueryMap` from key/value string pairs.use indexmap::IndexMap;

pub fn map_from_pairs(pairs: &[(&str, &str)]) -> QueryMap {use serde_json::Value;

    let mut map = QueryMap::new();use std::time::SystemTime;

    for (key, value) in pairs {

        map.insert((*key).to_owned(), Value::String((*value).to_owned()));pub fn parse_default(input: &str) -> ParseResult<QsValue> {

    }    parse(input)

    map}

}

pub fn parse_with(input: &str, options: ParseOptions) -> ParseResult<QsValue> {

/// Assert that a map contains a string entry equal to `expected`.    parse_with_options(input, options)

pub fn assert_str_entry(map: &QueryMap, key: &str, expected: &str) {}

    let value = map

        .get(key)pub fn build_options(configure: impl FnOnce(&mut ParseOptions)) -> ParseOptions {

        .unwrap_or_else(|| panic!("missing key `{key}` in QueryMap"));    let mut options = ParseOptions::default();

    assert_eq!(value.as_str().unwrap_or_else(|| panic!(    configure(&mut options);

        "value for `{key}` was not a string: {value:?}"    options

    )), expected);}

}

pub fn make_object(entries: Vec<(&str, QsValue)>) -> QsValue {

/// Assert that the provided value is an array of strings matching `expected`.    let mut map: IndexMap<String, QsValue> = IndexMap::new();

pub fn assert_string_array(value: &Value, expected: &[&str]) {    for (key, value) in entries {

    match value {        map.insert(key.to_string(), value);

        Value::Array(items) => {    }

            assert_eq!(items.len(), expected.len(), "array length mismatch");    QsValue::Object(map)

            for (idx, expected_value) in expected.iter().enumerate() {}

                let actual = items[idx]

                    .as_str()pub fn make_array(values: Vec<QsValue>) -> QsValue {

                    .unwrap_or_else(|| panic!("array index {idx} not a string"));    QsValue::Array(values)

                assert_eq!(actual, *expected_value, "array value mismatch at index {idx}");}

            }

        }pub fn from_json(value: Value) -> QsValue {

        other => panic!("expected array value, got {other:?}"),    match value {

    }        Value::Null => QsValue::Null,

}        Value::Bool(b) => QsValue::Bool(b),

        Value::Number(n) => QsValue::Number(n.as_f64().unwrap_or_default()),

/// Assert that the provided value is an object and return a reference to it.        Value::String(s) => QsValue::String(s),

pub fn expect_object<'a>(value: &'a Value) -> &'a QueryMap {        Value::Array(arr) => QsValue::Array(arr.into_iter().map(from_json).collect()),

    value        Value::Object(obj) => {

        .as_object()            let mut map = IndexMap::new();

        .unwrap_or_else(|| panic!("expected object value, got {value:?}"))            for (key, value) in obj {

}                map.insert(key, from_json(value));

            }

/// Assert that the provided value is an array and return a reference to it.            QsValue::Object(map)

pub fn expect_array<'a>(value: &'a Value) -> &'a [Value] {        }

    value    }

        .as_array()}

        .unwrap_or_else(|| panic!("expected array value, got {value:?}"))

}pub fn bytes(data: &[u8]) -> QsValue {

    QsValue::Bytes(data.to_vec())

/// Load a fixture file from `tests/data` as a string.}

pub fn load_fixture(path: &str) -> String {

    let full_path = fixture_root().join(path);pub fn js_date(time: SystemTime) -> QsValue {

    fs::read_to_string(&full_path).unwrap_or_else(|err| {    QsValue::Date(time)

        panic!(}

            "failed to read fixture `{}`: {}",

            full_path.display(),pub fn assert_parse(input: &str, options: Option<ParseOptions>, expected: QsValue) {

            err    let result = match options {

        )        Some(opts) => parse_with(input, opts),

    })        None => parse_default(input),

}    };



/// Load a JSON fixture (requires the `serde` feature).    match result {

#[cfg(feature = "serde")]        Ok(actual) => assert_eq!(actual, expected, "unexpected parse result for {input:?}"),

pub fn load_json_fixture<T>(path: &str) -> T        Err(error) => panic!("parse({input:?}) returned error: {error:?}"),

where    }

    T: serde::de::DeserializeOwned,}

{

    let contents = load_fixture(path);pub fn assert_parse_default(input: &str, expected: QsValue) {

    serde_json::from_str(&contents).unwrap_or_else(|err| {    assert_parse(input, None, expected)

        panic!("failed to parse JSON fixture `{path}`: {err}")}

    })

}pub fn stringify_default(value: &QsValue) -> StringifyResult<String> {

    stringify(value)

fn fixture_root() -> PathBuf {}

    Path::new(env!("CARGO_MANIFEST_DIR"))

        .join("tests")pub fn stringify_with(value: &QsValue, options: StringifyOptions) -> StringifyResult<String> {

        .join("data")    stringify_with_options(value, options)

}}


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
