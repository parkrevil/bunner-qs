mod encode;

use self::encode::{encode_key_into, encode_value_into};
use crate::config::StringifyOptions;
use crate::error::{SerdeStringifyError, SerdeStringifyResult, StringifyError, StringifyResult};
use crate::memory::acquire_string;
use crate::model::{QueryMap, Value};
use crate::serde_adapter::to_query_map;
use serde::Serialize;
use smallvec::SmallVec;

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
    let map = to_query_map(data).map_err(SerdeStringifyError::from)?;
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
    let mut stack: SmallVec<[StackItem<'_>; 96]> = SmallVec::with_capacity(map.len().min(96));

    for (key, value) in map.iter().rev() {
        ensure_no_control(key).map_err(|_| StringifyError::InvalidKey {
            key: key.to_owned(),
        })?;
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
            buffer.push('[');
            push_usize_decimal(buffer, index);
            buffer.push(']');
        }
    }
}

fn push_usize_decimal(buffer: &mut String, mut value: usize) {
    if value == 0 {
        buffer.push('0');
        return;
    }

    const MAX_DIGITS: usize = 39; // Enough for 128-bit usize values
    let mut digits = [0u8; MAX_DIGITS];
    let mut pos = MAX_DIGITS;

    while value > 0 {
        pos -= 1;
        digits[pos] = b'0' + (value % 10) as u8;
        value /= 10;
    }

    let slice = &digits[pos..];
    // SAFETY: slice contains only ASCII digit bytes written above.
    buffer.push_str(unsafe { std::str::from_utf8_unchecked(slice) });
}

fn write_pair(
    output: &mut String,
    key: &str,
    value: &str,
    space_as_plus: bool,
    first_pair: &mut bool,
) {
    let base_len = key.len() + value.len();
    let separators = if *first_pair { 1 } else { 2 };
    let available = output.capacity() - output.len();
    let conservative_need = separators + base_len;
    if available < conservative_need {
        let worst_case = separators + base_len.saturating_mul(3);
        output.reserve(worst_case - available);
    }

    if !*first_pair {
        output.push('&');
    } else {
        *first_pair = false;
    }

    encode_key_into(output, key, space_as_plus);
    output.push('=');
    encode_value_into(output, value, space_as_plus);
}
