use crate::DuplicateKeyBehavior;
use crate::ParseError;
use crate::parsing::arena::{ArenaQueryMap, ArenaValue, ArenaVec, ParseArena};
use hashbrown::hash_map::RawEntryMut;
use smallvec::SmallVec;

use super::container::{arena_ensure_container, arena_initial_container};
use super::key_path::KEY_PATH_INLINE_SEGMENTS;
use super::pattern_state::PatternState;
use super::segment::{ContainerType, ResolvedSegment, SegmentKind};
use std::cell::Cell;

thread_local! {
    static STRING_PROMOTION_SUPPRESSED: Cell<bool> = const { Cell::new(false) };
}

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
    duplicate_keys: DuplicateKeyBehavior,
) -> Result<(), ParseError> {
    if segments.is_empty() {
        return Ok(());
    }

    let root_key = segments[0];

    if segments.len() == 1 {
        if let Some(existing) = map.get_mut(root_key) {
            return match duplicate_keys {
                DuplicateKeyBehavior::Reject => Err(ParseError::DuplicateKey {
                    key: root_key.to_string(),
                }),
                DuplicateKeyBehavior::FirstWins => Ok(()),
                DuplicateKeyBehavior::LastWins => {
                    *existing = ArenaValue::string(value);
                    Ok(())
                }
            };
        }

        try_insert_or_duplicate(root_key, || {
            map.try_insert_str(arena, root_key, ArenaValue::string(value))
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
        duplicate_keys,
    )
}

fn arena_build_nested_path<'arena>(
    arena: &'arena ParseArena,
    map: &mut ArenaQueryMap<'arena>,
    segments: &[ResolvedSegment<'_>],
    final_value: &'arena str,
    state: &PatternState,
    root_key: &str,
    duplicate_keys: DuplicateKeyBehavior,
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
        try_insert_or_duplicate(root_key, || {
            map.try_insert_str(arena, root_segment, initial)
        })?;
    }

    let root_value = get_root_value(map, root_segment, root_key)?;
    let ctx = ArenaSetContext {
        arena,
        state,
        root_key,
        duplicate_keys,
    };
    arena_set_nested_value(&ctx, root_value, segments, 1, final_value)
}

struct ArenaSetContext<'arena, 'pattern> {
    arena: &'arena ParseArena,
    state: &'pattern PatternState,
    root_key: &'pattern str,
    duplicate_keys: DuplicateKeyBehavior,
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
    let mut path: SmallVec<[&str; KEY_PATH_INLINE_SEGMENTS]> =
        SmallVec::with_capacity(segments.len().min(KEY_PATH_INLINE_SEGMENTS));
    path.extend(segments[..depth].iter().map(|segment| segment.as_str()));

    loop {
        if let NodePreparation::NeedsRetry = prepare_current_node(ctx, node, &path)? {
            continue;
        }

        let segment = segments[depth].as_str();
        let is_last = depth == segments.len() - 1;

        match node {
            ArenaValue::Map { entries, index } => match visit_map_node(
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
                TraversalStep::Complete => return Ok(()),
                TraversalStep::Descend(next_node) => {
                    node = next_node;
                    depth += 1;
                    path.push(segment);
                }
            },
            ArenaValue::Seq(items) => match visit_seq_node(
                ctx,
                items,
                segments,
                &mut path,
                depth,
                segment,
                is_last,
                &mut value_to_set,
            )? {
                TraversalStep::Complete => return Ok(()),
                TraversalStep::Descend(next_node) => {
                    node = next_node;
                    depth += 1;
                    path.push(segment);
                }
            },
            ArenaValue::String(_) => {
                return Err(unexpected_nested_string(ctx.root_key));
            }
        }
    }
}

enum StepOutcome {
    Complete,
    Descend { next_index: usize },
}

#[derive(Debug)]
enum NodePreparation {
    Ready,
    NeedsRetry,
}

#[derive(Debug)]
enum TraversalStep<'node, 'arena> {
    Complete,
    Descend(&'node mut ArenaValue<'arena>),
}

fn prepare_current_node<'arena>(
    ctx: &ArenaSetContext<'arena, '_>,
    node: &mut ArenaValue<'arena>,
    path: &[&str],
) -> Result<NodePreparation, ParseError> {
    let container_hint = ctx.state.container_type(path);
    if let Some(expected) = container_hint {
        arena_ensure_container(ctx.arena, node, expected, ctx.root_key)?;
    }

    if matches!(node, ArenaValue::String(_)) && should_promote_string_node() {
        let container = container_hint.unwrap_or(ContainerType::Object);
        *node = arena_initial_container(ctx.arena, container, 0);
        return Ok(NodePreparation::NeedsRetry);
    }

    Ok(NodePreparation::Ready)
}

