use crate::ordered_map::new_map;
use crate::value::{QueryMap, Value};
use crate::{ParseError, ParseResult};
use ahash::AHashMap;

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

pub(crate) fn insert_nested_value(
    map: &mut QueryMap,
    segments: &[String],
    value: String,
    state: &mut PatternState,
    diagnostics: bool,
) -> ParseResult<()> {
    if segments.is_empty() {
        return Ok(());
    }

    let root_key = &segments[0];

    if segments.len() == 1 {
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

    let resolved_segments = resolve_segments(state, segments)?;

    build_nested_path(map, &resolved_segments, value, state, diagnostics)
}

fn build_nested_path(
    map: &mut QueryMap,
    segments: &[String],
    final_value: String,
    state: &PatternState,
    diagnostics: bool,
) -> ParseResult<()> {
    let root_key = &segments[0];

    let container_type = state
        .container_type(&segments[..1])
        .unwrap_or(ContainerType::Object);

    if map.contains_key(root_key) {
        let root_value = map.get_mut(root_key).unwrap();
        ensure_container(root_value, container_type, root_key)?;
    } else {
        map.insert(root_key.clone(), initial_container(container_type));
    }

    let root_value = map.get_mut(root_key).unwrap();
    set_nested_value(
        root_value,
        segments,
        1,
        final_value,
        state,
        root_key,
        diagnostics,
    )
}

fn initial_container(container_type: ContainerType) -> Value {
    match container_type {
        ContainerType::Array => Value::Array(Vec::new()),
        ContainerType::Object => Value::Object(new_map()),
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
                *value = Value::Object(new_map());
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
    segments: &[String],
    mut depth: usize,
    final_value: String,
    state: &PatternState,
    root_key: &str,
    diagnostics: bool,
) -> ParseResult<()> {
    if depth >= segments.len() {
        return Ok(());
    }

    let mut node = current;
    let mut final_value = Some(final_value);

    loop {
        if let Some(expected) = state.container_type(&segments[..depth]) {
            ensure_container(node, expected, root_key)?;
        }

        if matches!(node, Value::String(_)) {
            *node = initial_container(
                state
                    .container_type(&segments[..depth])
                    .unwrap_or(ContainerType::Object),
            );
            continue;
        }

        let segment = &segments[depth];
        let is_last = depth == segments.len() - 1;

        match node {
            Value::Object(obj) => {
                if is_last {
                    if obj.contains_key(segment) {
                        return Err(ParseError::DuplicateKey {
                            key: if diagnostics {
                                segment.clone()
                            } else {
                                root_key.to_string()
                            },
                        });
                    }
                    obj.insert(segment.clone(), Value::String(final_value.take().unwrap()));
                    return Ok(());
                }

                let next_is_numeric = segments[depth + 1].chars().all(|c| c.is_ascii_digit());
                let entry = obj.entry(segment.clone()).or_insert_with(|| {
                    if next_is_numeric {
                        Value::Array(Vec::new())
                    } else {
                        Value::Object(new_map())
                    }
                });

                node = entry;
                depth += 1;
            }
            Value::Array(arr) => {
                let idx = segment
                    .parse::<usize>()
                    .map_err(|_| ParseError::DuplicateKey {
                        key: root_key.to_string(),
                    })?;

                if idx > arr.len() {
                    return Err(ParseError::DuplicateKey {
                        key: root_key.to_string(),
                    });
                }

                if is_last {
                    if idx == arr.len() {
                        arr.push(Value::String(final_value.take().unwrap()));
                    } else if !is_placeholder(&arr[idx]) {
                        return Err(ParseError::DuplicateKey {
                            key: if diagnostics {
                                segment.clone()
                            } else {
                                root_key.to_string()
                            },
                        });
                    } else {
                        arr[idx] = Value::String(final_value.take().unwrap());
                    }
                    return Ok(());
                }

                let next_is_numeric = segments[depth + 1].chars().all(|c| c.is_ascii_digit());

                if idx == arr.len() {
                    arr.push(if next_is_numeric {
                        Value::Array(Vec::new())
                    } else {
                        Value::Object(new_map())
                    });
                }

                if idx < arr.len() && matches!(&arr[idx], Value::String(s) if !s.is_empty()) {
                    return Err(ParseError::DuplicateKey {
                        key: root_key.to_string(),
                    });
                }

                node = &mut arr[idx];
                depth += 1;
            }
            Value::String(_) => unreachable!(),
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

    fn container_type(self) -> ContainerType {
        match self {
            SegmentKind::Empty | SegmentKind::Numeric => ContainerType::Array,
            SegmentKind::Other => ContainerType::Object,
        }
    }
}

#[derive(Debug, Default)]
pub(crate) struct PatternState {
    root: PathNode,
}

#[derive(Debug, Default)]
struct PathNode {
    kind: Option<SegmentKind>,
    next_index: usize,
    children: AHashMap<String, PathNode>,
}

impl PathNode {
    fn descend_mut(&mut self, path: &[String]) -> &mut PathNode {
        let mut node = self;
        for segment in path {
            node = node.children.entry(segment.clone()).or_default();
        }
        node
    }

    fn descend(&self, path: &[String]) -> Option<&PathNode> {
        let mut node = self;
        for segment in path {
            node = node.children.get(segment)?;
        }
        Some(node)
    }
}

impl PatternState {
    fn resolve(
        &mut self,
        container_path: &[String],
        segment: &str,
        root_key: &str,
    ) -> ParseResult<String> {
        let node = self.root.descend_mut(container_path);
        let kind = SegmentKind::classify(segment);

        match node.kind {
            Some(existing) if existing != kind => {
                return Err(ParseError::DuplicateKey {
                    key: root_key.to_string(),
                });
            }
            Some(_) => {}
            None => node.kind = Some(kind),
        }

        let resolved = match kind {
            SegmentKind::Empty => {
                let idx = node.next_index;
                node.next_index += 1;
                idx.to_string()
            }
            _ => segment.to_string(),
        };

        node.children.entry(resolved.clone()).or_default();

        Ok(resolved)
    }

    fn container_type(&self, path: &[String]) -> Option<ContainerType> {
        self.root
            .descend(path)
            .and_then(|node| node.kind.map(|kind| kind.container_type()))
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
