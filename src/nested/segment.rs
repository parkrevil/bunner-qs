use crate::ParseError;
use smallvec::SmallVec;
use std::borrow::{Borrow, Cow};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SegmentKind {
    Empty,
    Numeric,
    Other,
}

pub(crate) const SEGMENT_KEY_INLINE_CAPACITY: usize = 24;

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct SegmentKey(SmallVec<[u8; SEGMENT_KEY_INLINE_CAPACITY]>);

impl SegmentKey {
    pub(crate) fn new(segment: &str) -> Self {
        SegmentKey(SmallVec::from_slice(segment.as_bytes()))
    }

    pub(crate) fn as_str(&self) -> Result<&str, ParseError> {
        std::str::from_utf8(&self.0).map_err(|_| ParseError::InvalidUtf8)
    }
}

impl Borrow<[u8]> for SegmentKey {
    fn borrow(&self) -> &[u8] {
        &self.0
    }
}

impl fmt::Debug for SegmentKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.as_str() {
            Ok(text) => f.debug_tuple("SegmentKey").field(&text).finish(),
            Err(_) => f
                .debug_tuple("SegmentKey")
                .field(&format_args!("<invalid utf-8: {:?}>", &self.0))
                .finish(),
        }
    }
}

pub(crate) struct ResolvedSegment<'a> {
    text: Cow<'a, str>,
    pub(crate) kind: SegmentKind,
}

impl<'a> ResolvedSegment<'a> {
    pub(crate) fn new(text: Cow<'a, str>) -> Self {
        let kind = SegmentKind::classify(text.as_ref());
        Self { text, kind }
    }

    pub(crate) fn as_str(&self) -> &str {
        self.text.as_ref()
    }
}

impl SegmentKind {
    pub(crate) fn classify(segment: &str) -> Self {
        if segment.is_empty() {
            SegmentKind::Empty
        } else if segment.chars().all(|c| c.is_ascii_digit()) {
            SegmentKind::Numeric
        } else {
            SegmentKind::Other
        }
    }

    pub(crate) fn container_type(self) -> ContainerType {
        match self {
            SegmentKind::Empty | SegmentKind::Numeric => ContainerType::Array,
            SegmentKind::Other => ContainerType::Object,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ContainerType {
    Array,
    Object,
}

#[cfg(test)]
#[path = "segment_test.rs"]
mod segment_test;
