use crate::value::{QueryMap, Value};
use crate::{ParseError, ParseResult};
use indexmap::IndexMap;
use std::collections::HashMap;

/// Parse a key with bracket notation into path segments
/// e.g., "foo[bar][0]" -> ["foo", "bar", "0"]
pub fn parse_key_path(key: &str) -> Vec<String> {
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut in_brackets = false;
    let chars = key.chars();

    for ch in chars {
        match ch {
            '[' if !in_brackets => {
                if !current.is_empty() {
                    segments.push(std::mem::take(&mut current));
                }
                in_brackets = true;
            }
            ']' if in_brackets => {
                segments.push(std::mem::take(&mut current));
                in_brackets = false;
            }
            _ => {
                current.push(ch);
            }
        }
    }

    if !current.is_empty() {
        segments.push(current);
    }

    segments
}

fn is_placeholder(value: &Value) -> bool {
    matches!(value, Value::String(s) if s.is_empty())
}

/// Insert a value into nested structure based on path segments
pub(crate) fn insert_nested_value(
    map: &mut QueryMap,
    segments: &[String],
    value: String,
    state: &mut PatternState,
) -> ParseResult<()> {
    if segments.is_empty() {
        return Ok(());
    }

    let root_key = &segments[0];

    if segments.len() == 1 {
        // Simple key without nesting
        match map.get_mut(root_key) {
            Some(_) => {
                return Err(ParseError::DuplicateKey {
                    key: root_key.clone(),
                });
            }
            None => {
                map.insert(root_key.clone(), Value::String(value));
            }
        }
        return Ok(());
    }

    // Resolve segments to enforce array patterns and detect mixing
    let resolved_segments = resolve_segments(state, segments)?;

    // Build nested path iteratively
    build_nested_path(map, &resolved_segments, value, state)
}

fn build_nested_path(
    map: &mut QueryMap,
    segments: &[String],
    final_value: String,
    state: &PatternState,
) -> ParseResult<()> {
    let root_key = &segments[0];

    let container_path = vec![root_key.clone()];
    let container_type = state
        .container_type(&container_path)
        .unwrap_or(ContainerType::Object);

    // Create the root entry if it doesn't exist
    if map.contains_key(root_key) {
        let root_value = map.get_mut(root_key).unwrap();
        ensure_container(root_value, container_type, root_key)?;
    } else {
        map.insert(root_key.clone(), initial_container(container_type));
    }

    // Build path recursively
    set_nested_value(
        map.get_mut(root_key).unwrap(),
        &segments[1..],
        final_value,
        state,
        root_key,
        container_path,
    )
}

fn initial_container(container_type: ContainerType) -> Value {
    match container_type {
        ContainerType::Array => Value::Array(Vec::new()),
        ContainerType::Object => Value::Object(IndexMap::new()),
    }
}

fn ensure_container(value: &mut Value, expected: ContainerType, root_key: &str) -> ParseResult<()> {
    match expected {
        ContainerType::Array => {
            if matches!(value, Value::Array(_)) {
                Ok(())
            } else if matches!(value, Value::Object(_)) {
                *value = Value::Array(Vec::new());
                Ok(())
            } else {
                Err(ParseError::DuplicateKey {
                    key: root_key.to_string(),
                })
            }
        }
        ContainerType::Object => {
            if matches!(value, Value::Object(_)) {
                Ok(())
            } else if matches!(value, Value::Array(_)) {
                *value = Value::Object(IndexMap::new());
                Ok(())
            } else {
                Err(ParseError::DuplicateKey {
                    key: root_key.to_string(),
                })
            }
        }
    }
}

