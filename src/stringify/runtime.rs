use super::{StringifyError, StringifyResult};
use crate::config::StringifyOptions;
use crate::memory::{acquire_string, buffer::StringGuard};
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

    let PreparedState {
        runtime,
        mut output,
        mut key_guard,
        mut stack,
    } = prepare_stringify_state(map, options)?;

    let mut first_pair = true;
    process_pairs(
        runtime,
        &mut stack,
        key_guard.as_mut(),
        &mut output,
        &mut first_pair,
    )?;

    Ok(output)
}

struct PreparedState<'map> {
    runtime: StringifyRuntime,
    output: String,
    key_guard: StringGuard,
    stack: SmallVec<[StackItem<'map>; 96]>,
}

fn prepare_stringify_state<'map>(
    map: &'map QueryMap,
    options: &StringifyOptions,
) -> StringifyResult<PreparedState<'map>> {
    debug_assert!(
        !map.is_empty(),
        "prepare_stringify_state should not be called with an empty map",
    );

    let runtime = StringifyRuntime::new(options);
    let output = String::with_capacity(map.len().saturating_mul(16));
    let key_guard = acquire_string();
    let mut stack: SmallVec<[StackItem<'map>; 96]> = SmallVec::with_capacity(map.len().min(96));

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

    Ok(PreparedState {
        runtime,
        output,
        key_guard,
        stack,
    })
}

fn process_pairs(
    runtime: StringifyRuntime,
    stack: &mut SmallVec<[StackItem<'_>; 96]>,
    key_buffer: &mut String,
    output: &mut String,
    first_pair: &mut bool,
) -> StringifyResult<()> {
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
                write_pair(output, key_buffer, s, runtime.space_as_plus, first_pair);
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

    Ok(())
}
