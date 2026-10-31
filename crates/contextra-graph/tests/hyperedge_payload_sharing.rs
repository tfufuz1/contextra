//! Modul `hyperedge_payload_sharing` im Ring 0 (Graph-Subsystem) von Contextra.
//!
//! Stellt Kernkomponenten für Graphtraversierung, CSR-Repräsentationen und
//! Analysemethoden bereit. Wird von Integrationsschichten und Algorithmenmodulen verwendet.

use contextra_graph::csr::EdgeType;
use contextra_graph::hyperedge::{HyperEdge, HyperEdgeId, RoleBinding, RoleId};
use contextra_types::EntityId;
use std::sync::Arc;

// Prüft das korrekte Verhalten und die Invarianten in diesem Testszenario.
#[test]
fn test_hyperedge_payload_sharing_arc_ptr_eq() {
    let bindings: Arc<[RoleBinding]> = Arc::from(vec![
        RoleBinding::new(RoleId::new(1), EntityId::new(101)),
        RoleBinding::new(RoleId::new(2), EntityId::new(102)),
        RoleBinding::new(RoleId::new(3), EntityId::new(103)),
    ]);

    let edge1 = HyperEdge::new(
        HyperEdgeId::new(1001),
        EdgeType::Default,
        Arc::clone(&bindings),
        1.0,
    );

    let edge2 = HyperEdge::new(
        HyperEdgeId::new(1002),
        EdgeType::Default,
        Arc::clone(&bindings),
        2.5,
    );

    // SPECS: Verify that both hyperedges reference the exact same memory allocation for role bindings
    assert!(
        Arc::ptr_eq(&edge1.participants, &edge2.participants),
        "HyperEdge participants must share identical memory heap allocation via Arc::ptr_eq"
    );

    assert_eq!(edge1.participants.len(), 3);
    assert_eq!(edge2.participants.len(), 3);
    assert_eq!(edge1.participants[0], bindings[0]);
}
