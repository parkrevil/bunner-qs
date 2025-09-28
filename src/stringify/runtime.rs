use crate::config::StringifyOptions;
use super::{StringifyError, StringifyResult};
use crate::memory::acquire_string;
use crate::model::{QueryMap, Value};
use smallvec::SmallVec;

use super::validate::ensure_no_control;
use super::walker::{Segment, StackItem, append_segment};
use super::writer::write_pair;

#[derive(Clone, Copy)]
pub(crate) struct StringifyRuntime {
    pub(crate) space_as_plus: bool,
}

impl StringifyRuntime {
    pub(crate) fn new(options: &StringifyOptions) -> Self {
        Self {
            space_as_plus: options.space_as_plus,
        }
    }
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
