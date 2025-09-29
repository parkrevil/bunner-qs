use crate::ParseError;
use crate::parsing::arena::{ArenaQueryMap, ArenaValue, ArenaVec, ParseArena};
use hashbrown::hash_map::RawEntryMut;
use smallvec::SmallVec;

use super::container::{arena_ensure_container, arena_initial_container};
use super::pattern_state::PatternState;
use super::segment::{ContainerType, ResolvedSegment, SegmentKind};

fn arena_is_placeholder(value: &ArenaValue<'_>) -> bool {
    matches!(value, ArenaValue::String(s) if s.is_empty())
}

const MAX_CHILD_CAPACITY_HINT: usize = 64;

pub(crate) fn insert_nested_value_arena<'arena>(
    arena: &'arena ParseArena,
    map: &mut ArenaQueryMap<'arena>,
    segments: &[&str],
    value: &'arena str,
    state: &mut PatternState,
) -> Result<(), ParseError> {
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
    arena_build_nested_path(arena, map, &resolved_segments, value, state, root_key)
}

fn arena_build_nested_path<'arena>(
    arena: &'arena ParseArena,
    map: &mut ArenaQueryMap<'arena>,
    segments: &[ResolvedSegment<'_>],
    final_value: &'arena str,
    state: &PatternState,
    root_key: &str,
) -> Result<(), ParseError> {
    let root_segment = segments[0].as_str();
    let root_path = [root_segment];

    let container_type = state
        .container_type(&root_path)
        .unwrap_or(ContainerType::Object);
    let capacity_hint = state
        .child_capacity(&root_path)
        .saturating_add(1)
        .min(MAX_CHILD_CAPACITY_HINT);

    if let Some(existing) = map.get_mut(root_segment) {
        arena_ensure_container(arena, existing, container_type, root_key)?;
    } else {
        let initial = arena_initial_container(arena, container_type, capacity_hint);
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
    };
    arena_set_nested_value(&ctx, root_value, segments, 1, final_value)
}

struct ArenaSetContext<'arena, 'pattern> {
    arena: &'arena ParseArena,
    state: &'pattern PatternState,
    root_key: &'pattern str,
}

fn arena_set_nested_value<'arena>(
    ctx: &ArenaSetContext<'arena, '_>,
    current: &mut ArenaValue<'arena>,
    segments: &[ResolvedSegment<'_>],
    mut depth: usize,
    final_value: &'arena str,
) -> Result<(), ParseError> {
    if depth >= segments.len() {
        return Ok(());
    }

    let mut node = current;
    let mut value_to_set = Some(final_value);
    let mut path: SmallVec<[&str; 16]> = SmallVec::with_capacity(segments.len().min(16));
    path.extend(segments[..depth].iter().map(|segment| segment.as_str()));

    loop {
        let container_hint = ctx.state.container_type(&path);
        if let Some(expected) = container_hint {
            arena_ensure_container(ctx.arena, node, expected, ctx.root_key)?;
        }

        if matches!(node, ArenaValue::String(_)) {
            *node = arena_initial_container(
                ctx.arena,
                container_hint.unwrap_or(ContainerType::Object),
                0,
            );
            continue;
        }

        let segment = segments[depth].as_str();
        let is_last = depth == segments.len() - 1;

        match node {
            // Handle insertion into map structures
            ArenaValue::Map { entries, index } => {
                match handle_map_segment(
                    ctx,
                    entries,
                    index,
                    segments,
                    &mut path,
                    depth,
                    segment,
                    is_last,
                    &mut value_to_set,
                )? {
                    StepOutcome::Complete => return Ok(()),
                    StepOutcome::Descend { next_index } => {
                        node = &mut entries[next_index].1;
                        depth += 1;
                        path.push(segment);
                    }
                }
            }
            // Handle insertion into sequence structures
            ArenaValue::Seq(items) => {
                match handle_seq_segment(
                    ctx,
                    items,
                    segments,
                    &mut path,
                    depth,
                    segment,
                    is_last,
                    &mut value_to_set,
                )? {
                    StepOutcome::Complete => return Ok(()),
                    StepOutcome::Descend { next_index } => {
                        node = &mut items[next_index];
                        depth += 1;
                        path.push(segment);
                    }
                }
            }
            ArenaValue::String(_) => unreachable!(),
        }
    }
}

enum StepOutcome {
    Complete,
    Descend { next_index: usize },
}

