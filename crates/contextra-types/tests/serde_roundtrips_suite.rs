//! Exhaustive Serde roundtrip and invalid payload rejection test suite for `contextra-types`.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use contextra_types::error::StepId;
use contextra_types::*;
use std::sync::Arc;

#[test]
fn test_exhaustive_serde_json_strings_and_bytes() {
    let tenant = TenantId::try_new(999).unwrap();
    let json_t = serde_json::to_string(&tenant).unwrap();
    assert_eq!(json_t, "999");
    assert_eq!(serde_json::from_str::<TenantId>("999").unwrap(), tenant);
    assert_eq!(
        serde_json::from_str::<TenantId>("0").unwrap(),
        TenantId::SYSTEM
    );

    let collection = CollectionId::try_new(500).unwrap();
    let json_c = serde_json::to_string(&collection).unwrap();
    assert_eq!(json_c, "500");
    assert_eq!(
        serde_json::from_str::<CollectionId>("500").unwrap(),
        collection
    );

    let doc = DocId::new(123456789);
    let json_d = serde_json::to_string(&doc).unwrap();
    assert_eq!(json_d, "123456789");
    assert_eq!(serde_json::from_str::<DocId>("123456789").unwrap(), doc);

    let entity_id = EntityId::new(987654321);
    let json_e = serde_json::to_string(&entity_id).unwrap();
    assert_eq!(json_e, "987654321");
    assert_eq!(
        serde_json::from_str::<EntityId>("987654321").unwrap(),
        entity_id
    );

    let tx = TxId::new(100000);
    let json_tx = serde_json::to_string(&tx).unwrap();
    assert_eq!(json_tx, "100000");
    assert_eq!(serde_json::from_str::<TxId>("100000").unwrap(), tx);

    let step = StepId::new(12);
    let json_step = serde_json::to_string(&step).unwrap();
    assert_eq!(json_step, "12");
    assert_eq!(serde_json::from_str::<StepId>("12").unwrap(), step);
}

#[test]
fn test_serde_invalid_json_payloads_rejected() {
    // Malformed JSON strings
    assert!(serde_json::from_str::<TenantId>("invalid_num").is_err());
    assert!(serde_json::from_str::<CollectionId>("\"abc\"").is_err());
    assert!(serde_json::from_str::<DocId>("null").is_err());
    assert!(serde_json::from_str::<TxId>("[]").is_err());
    assert!(serde_json::from_str::<StepId>("{}").is_err());

    // Invalid enum variants
    assert!(serde_json::from_str::<DistanceMetric>("\"UnknownMetric\"").is_err());
    assert!(serde_json::from_str::<MemoryType>("\"UnknownType\"").is_err());
    assert!(serde_json::from_str::<LinkRelation>("\"NonExistent\"").is_err());
}

#[test]
fn test_bincode_binary_payload_sizes() {
    let tenant = TenantId::try_new(42).unwrap();
    let bytes_t = bincode::serialize(&tenant).unwrap();
    assert_eq!(bytes_t.len(), 8);

    let coll = CollectionId::try_new(10).unwrap();
    let bytes_c = bincode::serialize(&coll).unwrap();
    assert_eq!(bytes_c.len(), 8);

    let tx = TxId::new(100);
    let bytes_tx = bincode::serialize(&tx).unwrap();
    assert_eq!(bytes_tx.len(), 8);

    #[cfg(not(feature = "docid-128"))]
    {
        let doc = DocId::new(1);
        let bytes_d = bincode::serialize(&doc).unwrap();
        assert_eq!(bytes_d.len(), 8);
    }
    #[cfg(feature = "docid-128")]
    {
        let doc = DocId::new(1);
        let bytes_d = bincode::serialize(&doc).unwrap();
        assert_eq!(bytes_d.len(), 16);
    }
}

#[test]
fn test_serde_complex_nested_structures() {
    let filter = FilterExpr::And(
        Box::new(FilterExpr::Eq {
            field: "status".to_string(),
            value: serde_json::json!("active"),
        }),
        Box::new(FilterExpr::Gte {
            field: "age".to_string(),
            value: serde_json::json!(18),
        }),
    );

    let json = serde_json::to_string_pretty(&filter).unwrap();
    let deser: FilterExpr = serde_json::from_str(&json).unwrap();
    assert_eq!(filter, deser);
}

#[test]
fn test_workflow_state_hex_serialization() {
    let hash = [0x12u8; 32];
    let state = WorkflowState {
        tx: TxId::new(42),
        graph_hash: hash,
    };

    let json = serde_json::to_string(&state).unwrap();
    assert!(json.contains("1212121212121212121212121212121212121212121212121212121212121212"));

    let deser: WorkflowState = serde_json::from_str(&json).unwrap();
    assert_eq!(deser.graph_hash, hash);
    assert_eq!(deser.tx, TxId::new(42));
}

#[test]
fn test_entity_and_edge_attribute_maps() {
    let mut attrs = ahash::AHashMap::new();
    attrs.insert("level".to_string(), serde_json::json!(5));
    attrs.insert("active".to_string(), serde_json::json!(true));

    let entity = Entity {
        id: EntityId::new(1),
        name: Arc::from("Agent Alpha"),
        entity_type: Arc::from("SystemAgent"),
        attributes: attrs,
    };

    let json = serde_json::to_string(&entity).unwrap();
    let deser: Entity = serde_json::from_str(&json).unwrap();
    assert_eq!(deser.id, EntityId::new(1));
    assert_eq!(deser.attributes.get("level"), Some(&serde_json::json!(5)));
}
