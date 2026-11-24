#![cfg(feature = "workspace")]
use contextra_graph::types::local as L;
use contextra_ports::graph as P;
use contextra_types as W;
use proptest::prelude::*;

#[test]
fn test_entity_id_parity() {
    let raw = 42u64;
    let l = L::EntityId::new(raw);
    let w = W::EntityId::new(raw);
    assert_eq!(l.inner(), w.inner());
    assert_eq!(l.to_string(), w.to_string());
    assert_eq!(serde_json::to_string(&l).unwrap(), serde_json::to_string(&w).unwrap());
    assert_eq!(bincode::serialize(&l).unwrap(), bincode::serialize(&w).unwrap());
    let l_from_key = L::EntityId::from_key("test_node").unwrap();
    let w_from_key = W::EntityId::from_key("test_node").unwrap();
    assert_eq!(l_from_key.inner(), w_from_key.inner());
}

#[test]
fn test_doc_id_parity() {
    let raw = 12345u64;
    let l = L::DocId::new(raw);
    let w = W::DocId::new(raw);
    assert_eq!(l.inner(), w.inner());
    assert_eq!(l.to_string(), w.to_string());
    assert_eq!(serde_json::to_string(&l).unwrap(), serde_json::to_string(&w).unwrap());
    assert_eq!(bincode::serialize(&l).unwrap(), bincode::serialize(&w).unwrap());
    let l_from_key = L::DocId::from_key("doc_item").unwrap();
    let w_from_key = W::DocId::from_key("doc_item").unwrap();
    assert_eq!(l_from_key.inner(), w_from_key.inner());
}

#[test]
fn test_tx_id_parity() {
    let raw = 987u64;
    let l = L::TxId::new(raw);
    let w = W::TxId::new(raw);
    assert_eq!(l.inner(), w.inner());
    assert_eq!(l.to_string(), w.to_string());
    assert_eq!(serde_json::to_string(&l).unwrap(), serde_json::to_string(&w).unwrap());
    assert_eq!(bincode::serialize(&l).unwrap(), bincode::serialize(&w).unwrap());
}

#[test]
fn test_hyperedge_id_and_role_parity() {
    let he_raw = 555u64;
    let l_he = L::HyperEdgeId::new(he_raw);
    let w_he = P::HyperEdgeId::new(he_raw);
    assert_eq!(l_he.inner(), w_he.inner());
    assert_eq!(l_he.to_string(), w_he.to_string());
    assert_eq!(serde_json::to_string(&l_he).unwrap(), serde_json::to_string(&w_he).unwrap());
    assert_eq!(bincode::serialize(&l_he).unwrap(), bincode::serialize(&w_he).unwrap());

    let r_raw = 77u32;
    let l_role = L::RoleId::new(r_raw);
    let w_role = P::RoleId::new(r_raw);
    assert_eq!(l_role.inner(), w_role.inner());
    assert_eq!(l_role.to_string(), w_role.to_string());
    assert_eq!(serde_json::to_string(&l_role).unwrap(), serde_json::to_string(&w_role).unwrap());
    assert_eq!(bincode::serialize(&l_role).unwrap(), bincode::serialize(&w_role).unwrap());

    let l_rb = L::RoleBinding::new(l_role, L::EntityId::new(101));
    let w_rb = P::RoleBinding::new(w_role, W::EntityId::new(101));
    assert_eq!(serde_json::to_string(&l_rb).unwrap(), serde_json::to_string(&w_rb).unwrap());
    assert_eq!(bincode::serialize(&l_rb).unwrap(), bincode::serialize(&w_rb).unwrap());
}

#[test]
fn test_entity_and_edge_parity() {
    let l_ent = L::Entity::new(L::EntityId::new(1), "Alice", "Person");
    let w_ent = W::Entity::new(W::EntityId::new(1), "Alice", "Person");
    assert_eq!(serde_json::to_string(&l_ent).unwrap(), serde_json::to_string(&w_ent).unwrap());
    assert_eq!(bincode::serialize(&l_ent).unwrap(), bincode::serialize(&w_ent).unwrap());

    let l_edge = L::Edge::new(L::EntityId::new(1), L::EntityId::new(2), "knows")
        .with_weight(0.75)
        .with_source_doc_id(Some(L::DocId::new(10)));
    let mut w_edge = W::Edge::new(W::EntityId::new(1), W::EntityId::new(2), "knows");
    w_edge.weight = 0.75;
    w_edge.source_doc_id = Some(W::DocId::new(10));
    assert_eq!(serde_json::to_string(&l_edge).unwrap(), serde_json::to_string(&w_edge).unwrap());
    assert_eq!(bincode::serialize(&l_edge).unwrap(), bincode::serialize(&w_edge).unwrap());
}

#[test]
fn test_ppr_config_and_params_parity() {
    let l_tl = L::TlHfdParams::default();
    let w_tl = W::TlHfdParams::default();
    assert_eq!(serde_json::to_string(&l_tl).unwrap(), serde_json::to_string(&w_tl).unwrap());
    assert_eq!(bincode::serialize(&l_tl).unwrap(), bincode::serialize(&w_tl).unwrap());

    let l_cfg = L::PprConfig::default();
    let w_cfg = W::PprConfig::default();
    assert_eq!(serde_json::to_string(&l_cfg).unwrap(), serde_json::to_string(&w_cfg).unwrap());
    assert_eq!(bincode::serialize(&l_cfg).unwrap(), bincode::serialize(&w_cfg).unwrap());
    assert_eq!(L::MAX_SEARCH_K, W::MAX_SEARCH_K);
}

proptest! {
    #[test]
    fn prop_entity_id_bincode_parity(id in any::<u64>()) {
        let l = L::EntityId::new(id);
        let w = W::EntityId::new(id);
        prop_assert_eq!(bincode::serialize(&l).unwrap(), bincode::serialize(&w).unwrap());
        prop_assert_eq!(serde_json::to_string(&l).unwrap(), serde_json::to_string(&w).unwrap());
    }

    #[test]
    fn prop_doc_id_bincode_parity(id in any::<u64>()) {
        let l = L::DocId::new(id);
        let w = W::DocId::new(id);
        prop_assert_eq!(bincode::serialize(&l).unwrap(), bincode::serialize(&w).unwrap());
        prop_assert_eq!(serde_json::to_string(&l).unwrap(), serde_json::to_string(&w).unwrap());
    }

    #[test]
    fn prop_from_key_parity(key in "[a-zA-Z0-9_-]{1,64}") {
        let l = L::EntityId::from_key(&key).unwrap();
        let w = W::EntityId::from_key(&key).unwrap();
        prop_assert_eq!(l.inner(), w.inner());
        let l_doc = L::DocId::from_key(&key).unwrap();
        let w_doc = W::DocId::from_key(&key).unwrap();
        prop_assert_eq!(l_doc.inner(), w_doc.inner());
    }
}
