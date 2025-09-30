use super::{ContainerType, ResolvedSegment, SegmentKey, SegmentKind};
use std::borrow::{Borrow, Cow};

#[test]
fn classify_returns_empty_for_empty_segment() {
    assert_eq!(SegmentKind::classify(""), SegmentKind::Empty);
}

#[test]
fn classify_detects_numeric_segments() {
    assert_eq!(SegmentKind::classify("123"), SegmentKind::Numeric);
    assert_eq!(SegmentKind::classify("0001"), SegmentKind::Numeric);
}

#[test]
fn classify_treats_non_ascii_digits_as_other() {
    assert_eq!(SegmentKind::classify("١٢٣"), SegmentKind::Other);
    assert_eq!(SegmentKind::classify("42a"), SegmentKind::Other);
}

#[test]
fn container_type_matches_segment_kind() {
    assert_eq!(SegmentKind::Empty.container_type(), ContainerType::Array);
    assert_eq!(SegmentKind::Numeric.container_type(), ContainerType::Array);
    assert_eq!(SegmentKind::Other.container_type(), ContainerType::Object);
}

#[test]
fn segment_key_clones_bytes_and_exposes_str() {
    let source = String::from("status");
    let key = SegmentKey::new(&source);
    drop(source);

    assert_eq!(key.as_str(), "status");
    assert_eq!(<SegmentKey as Borrow<[u8]>>::borrow(&key), b"status");
}

#[test]
fn resolved_segment_infers_kind_and_preserves_text() {
    let borrowed = ResolvedSegment::new(Cow::Borrowed("items"));
    assert_eq!(borrowed.as_str(), "items");
    assert_eq!(borrowed.kind, SegmentKind::Other);

    let owned = ResolvedSegment::new(Cow::Owned(String::from("123")));
    assert_eq!(owned.as_str(), "123");
    assert_eq!(owned.kind, SegmentKind::Numeric);
}