#[allow(clippy::too_many_arguments)]
fn handle_map_segment<'arena, S>(
    ctx: &ArenaSetContext<'arena, '_>,
    entries: &mut ArenaVec<'arena, (&'arena str, ArenaValue<'arena>)>,
    index: &mut hashbrown::HashMap<&'arena str, usize, S>,
    segments: &[ResolvedSegment<'_>],
    path: &mut SmallVec<[&str; 16]>,
    depth: usize,
    segment: &str,
    is_last: bool,
    value_to_set: &mut Option<&'arena str>,
) -> Result<StepOutcome, ParseError>
where
    S: std::hash::BuildHasher,
{
    if is_last {
        match index.raw_entry_mut().from_key(segment) {
            RawEntryMut::Occupied(_) => {
                return Err(ParseError::DuplicateKey {
                    key: segment.to_string(),
                });
            }
            RawEntryMut::Vacant(vacant) => {
                let key_ref = ctx.arena.alloc_str(segment);
                let idx = entries.len();
                entries.push((key_ref, ArenaValue::string(value_to_set.take().unwrap())));
                vacant.insert(key_ref, idx);
                return Ok(StepOutcome::Complete);
            }
        }
    }

    let next_kind = segments[depth + 1].kind;
    let next_is_numeric = matches!(next_kind, SegmentKind::Numeric | SegmentKind::Empty);

    let entry_index = match index.raw_entry_mut().from_key(segment) {
        RawEntryMut::Occupied(entry) => *entry.get(),
        RawEntryMut::Vacant(vacant) => {
            let key_ref = ctx.arena.alloc_str(segment);
            let capacity_hint = child_capacity_hint(ctx.state, path, segment)
                .saturating_add(1)
                .min(MAX_CHILD_CAPACITY_HINT);
            let child = if next_is_numeric {
                ArenaValue::seq_with_capacity(ctx.arena, capacity_hint)
            } else {
                ArenaValue::map_with_capacity(ctx.arena, capacity_hint)
            };
            let idx = entries.len();
            entries.push((key_ref, child));
            vacant.insert(key_ref, idx);
            idx
        }
    };

    Ok(StepOutcome::Descend {
        next_index: entry_index,
    })
}

#[allow(clippy::too_many_arguments)]
fn handle_seq_segment<'arena>(
    ctx: &ArenaSetContext<'arena, '_>,
    items: &mut ArenaVec<'arena, ArenaValue<'arena>>,
    segments: &[ResolvedSegment<'_>],
    path: &mut SmallVec<[&str; 16]>,
    depth: usize,
    segment: &str,
    is_last: bool,
    value_to_set: &mut Option<&'arena str>,
) -> Result<StepOutcome, ParseError> {
    let idx = match segments[depth].kind {
        SegmentKind::Numeric | SegmentKind::Empty => {
            segment
                .parse::<usize>()
                .map_err(|_| ParseError::DuplicateKey {
                    key: ctx.root_key.to_string(),
                })?
        }
        SegmentKind::Other => {
            return Err(ParseError::DuplicateKey {
                key: ctx.root_key.to_string(),
            });
        }
    };

    if idx > items.len() {
        return Err(ParseError::DuplicateKey {
            key: ctx.root_key.to_string(),
        });
    }

    if is_last {
        if idx == items.len() {
            items.push(ArenaValue::string(value_to_set.take().unwrap()));
            return Ok(StepOutcome::Complete);
        }
        if !arena_is_placeholder(&items[idx]) {
            return Err(ParseError::DuplicateKey {
                key: segment.to_string(),
            });
        }
        items[idx] = ArenaValue::string(value_to_set.take().unwrap());
        return Ok(StepOutcome::Complete);
    }

    let next_kind = segments[depth + 1].kind;
    let next_is_numeric = matches!(next_kind, SegmentKind::Numeric | SegmentKind::Empty);

    if idx == items.len() {
        let capacity_hint = child_capacity_hint(ctx.state, path, segment)
            .saturating_add(1)
            .min(MAX_CHILD_CAPACITY_HINT);
        let child = if next_is_numeric {
            ArenaValue::seq_with_capacity(ctx.arena, capacity_hint)
        } else {
            ArenaValue::map_with_capacity(ctx.arena, capacity_hint)
        };
        items.push(child);
    }

    if idx < items.len() && matches!(&items[idx], ArenaValue::String(s) if !s.is_empty()) {
        return Err(ParseError::DuplicateKey {
            key: ctx.root_key.to_string(),
        });
    }

    Ok(StepOutcome::Descend { next_index: idx })
}

fn child_capacity_hint(state: &PatternState, path: &[&str], segment: &str) -> usize {
    let mut full_path: SmallVec<[&str; 16]> = SmallVec::with_capacity(path.len() + 1);
    full_path.extend_from_slice(path);
    full_path.push(segment);
    state
        .child_capacity(&full_path)
        .min(MAX_CHILD_CAPACITY_HINT)
}

pub(crate) fn resolve_segments<'a>(
    state: &mut PatternState,
    original: &[&'a str],
) -> Result<SmallVec<[ResolvedSegment<'a>; 16]>, ParseError> {
    if original.len() <= 1 {
        let mut out: SmallVec<[ResolvedSegment<'a>; 16]> = SmallVec::with_capacity(original.len());
        for segment in original {
            out.push(ResolvedSegment::new(std::borrow::Cow::Borrowed(*segment)));
        }
        return Ok(out);
    }

    let mut resolved: SmallVec<[ResolvedSegment<'a>; 16]> = SmallVec::with_capacity(original.len());

    resolved.push(ResolvedSegment::new(std::borrow::Cow::Borrowed(
        original[0],
    )));

    for &segment in &original[1..] {
        let resolved_segment = state.resolve(&resolved, segment, original[0])?;
        resolved.push(ResolvedSegment::new(resolved_segment));
    }

    Ok(resolved)
}
