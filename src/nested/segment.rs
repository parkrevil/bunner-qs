use smallvec::SmallVec;
use std::borrow::{Borrow, Cow};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SegmentKind {
    Empty,
    Numeric,
    Other,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) struct SegmentKey(SmallVec<[u8; 24]>);

impl SegmentKey {
    pub(crate) fn new(segment: &str) -> Self {
        SegmentKey(SmallVec::from_slice(segment.as_bytes()))
    }

    pub(crate) fn as_str(&self) -> &str {
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