fn set_nested_value(
    current: &mut Value,
    path: &[String],
    final_value: String,
    state: &PatternState,
    root_key: &str,
    current_path: Vec<String>,
) -> ParseResult<()> {
    if path.is_empty() {
        return Ok(());
    }

    if let Some(expected) = state.container_type(&current_path) {
        ensure_container(current, expected, root_key)?;
    }

    if path.len() == 1 {
        let segment = &path[0];
        match current {
            Value::Object(obj) => {
                if obj.contains_key(segment) {
                    return Err(ParseError::DuplicateKey {
                        key: segment.clone(),
                    });
                }
                obj.insert(segment.clone(), Value::String(final_value));
            }
            Value::Array(arr) => {
                if let Ok(idx) = segment.parse::<usize>() {
                    if idx > arr.len() {
                        return Err(ParseError::DuplicateKey {
                            key: root_key.to_string(),
                        });
                    }

                    if idx == arr.len() {
                        arr.push(Value::String(String::new()));
                    } else if !is_placeholder(&arr[idx]) {
                        return Err(ParseError::DuplicateKey {
                            key: segment.clone(),
                        });
                    }

                    arr[idx] = Value::String(final_value);
                } else {
                    return Err(ParseError::DuplicateKey {
                        key: root_key.to_string(),
                    });
                }
            }
            Value::String(_) => {
                *current = Value::Object(IndexMap::new());
                if let Value::Object(obj) = current {
                    obj.insert(segment.clone(), Value::String(final_value));
                }
            }
        }
        return Ok(());
    }

    let segment = &path[0];
    let remaining_path = &path[1..];
    let mut next_path = current_path.clone();
    next_path.push(segment.clone());

    let next_is_numeric = remaining_path
        .first()
        .map(|s| s.chars().all(|c| c.is_ascii_digit()))
        .unwrap_or(false);

    match current {
        Value::Object(obj) => {
            let entry = obj.entry(segment.clone()).or_insert_with(|| {
                if next_is_numeric {
                    Value::Array(Vec::new())
                } else {
                    Value::Object(IndexMap::new())
                }
            });
            if let Some(expected) = state.container_type(&next_path) {
                ensure_container(entry, expected, root_key)?;
            }
            set_nested_value(
                entry,
                remaining_path,
                final_value,
                state,
                root_key,
                next_path,
            )
        }
        Value::Array(arr) => {
            if let Ok(idx) = segment.parse::<usize>() {
                if idx > arr.len() {
                    return Err(ParseError::DuplicateKey {
                        key: root_key.to_string(),
                    });
                }

                if idx == arr.len() {
                    arr.push(if next_is_numeric {
                        Value::Array(Vec::new())
                    } else {
                        Value::Object(IndexMap::new())
                    });
                }
                if let Some(expected) = state.container_type(&next_path) {
                    ensure_container(&mut arr[idx], expected, root_key)?;
                }
                set_nested_value(
                    &mut arr[idx],
                    remaining_path,
                    final_value,
                    state,
                    root_key,
                    next_path,
                )
            } else {
                Err(ParseError::DuplicateKey {
                    key: root_key.to_string(),
                })
            }
        }
        Value::String(_) => {
            *current = initial_container(
                state
                    .container_type(&current_path)
                    .unwrap_or(ContainerType::Object),
            );
            set_nested_value(current, path, final_value, state, root_key, current_path)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SegmentKind {
    Empty,
    Numeric,
    Other,
}

impl SegmentKind {
    fn classify(segment: &str) -> Self {
        if segment.is_empty() {
            SegmentKind::Empty
        } else if segment.chars().all(|c| c.is_ascii_digit()) {
            SegmentKind::Numeric
        } else {
            SegmentKind::Other
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct PatternState {
    containers: HashMap<Vec<String>, ContainerState>,
}

impl PatternState {
    fn resolve(
        &mut self,
        container_path: &[String],
        segment: &str,
        root_key: &str,
    ) -> ParseResult<String> {
        let kind = SegmentKind::classify(segment);
        let entry = self
            .containers
            .entry(container_path.to_vec())
            .or_insert_with(|| ContainerState::new(kind));

        if entry.kind != kind {
            return Err(ParseError::DuplicateKey {
                key: root_key.to_string(),
            });
        }

        Ok(match kind {
            SegmentKind::Empty => {
                let current = entry.next_index;
                entry.next_index += 1;
                current.to_string()
            }
            _ => segment.to_string(),
        })
    }
}

#[derive(Debug, Clone, Copy)]
struct ContainerState {
    kind: SegmentKind,
    next_index: usize,
}

impl ContainerState {
    fn new(kind: SegmentKind) -> Self {
        Self {
            kind,
            next_index: 0,
        }
    }

    fn container_type(&self) -> ContainerType {
        match self.kind {
            SegmentKind::Empty | SegmentKind::Numeric => ContainerType::Array,
            SegmentKind::Other => ContainerType::Object,
        }
    }
}

impl PatternState {
    fn container_type(&self, path: &[String]) -> Option<ContainerType> {
        self.containers
            .get(path)
            .map(|state| state.container_type())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContainerType {
    Array,
    Object,
}

fn resolve_segments(state: &mut PatternState, original: &[String]) -> ParseResult<Vec<String>> {
    if original.len() <= 1 {
        return Ok(original.to_vec());
    }

    let mut resolved = Vec::with_capacity(original.len());
    resolved.push(original[0].clone());

    for idx in 1..original.len() {
        let container_path = &resolved[..idx];
        let segment = state.resolve(container_path, &original[idx], &original[0])?;
        resolved.push(segment);
    }

    Ok(resolved)
}
