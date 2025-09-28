use crate::parsing::arena::{ArenaQueryMap, ArenaValue, ParseArena};
use crate::{ParseError, ParseResult};
use ahash::AHashMap;
use hashbrown::hash_map::RawEntryMut;
use memchr::memchr;
use smallvec::SmallVec;
use std::borrow::{Borrow, Cow};
use std::cell::RefCell;
use std::fmt;
use std::ops::{Deref, DerefMut};

pub fn parse_key_path(key: &str) -> SmallVec<[&str; 16]> {
    let mut segments: SmallVec<[&str; 16]> = SmallVec::new();
    let bytes = key.as_bytes();
    let mut start = 0usize;
    let mut cursor = 0usize;

    while cursor < bytes.len() {
        if bytes[cursor] == b'[' {
            if start < cursor {
                segments.push(&key[start..cursor]);
            }
            cursor += 1;
            start = cursor;
            if cursor >= bytes.len() {
                break;
            }

            let rel = memchr(b']', &bytes[cursor..]);
            let end = rel.map(|offset| cursor + offset).unwrap_or(bytes.len());
            if start < end {
                segments.push(&key[start..end]);
            } else {
                segments.push("");
            }
            cursor = end.saturating_add(1);
            start = cursor;
        } else {
            cursor += 1;
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
    segments: &[ResolvedSegment<'_>],
    final_value: &'arena str,
    state: &PatternState,
    root_key: &str,
    diagnostics: bool,
) -> ParseResult<()> {
    let root_segment = segments[0].as_str();
    let root_path = [root_segment];

    let container_type = state
        .container_type(&root_path)
        .unwrap_or(ContainerType::Object);
    let capacity_hint = state.child_capacity(&root_path).saturating_add(1);

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
    capacity_hint: usize,
) -> ArenaValue<'arena> {
    match container_type {
        ContainerType::Array => ArenaValue::seq_with_capacity(arena, capacity_hint),
        ContainerType::Object => ArenaValue::map_with_capacity(arena, capacity_hint),
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
                *value = ArenaValue::seq_with_capacity(arena, 0);
                Ok(())
            }
            ArenaValue::String(_) => Err(ParseError::DuplicateKey {
                key: root_key.to_string(),
            }),
        },
        ContainerType::Object => match value {
            ArenaValue::Map { .. } => Ok(()),
            ArenaValue::Seq(_) => {
                *value = ArenaValue::map_with_capacity(arena, 0);
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
    segments: &[ResolvedSegment<'_>],
    mut depth: usize,
    final_value: &'arena str,
) -> ParseResult<()> {
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
            ArenaValue::Map { entries, index } => {
                if is_last {
                    match index.raw_entry_mut().from_key(segment) {
                        RawEntryMut::Occupied(_) => {
                            return Err(ParseError::DuplicateKey {
                                key: if ctx.diagnostics {
                                    segment.to_string()
                                } else {
                                    ctx.root_key.to_string()
                                },
                            });
                        }
                        RawEntryMut::Vacant(vacant) => {
                            let key_ref = ctx.arena.alloc_str(segment);
                            let idx = entries.len();
                            entries
                                .push((key_ref, ArenaValue::string(value_to_set.take().unwrap())));
                            vacant.insert(key_ref, idx);
                            return Ok(());
                        }
                    }
                }

                let next_kind = segments[depth + 1].kind;
                let next_is_numeric =
                    matches!(next_kind, SegmentKind::Numeric | SegmentKind::Empty);

                let entry_index = match index.raw_entry_mut().from_key(segment) {
                    RawEntryMut::Occupied(entry) => *entry.get(),
                    RawEntryMut::Vacant(vacant) => {
                        let key_ref = ctx.arena.alloc_str(segment);
                        let capacity_hint =
                            child_capacity_hint(ctx.state, &path, segment).saturating_add(1);
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

                let entry_value = &mut entries[entry_index].1;

                node = entry_value;
                depth += 1;
                path.push(segment);
            }
            ArenaValue::Seq(items) => {
                let idx =
                    match segments[depth].kind {
                        SegmentKind::Numeric | SegmentKind::Empty => segment
                            .parse::<usize>()
                            .map_err(|_| ParseError::DuplicateKey {
                                key: ctx.root_key.to_string(),
                            })?,
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

                let next_kind = segments[depth + 1].kind;
                let next_is_numeric =
                    matches!(next_kind, SegmentKind::Numeric | SegmentKind::Empty);

                if idx == items.len() {
                    let capacity_hint =
                        child_capacity_hint(ctx.state, &path, segment).saturating_add(1);
                    let child = if next_is_numeric {
                        ArenaValue::seq_with_capacity(ctx.arena, capacity_hint)
                    } else {
                        ArenaValue::map_with_capacity(ctx.arena, capacity_hint)
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

fn child_capacity_hint(state: &PatternState, path: &[&str], segment: &str) -> usize {
    let mut full_path: SmallVec<[&str; 16]> = SmallVec::with_capacity(path.len() + 1);
    full_path.extend_from_slice(path);
    full_path.push(segment);
    state.child_capacity(&full_path)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SegmentKind {
    Empty,
    Numeric,
    Other,
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct SegmentKey(SmallVec<[u8; 24]>);

impl SegmentKey {
    fn new(segment: &str) -> Self {
        SegmentKey(SmallVec::from_slice(segment.as_bytes()))
    }

    fn as_str(&self) -> &str {
        // SAFETY: All keys originate from UTF-8 input segments.
        unsafe { std::str::from_utf8_unchecked(&self.0) }
    }
}

impl Borrow<[u8]> for SegmentKey {
    fn borrow(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Debug for SegmentKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("SegmentKey").field(&self.as_str()).finish()
    }
}

struct ResolvedSegment<'a> {
    text: Cow<'a, str>,
    kind: SegmentKind,
}

impl<'a> ResolvedSegment<'a> {
    fn new(text: Cow<'a, str>) -> Self {
        let kind = SegmentKind::classify(text.as_ref());
        Self { text, kind }
    }

    fn as_str(&self) -> &str {
        self.text.as_ref()
    }
}

thread_local! {
    static PATTERN_STATE_POOL: RefCell<PatternState> = RefCell::new(PatternState::default());
}

pub(crate) struct PatternStateGuard {
    state: Option<PatternState>,
}

impl PatternStateGuard {
    fn new(mut state: PatternState) -> Self {
        state.reset();
        Self { state: Some(state) }
    }
}

impl Deref for PatternStateGuard {
    type Target = PatternState;

    fn deref(&self) -> &Self::Target {
        self.state.as_ref().expect("pattern state already released")
    }
}

impl DerefMut for PatternStateGuard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.state.as_mut().expect("pattern state already released")
    }
}

impl Drop for PatternStateGuard {
    fn drop(&mut self) {
        if let Some(state) = self.state.take() {
            PATTERN_STATE_POOL.with(|cell| {
                if let Ok(mut slot) = cell.try_borrow_mut() {
                    *slot = state;
                }
            });
        }
    }
}

pub(crate) fn acquire_pattern_state() -> PatternStateGuard {
    PATTERN_STATE_POOL.with(|cell| match cell.try_borrow_mut() {
        Ok(mut stored) => {
            let state = std::mem::take(&mut *stored);
            PatternStateGuard::new(state)
        }
        Err(_) => PatternStateGuard::new(PatternState::default()),
    })
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

#[derive(Debug)]
pub(crate) struct PatternState {
    nodes: Vec<PathNode>,
    dirty_nodes: Vec<usize>,
    free_nodes: Vec<usize>,
}

impl Default for PatternState {
    fn default() -> Self {
        Self {
            nodes: vec![PathNode::default()],
            dirty_nodes: Vec::new(),
            free_nodes: Vec::new(),
        }
    }
}

#[derive(Debug, Default)]
struct PathNode {
    kind: Option<SegmentKind>,
    next_index: usize,
    children: AHashMap<SegmentKey, usize>,
    dirty: bool,
}

impl PatternState {
    fn mark_tracked(&mut self, idx: usize) {
        let node = &mut self.nodes[idx];
        if !node.dirty {
            node.dirty = true;
            self.dirty_nodes.push(idx);
        }
    }

    fn alloc_node(&mut self) -> usize {
        if let Some(idx) = self.free_nodes.pop() {
            let node = &mut self.nodes[idx];
            debug_assert!(!node.dirty);
            node.kind = None;
            node.next_index = 0;
            node.children.clear();
            idx
        } else {
            let idx = self.nodes.len();
            self.nodes.push(PathNode::default());
            idx
        }
    }

    fn ensure_child(&mut self, parent_idx: usize, key: &str) -> usize {
        if let Some(&idx) = self.nodes[parent_idx].children.get(key.as_bytes()) {
            return idx;
        }

        let idx = self.alloc_node();
        self.nodes[parent_idx]
            .children
            .insert(SegmentKey::new(key), idx);
        idx
    }

    fn descend_index(&self, path: &[&str]) -> Option<usize> {
        let mut idx = 0usize;
        for segment in path {
            let node = &self.nodes[idx];
            idx = *node.children.get(segment.as_bytes())?;
        }
        Some(idx)
    }

    fn reset(&mut self) {
        while let Some(idx) = self.dirty_nodes.pop() {
            let node = &mut self.nodes[idx];
            node.kind = None;
            node.next_index = 0;
            node.dirty = false;
            node.children.clear();
            if idx != 0 {
                self.free_nodes.push(idx);
            }
        }
    }

    fn resolve<'a>(
        &mut self,
        container_path: &[ResolvedSegment<'_>],
        segment: &'a str,
        root_key: &str,
    ) -> ParseResult<Cow<'a, str>> {
        let mut current = 0usize;
        self.mark_tracked(current);

        for part in container_path {
            let child_idx = self.ensure_child(current, part.as_str());
            self.mark_tracked(child_idx);
            current = child_idx;
        }

        let kind = SegmentKind::classify(segment);

        let generated = {
            let node = &mut self.nodes[current];
            match node.kind {
                Some(existing) if existing != kind => {
                    return Err(ParseError::DuplicateKey {
                        key: root_key.to_string(),
                    });
                }
                Some(_) => {}
                None => node.kind = Some(kind),
            }

            if let SegmentKind::Empty = kind {
                let idx = node.next_index;
                node.next_index += 1;
                Some(idx.to_string())
            } else {
                None
            }
        };

        if let Some(value) = generated {
            let child_idx = self.ensure_child(current, &value);
            self.mark_tracked(child_idx);
            return Ok(Cow::Owned(value));
        }

        let child_idx = self.ensure_child(current, segment);
        self.mark_tracked(child_idx);
        Ok(Cow::Borrowed(segment))
    }

    fn container_type(&self, path: &[&str]) -> Option<ContainerType> {
        let idx = self.descend_index(path)?;
        self.nodes[idx].kind.map(|kind| kind.container_type())
    }

    fn child_capacity(&self, path: &[&str]) -> usize {
        self.descend_index(path)
            .map(|idx| self.nodes[idx].children.len())
            .unwrap_or(0)
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
) -> ParseResult<SmallVec<[ResolvedSegment<'a>; 16]>> {
    if original.len() <= 1 {
        let mut out: SmallVec<[ResolvedSegment<'a>; 16]> = SmallVec::with_capacity(original.len());
        for segment in original {
            out.push(ResolvedSegment::new(Cow::Borrowed(*segment)));
        }
        return Ok(out);
    }

    let mut resolved: SmallVec<[ResolvedSegment<'a>; 16]> = SmallVec::with_capacity(original.len());

    resolved.push(ResolvedSegment::new(Cow::Borrowed(original[0])));

    for &segment in &original[1..] {
        let resolved_segment = state.resolve(&resolved, segment, original[0])?;
        resolved.push(ResolvedSegment::new(resolved_segment));
    }

    Ok(resolved)
}
