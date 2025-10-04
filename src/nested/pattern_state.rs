use crate::ParseError;
use ahash::AHashMap;
use std::borrow::Cow;
use std::cell::RefCell;

use super::segment::{ContainerType, ResolvedSegment, SegmentKey, SegmentKind};

thread_local! {
    static PATTERN_STATE_POOL: RefCell<Vec<PatternState>> = const { RefCell::new(Vec::new()) };
}

pub(crate) struct PatternStateGuard {
    state: PatternState,
}

impl PatternStateGuard {
    fn new(mut state: PatternState) -> Self {
        state.reset();
        Self { state }
    }
}

impl std::ops::Deref for PatternStateGuard {
    type Target = PatternState;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

impl std::ops::DerefMut for PatternStateGuard {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}

impl Drop for PatternStateGuard {
    fn drop(&mut self) {
        let state = std::mem::take(&mut self.state);
        PATTERN_STATE_POOL.with(|cell| {
            cell.borrow_mut().push(state);
        });
    }
}

pub(crate) fn acquire_pattern_state() -> PatternStateGuard {
    PATTERN_STATE_POOL.with(|cell| {
        let state = cell.borrow_mut().pop().unwrap_or_default();
        PatternStateGuard::new(state)
    })
}

#[cfg(test)]
#[path = "pattern_state_test.rs"]
mod pattern_state_test;

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
            node.kind = None;
            node.next_index = 0;
            node.children.clear();
            node.dirty = false;
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
