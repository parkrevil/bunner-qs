use crate::arena::{ArenaQueryMap, ArenaValue, ParseArena};
use crate::{ParseError, ParseResult};
use ahash::AHashMap;
use smallvec::SmallVec;
use std::borrow::Cow;

pub fn parse_key_path(key: &str) -> Vec<&str> {
    let mut segments = Vec::with_capacity(4);
    let mut start = 0usize;
    let mut in_brackets = false;

    for (idx, ch) in key.char_indices() {
        match ch {
            '[' if !in_brackets => {
                if start < idx {
                    segments.push(&key[start..idx]);
                }
                in_brackets = true;
                start = idx + ch.len_utf8();
            }
            ']' if in_brackets => {
                segments.push(&key[start..idx]);
                in_brackets = false;
                start = idx + ch.len_utf8();
            }
            _ => {}
        }
    }

    if start < key.len() {
        segments.push(&key[start..]);
    }

    segments
}

fn arena_is_placeholder(value: &ArenaValue<'_>) -> bool {
    matches!(value, ArenaValue::String(s) if s.is_empty())
}

pub(crate) fn insert_nested_value_arena<'arena>(
    arena: &'arena ParseArena,
    map: &mut ArenaQueryMap<'arena>,
    segments: &[&str],
    value: &'arena str,
    state: &mut PatternState,
    diagnostics: bool,
) -> ParseResult<()> {
    if segments.is_empty() {
        return Ok(());
    }

    let root_key = segments[0];

    if segments.len() == 1 {
        if map.contains_key(root_key) {
            return Err(ParseError::DuplicateKey {
                key: root_key.to_string(),
            });
        }

        map.try_insert_str(arena, root_key, ArenaValue::string(value))
            .map_err(|_| ParseError::DuplicateKey {
                key: root_key.to_string(),
            })?;
        return Ok(());
    }

    let resolved_segments = resolve_segments(state, segments)?;
    arena_build_nested_path(
        arena,
        map,
        &resolved_segments,
        value,
        state,
        root_key,
        diagnostics,
    )
}

fn arena_build_nested_path<'arena>(
    arena: &'arena ParseArena,
    map: &mut ArenaQueryMap<'arena>,
    segments: &[Cow<'_, str>],
    final_value: &'arena str,
    state: &PatternState,
    root_key: &str,
    diagnostics: bool,
) -> ParseResult<()> {
    let root_segment = segments[0].as_ref();
    let root_path = [root_segment];

    let container_type = state
        .container_type(&root_path)
        .unwrap_or(ContainerType::Object);

    if let Some(existing) = map.get_mut(root_segment) {
        arena_ensure_container(arena, existing, container_type, root_key)?;
    } else {
        let initial = arena_initial_container(arena, container_type);
        map.try_insert_str(arena, root_segment, initial)
            .map_err(|_| ParseError::DuplicateKey {
                key: root_key.to_string(),
            })?;
    }

    let root_value = map.get_mut(root_segment).unwrap();
    let ctx = ArenaSetContext {
        arena,
        state,
        root_key,
        diagnostics,
    };
    arena_set_nested_value(&ctx, root_value, segments, 1, final_value)
}

struct ArenaSetContext<'arena, 'pattern> {
    arena: &'arena ParseArena,
    state: &'pattern PatternState,
    root_key: &'pattern str,
    diagnostics: bool,
}

fn arena_initial_container<'arena>(
    arena: &'arena ParseArena,
    container_type: ContainerType,
) -> ArenaValue<'arena> {
    match container_type {
        ContainerType::Array => ArenaValue::seq(arena.alloc_vec()),
        ContainerType::Object => ArenaValue::map(arena),
    }
}

fn arena_ensure_container<'arena>(
    arena: &'arena ParseArena,
    value: &mut ArenaValue<'arena>,
    expected: ContainerType,
    root_key: &str,
) -> ParseResult<()> {
    match expected {
        ContainerType::Array => match value {
            ArenaValue::Seq(_) => Ok(()),
            ArenaValue::Map { .. } => {
                *value = ArenaValue::seq(arena.alloc_vec());
                Ok(())
            }
            ArenaValue::String(_) => Err(ParseError::DuplicateKey {
                key: root_key.to_string(),
            }),
        },
        ContainerType::Object => match value {
            ArenaValue::Map { .. } => Ok(()),
            ArenaValue::Seq(_) => {
                *value = ArenaValue::map(arena);
                Ok(())
            }
            ArenaValue::String(_) => Err(ParseError::DuplicateKey {
                key: root_key.to_string(),
            }),
        },
    }
}

