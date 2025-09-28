use crate::ParseError;
use ahash::AHashMap;
use std::borrow::Cow;
use std::cell::RefCell;

use super::segment::{ContainerType, ResolvedSegment, SegmentKey, SegmentKind};

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

impl std::ops::Deref for PatternStateGuard {
    type Target = PatternState;

    fn deref(&self) -> &Self::Target {
        self.state.as_ref().expect("pattern state already released")
    }
}

impl std::ops::DerefMut for PatternStateGuard {
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

    pub(crate) fn resolve<'a>(
        &mut self,
        container_path: &[ResolvedSegment<'_>],
        segment: &'a str,
        root_key: &str,
    ) -> Result<Cow<'a, str>, ParseError> {
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

    pub(crate) fn container_type(&self, path: &[&str]) -> Option<ContainerType> {
        let idx = self.descend_index(path)?;
        self.nodes[idx].kind.map(|kind| kind.container_type())
    }

    pub(crate) fn child_capacity(&self, path: &[&str]) -> usize {
        self.descend_index(path)
            .map(|idx| self.nodes[idx].children.len())
            .unwrap_or(0)
    }
}