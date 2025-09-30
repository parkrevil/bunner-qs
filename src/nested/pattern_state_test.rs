use super::{ContainerType, ResolvedSegment, acquire_pattern_state};
use std::borrow::Cow;

fn make_segments<'a>(parts: &'a [&'a str]) -> Vec<ResolvedSegment<'a>> {
    parts
        .iter()
        .map(|segment| ResolvedSegment::new(Cow::Borrowed(*segment)))
        .collect()
}

#[test]
fn resolve_reuses_existing_numeric_sequence_indices() {
    let mut guard = acquire_pattern_state();
    let path = make_segments(&["items"]);

    let first = guard
        .resolve(&path, "", "items")
        .expect("first blank segment should succeed");
    assert_eq!(first, "0");

    let second = guard
        .resolve(&path, "", "items")
        .expect("subsequent blank segment should succeed");
    assert_eq!(second, "1");
}

#[test]
fn resolve_errors_on_conflicting_segment_kinds() {
    let mut guard = acquire_pattern_state();
    let path = make_segments(&["items"]);

    guard
        .resolve(&path, "0", "items")
        .expect("numeric branch should be accepted");

    let error = guard
        .resolve(&path, "field", "items")
        .expect_err("conflicting kind should raise duplicate key error");

    match error {
        crate::ParseError::DuplicateKey { key } => assert_eq!(key, "items"),
        other => panic!("expected duplicate key error, got {other:?}"),
    }
}

#[test]
fn container_type_reflects_recorded_segment_kind() {
    let mut guard = acquire_pattern_state();
    let path = make_segments(&["items"]);

    guard
        .resolve(&path, "", "items")
        .expect("generated index should succeed");

    assert_eq!(guard.container_type(&["items"]), Some(ContainerType::Array));

    drop(guard);
    let mut guard = acquire_pattern_state();
    let path = make_segments(&["props"]);

    guard
        .resolve(&path, "field", "props")
        .expect("string segment should succeed");

    assert_eq!(
        guard.container_type(&["props"]),
        Some(ContainerType::Object)
    );
}

#[test]
fn child_capacity_reports_number_of_children() {
    let mut guard = acquire_pattern_state();
    let path = make_segments(&["items"]);

    guard.resolve(&path, "", "items").unwrap();
    guard.resolve(&path, "", "items").unwrap();
    guard.resolve(&path, "", "items").unwrap();

    assert_eq!(guard.child_capacity(&["items"]), 3);
}

#[test]
fn acquire_pattern_state_reuses_pool_instances() {
    {
        let mut guard = acquire_pattern_state();
        guard
            .resolve(&[], "foo", "root")
            .expect("first use should succeed");
    }

    let guard = acquire_pattern_state();
    assert!(guard.container_type(&["foo"]).is_none());
    assert_eq!(guard.child_capacity(&[]), 0);
}
