use crate::buffer_pool::acquire_string;
use crate::encoding::{encode_key_into, encode_value_into};
use crate::error::{SerdeStringifyError, SerdeStringifyResult, StringifyError, StringifyResult};
use crate::options::StringifyOptions;
use crate::value::{QueryMap, Value};
use serde::Serialize;

#[derive(Clone, Copy)]
struct StringifyRuntime {
    space_as_plus: bool,
}

impl StringifyRuntime {
    fn new(options: &StringifyOptions) -> Self {
        Self {
            space_as_plus: options.space_as_plus,
        }
    }
}

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

    let runtime = StringifyRuntime::new(options);
    let mut output = String::with_capacity(map.len().saturating_mul(16));
    let mut key_guard = acquire_string();
    let key_buffer = key_guard.as_mut();
    let mut first_pair = true;
    let mut stack = Vec::with_capacity(map.len().max(1));

    for (key, value) in map.iter().rev() {
        ensure_no_control(key).map_err(|_| StringifyError::InvalidKey { key: key.clone() })?;
        stack.push(StackItem {
            parent_len: 0,
            segment: Segment::Root(key),
            value,
        });
    }

    while let Some(item) = stack.pop() {
        let StackItem {
            parent_len,
            segment,
            value,
        } = item;

        key_buffer.truncate(parent_len);
        append_segment(key_buffer, segment);

        match value {
            Value::String(s) => {
                ensure_no_control(s).map_err(|_| StringifyError::InvalidValue {
                    key: key_buffer.clone(),
                })?;
                write_pair(
                    &mut output,
                    key_buffer,
                    s,
                    runtime.space_as_plus,
                    &mut first_pair,
                );
            }
            Value::Array(arr) => {
                let current_len = key_buffer.len();
                for idx in (0..arr.len()).rev() {
                    stack.push(StackItem {
                        parent_len: current_len,
                        segment: Segment::Array(idx),
                        value: &arr[idx],
                    });
                }
            }
            Value::Object(obj) => {
                let current_len = key_buffer.len();
                for (sub_key, sub_value) in obj.iter().rev() {
                    if ensure_no_control(sub_key).is_err() {
                        return Err(StringifyError::InvalidKey {
                            key: format!("{}[{}]", key_buffer, sub_key),
                        });
                    }
                    stack.push(StackItem {
                        parent_len: current_len,
                        segment: Segment::Object(sub_key),
                        value: sub_value,
                    });
                }
            }
        }
    }

    Ok(output)
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

struct StackItem<'a> {
    parent_len: usize,
    segment: Segment<'a>,
    value: &'a Value,
}

#[derive(Clone, Copy)]
enum Segment<'a> {
    Root(&'a str),
    Object(&'a str),
    Array(usize),
}

fn append_segment(buffer: &mut String, segment: Segment<'_>) {
    match segment {
        Segment::Root(key) => buffer.push_str(key),
        Segment::Object(sub_key) => {
            buffer.push('[');
            buffer.push_str(sub_key);
            buffer.push(']');
        }
        Segment::Array(index) => {
            use std::fmt::Write as _;
            buffer.push('[');
            let _ = write!(buffer, "{}", index);
            buffer.push(']');
        }
    }
}

fn write_pair(
    output: &mut String,
    key: &str,
    value: &str,
    space_as_plus: bool,
    first_pair: &mut bool,
) {
    if !*first_pair {
        output.push('&');
    } else {
        *first_pair = false;
    }

    encode_key_into(output, key, space_as_plus);
    output.push('=');
    encode_value_into(output, value, space_as_plus);
}
