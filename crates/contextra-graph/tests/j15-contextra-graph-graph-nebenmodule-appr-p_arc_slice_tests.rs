//! Integration tests for `ArcSlice::as_arc`.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_graph::csr::EdgeType;
use contextra_graph::{ArcSlice, HyperEdge, HyperEdgeId, RoleBinding, RoleId};
use contextra_types::EntityId;
use std::sync::Arc;

#[test]
fn test_arc_slice_as_arc_integration() {
    let participants = vec![
        RoleBinding::new(RoleId::new(1), EntityId::new(100)),
        RoleBinding::new(RoleId::new(2), EntityId::new(200)),
    ];

    let arc_slice = ArcSlice::from_vec(participants);
    let backing_arc: &Arc<[RoleBinding]> = arc_slice.as_arc();

    assert_eq!(backing_arc.len(), 2);
    assert_eq!(backing_arc[0].entity, EntityId::new(100));
    assert_eq!(backing_arc[1].entity, EntityId::new(200));

    let edge = HyperEdge::new(
        HyperEdgeId::new(1),
        EdgeType::Default,
        arc_slice.as_arc().clone(),
        1.0,
    );
    let view = edge.view();

    assert_eq!(view.participants.as_arc().len(), 2);
}