fn arena_set_nested_value<'arena>(
    ctx: &ArenaSetContext<'arena, '_>,
    current: &mut ArenaValue<'arena>,
    segments: &[Cow<'_, str>],
    mut depth: usize,
    final_value: &'arena str,
) -> ParseResult<()> {
    if depth >= segments.len() {
        return Ok(());
    }

    let mut node = current;
    let mut value_to_set = Some(final_value);
    let mut path: SmallVec<[&str; 8]> = SmallVec::with_capacity(segments.len().min(8));
    path.extend(segments[..depth].iter().map(|segment| segment.as_ref()));

    loop {
        let container_hint = ctx.state.container_type(&path);
        if let Some(expected) = container_hint {
            arena_ensure_container(ctx.arena, node, expected, ctx.root_key)?;
        }

        if matches!(node, ArenaValue::String(_)) {
            *node =
                arena_initial_container(ctx.arena, container_hint.unwrap_or(ContainerType::Object));
            continue;
        }

        let segment = segments[depth].as_ref();
        let is_last = depth == segments.len() - 1;

        match node {
            ArenaValue::Map { entries, index } => {
                if is_last {
                    if index.contains_key(segment) {
                        return Err(ParseError::DuplicateKey {
                            key: if ctx.diagnostics {
                                segment.to_string()
                            } else {
                                ctx.root_key.to_string()
                            },
                        });
                    }

                    let key_ref = ctx.arena.alloc_str(segment);
                    let idx = entries.len();
                    entries.push((key_ref, ArenaValue::string(value_to_set.take().unwrap())));
                    index.insert(key_ref, idx);
                    return Ok(());
                }

                let next_kind = SegmentKind::classify(segments[depth + 1].as_ref());
                let next_is_numeric =
                    matches!(next_kind, SegmentKind::Numeric | SegmentKind::Empty);
                let entry_index = if let Some(&idx) = index.get(segment) {
                    idx
                } else {
                    let key_ref = ctx.arena.alloc_str(segment);
                    let child = if next_is_numeric {
                        ArenaValue::seq(ctx.arena.alloc_vec())
                    } else {
                        ArenaValue::map(ctx.arena)
                    };
                    let idx = entries.len();
                    entries.push((key_ref, child));
                    index.insert(key_ref, idx);
                    idx
                };

                let entry_value = &mut entries[entry_index].1;

                node = entry_value;
                depth += 1;
                path.push(segment);
            }
            ArenaValue::Seq(items) => {
                let idx = segment
                    .parse::<usize>()
                    .map_err(|_| ParseError::DuplicateKey {
                        key: ctx.root_key.to_string(),
                    })?;

                if idx > items.len() {
                    return Err(ParseError::DuplicateKey {
                        key: ctx.root_key.to_string(),
                    });
                }

                if is_last {
                    if idx == items.len() {
                        items.push(ArenaValue::string(value_to_set.take().unwrap()));
                        return Ok(());
                    } else if !arena_is_placeholder(&items[idx]) {
                        return Err(ParseError::DuplicateKey {
                            key: if ctx.diagnostics {
                                segment.to_string()
                            } else {
                                ctx.root_key.to_string()
                            },
                        });
                    } else {
                        items[idx] = ArenaValue::string(value_to_set.take().unwrap());
                        return Ok(());
                    }
                }

                let next_kind = SegmentKind::classify(segments[depth + 1].as_ref());
                let next_is_numeric =
                    matches!(next_kind, SegmentKind::Numeric | SegmentKind::Empty);

                if idx == items.len() {
                    let child = if next_is_numeric {
                        ArenaValue::seq(ctx.arena.alloc_vec())
                    } else {
                        ArenaValue::map(ctx.arena)
                    };
                    items.push(child);
                }

                if idx < items.len()
                    && matches!(&items[idx], ArenaValue::String(s) if !s.is_empty())
                {
                    return Err(ParseError::DuplicateKey {
                        key: ctx.root_key.to_string(),
                    });
                }

                node = &mut items[idx];
                depth += 1;
                path.push(segment);
            }
            ArenaValue::String(_) => unreachable!(),
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
    fn descend_mut(&mut self, path: &[&str]) -> &mut PathNode {
        let mut node = self;
        for segment in path {
            if node.children.contains_key(*segment) {
                node = node.children.get_mut(*segment).unwrap();
            } else {
                node = node.children.entry((*segment).to_string()).or_default();
            }
        }
        node
    }

    fn descend(&self, path: &[&str]) -> Option<&PathNode> {
        let mut node = self;
        for segment in path {
            node = node.children.get(*segment)?;
        }
        Some(node)
    }
}

impl PatternState {
    fn resolve<'a>(
        &mut self,
        container_path: &[&str],
        segment: &'a str,
        root_key: &str,
    ) -> ParseResult<Cow<'a, str>> {
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

        match kind {
            SegmentKind::Empty => {
                let idx = node.next_index;
                node.next_index += 1;
                let value = idx.to_string();
                node.children.entry(value.clone()).or_default();
                Ok(Cow::Owned(value))
            }
            _ => {
                if !node.children.contains_key(segment) {
                    node.children
                        .insert(segment.to_string(), PathNode::default());
                }
                Ok(Cow::Borrowed(segment))
            }
        }
    }

    fn container_type(&self, path: &[&str]) -> Option<ContainerType> {
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

fn resolve_segments<'a>(
    state: &mut PatternState,
    original: &[&'a str],
) -> ParseResult<Vec<Cow<'a, str>>> {
    if original.len() <= 1 {
        return Ok(original
            .iter()
            .map(|segment| Cow::Borrowed(*segment))
            .collect());
    }

    let mut resolved: Vec<Cow<'a, str>> = Vec::with_capacity(original.len());
    let mut path_indices: SmallVec<[usize; 8]> = SmallVec::with_capacity(original.len().min(8));

    resolved.push(Cow::Borrowed(original[0]));
    path_indices.push(0);

    for &segment in &original[1..] {
        let resolved_segment = {
            let mut path_refs: SmallVec<[&str; 8]> = SmallVec::with_capacity(path_indices.len());
            for &idx in &path_indices {
                path_refs.push(resolved[idx].as_ref());
            }

            state.resolve(&path_refs, segment, original[0])?
        };

        resolved.push(resolved_segment);
        path_indices.push(resolved.len() - 1);
    }

    Ok(resolved)
}