#[allow(clippy::too_many_arguments)]
fn handle_map_segment<'arena, S>(
    ctx: &ArenaSetContext<'arena, '_>,
    entries: &mut ArenaVec<'arena, (&'arena str, ArenaValue<'arena>)>,
    index: &mut hashbrown::HashMap<&'arena str, usize, S>,
    segments: &[ResolvedSegment<'_>],
    path: &mut SmallVec<[&str; KEY_PATH_INLINE_SEGMENTS]>,
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
            RawEntryMut::Occupied(entry) => {
                return match ctx.duplicate_keys {
                    DuplicateKeyBehavior::Reject => Err(ParseError::DuplicateKey {
                        key: segment.to_string(),
                    }),
                    DuplicateKeyBehavior::FirstWins => Ok(StepOutcome::Complete),
                    DuplicateKeyBehavior::LastWins => {
                        let idx = *entry.get();
                        let value =
                            value_to_set
                                .take()
                                .ok_or_else(|| ParseError::DuplicateKey {
                                    key: segment.to_string(),
                                })?;
                        entries[idx].1 = ArenaValue::string(value);
                        Ok(StepOutcome::Complete)
                    }
                };
            }
            RawEntryMut::Vacant(vacant) => {
                let key_ref = ctx.arena.alloc_str(segment);
                let idx = entries.len();
                let value = value_to_set
                    .take()
                    .ok_or_else(|| ParseError::DuplicateKey {
                        key: segment.to_string(),
                    })?;
                entries.push((key_ref, ArenaValue::string(value)));
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
fn visit_map_node<'arena, 'node, S>(
    ctx: &ArenaSetContext<'arena, '_>,
    entries: &'node mut ArenaVec<'arena, (&'arena str, ArenaValue<'arena>)>,
    index: &'node mut hashbrown::HashMap<&'arena str, usize, S>,
    segments: &[ResolvedSegment<'_>],
    path: &mut SmallVec<[&str; KEY_PATH_INLINE_SEGMENTS]>,
    depth: usize,
    segment: &str,
    is_last: bool,
    value_to_set: &mut Option<&'arena str>,
) -> Result<TraversalStep<'node, 'arena>, ParseError>
where
    S: std::hash::BuildHasher,
{
    match handle_map_segment(
        ctx,
        entries,
        index,
        segments,
        path,
        depth,
        segment,
        is_last,
        value_to_set,
    )? {
        StepOutcome::Complete => Ok(TraversalStep::Complete),
        StepOutcome::Descend { next_index } => {
            Ok(TraversalStep::Descend(&mut entries[next_index].1))
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn handle_seq_segment<'arena>(
    ctx: &ArenaSetContext<'arena, '_>,
    items: &mut ArenaVec<'arena, ArenaValue<'arena>>,
    segments: &[ResolvedSegment<'_>],
    path: &mut SmallVec<[&str; KEY_PATH_INLINE_SEGMENTS]>,
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
            let value = value_to_set
                .take()
                .ok_or_else(|| ParseError::DuplicateKey {
                    key: segment.to_string(),
                })?;
            items.push(ArenaValue::string(value));
            return Ok(StepOutcome::Complete);
        }
        if !arena_is_placeholder(&items[idx]) {
            return match ctx.duplicate_keys {
                DuplicateKeyBehavior::Reject => Err(ParseError::DuplicateKey {
                    key: segment.to_string(),
                }),
                DuplicateKeyBehavior::FirstWins => Ok(StepOutcome::Complete),
                DuplicateKeyBehavior::LastWins => {
                    let value = value_to_set
                        .take()
                        .ok_or_else(|| ParseError::DuplicateKey {
                            key: segment.to_string(),
                        })?;
                    items[idx] = ArenaValue::string(value);
                    Ok(StepOutcome::Complete)
                }
            };
        }
        let value = value_to_set
            .take()
            .ok_or_else(|| ParseError::DuplicateKey {
                key: segment.to_string(),
            })?;
        items[idx] = ArenaValue::string(value);
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

#[allow(clippy::too_many_arguments)]
fn visit_seq_node<'arena, 'node>(
    ctx: &ArenaSetContext<'arena, '_>,
    items: &'node mut ArenaVec<'arena, ArenaValue<'arena>>,
    segments: &[ResolvedSegment<'_>],
    path: &mut SmallVec<[&str; KEY_PATH_INLINE_SEGMENTS]>,
    depth: usize,
    segment: &str,
    is_last: bool,
    value_to_set: &mut Option<&'arena str>,
) -> Result<TraversalStep<'node, 'arena>, ParseError> {
    match handle_seq_segment(
        ctx,
        items,
        segments,
        path,
        depth,
        segment,
        is_last,
        value_to_set,
    )? {
        StepOutcome::Complete => Ok(TraversalStep::Complete),
        StepOutcome::Descend { next_index } => Ok(TraversalStep::Descend(&mut items[next_index])),
    }
}

fn child_capacity_hint(state: &PatternState, path: &[&str], segment: &str) -> usize {
    let mut full_path: SmallVec<[&str; KEY_PATH_INLINE_SEGMENTS]> =
        SmallVec::with_capacity(path.len() + 1);
    full_path.extend_from_slice(path);
    full_path.push(segment);
    state
        .child_capacity(&full_path)
        .min(MAX_CHILD_CAPACITY_HINT)
}

#[inline]
fn try_insert_or_duplicate<F>(key: &str, insert: F) -> Result<(), ParseError>
where
    F: FnOnce() -> Result<(), ()>,
{
    insert().map_err(|_| ParseError::DuplicateKey {
        key: key.to_string(),
    })
}

#[inline]
fn get_root_value<'arena, 'map>(
    map: &'map mut ArenaQueryMap<'arena>,
    root_segment: &str,
    root_key: &str,
) -> Result<&'map mut ArenaValue<'arena>, ParseError> {
    map.get_mut(root_segment)
        .ok_or_else(|| ParseError::DuplicateKey {
            key: root_key.to_string(),
        })
}

#[inline]
fn unexpected_nested_string(root_key: &str) -> ParseError {
    ParseError::DuplicateKey {
        key: root_key.to_string(),
    }
}

#[inline]
fn should_promote_string_node() -> bool {
    !STRING_PROMOTION_SUPPRESSED.with(|flag| flag.get())
}

#[cfg(test)]
pub(crate) fn with_string_promotion_suppressed<F, R>(operation: F) -> R
where
    F: FnOnce() -> R,
{
    STRING_PROMOTION_SUPPRESSED.with(|flag| {
        let previous = flag.replace(true);
        let result = operation();
        flag.set(previous);
        result
    })
}

pub(crate) fn resolve_segments<'a>(
    state: &mut PatternState,
    original: &[&'a str],
) -> Result<SmallVec<[ResolvedSegment<'a>; KEY_PATH_INLINE_SEGMENTS]>, ParseError> {
    if original.len() <= 1 {
        let mut out: SmallVec<[ResolvedSegment<'a>; KEY_PATH_INLINE_SEGMENTS]> =
            SmallVec::with_capacity(original.len());
        for segment in original {
            out.push(ResolvedSegment::new(std::borrow::Cow::Borrowed(*segment)));
        }
        return Ok(out);
    }

    let mut resolved: SmallVec<[ResolvedSegment<'a>; KEY_PATH_INLINE_SEGMENTS]> =
        SmallVec::with_capacity(original.len());

    resolved.push(ResolvedSegment::new(std::borrow::Cow::Borrowed(
        original[0],
    )));

    for &segment in &original[1..] {
        let resolved_segment = state.resolve(&resolved, segment, original[0])?;
        resolved.push(ResolvedSegment::new(resolved_segment));
    }

    Ok(resolved)
}

#[cfg(test)]
#[path = "insertion_test.rs"]
mod insertion_test;
