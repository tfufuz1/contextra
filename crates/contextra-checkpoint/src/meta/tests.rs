use super::*;

#[test]
fn checkpoint_meta_CASE_serialization_roundtrip() {
    let meta = CheckpointMeta {
        name: "test_checkpoint".to_string(),
        collection_id: "test_coll".to_string(),
        seq_no: 100,
        tx_id: TxId::new(42),
        metadata: serde_json::json!({"key": "value"}),
        created_at: 1700000000123,
    };

    let serialized = serde_json::to_vec(&meta).expect("Serialization must succeed");
    let deserialized: CheckpointMeta =
        serde_json::from_slice(&serialized).expect("Deserialization must succeed");

    assert_eq!(deserialized, meta);
    assert_eq!(deserialized.name, "test_checkpoint");
    assert_eq!(deserialized.collection_id, "test_coll");
    assert_eq!(deserialized.seq_no, 100);
    assert_eq!(deserialized.tx_id, TxId::new(42));
    assert_eq!(deserialized.metadata, serde_json::json!({"key": "value"}));
    assert_eq!(deserialized.created_at, 1700000000123);
}

#[test]
fn state_checkpoint_CASE_serialization_roundtrip() {
    let state = StateCheckpoint {
        tx_id: TxId::new(123),
        timestamp_ms: 1700000000456,
        namespace: Some("test_namespace".to_string()),
    };

    let serialized = serde_json::to_vec(&state).expect("Serialization must succeed");
    let deserialized: StateCheckpoint =
        serde_json::from_slice(&serialized).expect("Deserialization must succeed");

    assert_eq!(deserialized, state);
    assert_eq!(deserialized.tx_id, TxId::new(123));
    assert_eq!(deserialized.timestamp_ms, 1700000000456);
    assert_eq!(deserialized.namespace, Some("test_namespace".to_string()));
}

#[test]
fn state_checkpoint_deserializes_legacy_json_without_namespace() {
    let legacy_json = r#"{"tx_id": 999, "timestamp_ms": 1700000000000}"#;
    let deserialized: StateCheckpoint =
        serde_json::from_str(legacy_json).expect("Legacy JSON without namespace must deserialize");

    assert_eq!(deserialized.tx_id, TxId::new(999));
    assert_eq!(deserialized.timestamp_ms, 1700000000000);
    assert_eq!(deserialized.namespace, None);
}

#[test]
fn into_workflow_state_CASE_valid_conversion() {
    let state = StateCheckpoint {
        tx_id: TxId::new(456),
        timestamp_ms: 1000,
        namespace: Some("ns".to_string()),
    };

    let ws = state.into_workflow_state();
    assert_eq!(ws.tx, TxId::new(456));
    assert_ne!(ws.graph_hash, [0u8; 32]);
}

#[test]
fn test_validate_identifier_empty_or_whitespace() {
    assert!(validate_identifier("field", "").is_err());
    assert!(validate_identifier("field", "   ").is_err());
    assert!(validate_identifier("field", "\t\n").is_err());
}

#[test]
fn test_validate_identifier_boundary_oversized_257() {
    let valid_256 = "a".repeat(256);
    assert!(validate_identifier("field", &valid_256).is_ok());

    let invalid_257 = "a".repeat(257);
    assert!(validate_identifier("field", &invalid_257).is_err());
}
