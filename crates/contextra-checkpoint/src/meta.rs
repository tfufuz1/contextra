use contextra_types::{ContextraError, Result, TxId, WorkflowState};
use serde::{Deserialize, Serialize};

// RESOLVED: AGT-CKPT-001 — UTF-8 char counting used for 256 char limit (TS: 2026-09-01T23:07:05Z) (SESSION: 358e3b0a)
/// Validates identifier strings (checkpoint name, collection ID) against empty/whitespace or size limits.
pub(crate) fn validate_identifier(field_name: &str, value: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(ContextraError::InvalidInput(format!(
            "{field_name} cannot be empty or whitespace-only"
        )));
    }
    let char_count = value.chars().count();
    if char_count > 256 {
        return Err(ContextraError::InvalidInput(format!(
            "{field_name} exceeds maximum length of 256 characters (got {char_count})"
        )));
    }
    Ok(())
}

/// Metadata for a persistent checkpoint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CheckpointMeta {
    pub name: String,
    pub collection_id: String,
    pub seq_no: u64,
    pub tx_id: TxId,
    pub metadata: serde_json::Value,
    pub created_at: u64,
}

impl CheckpointMeta {
    pub fn into_workflow_state(&self) -> WorkflowState {
        WorkflowState {
            tx: self.tx_id,
            graph_hash: *blake3::hash(format!("seq-{}", self.seq_no).as_bytes()).as_bytes(),
        }
    }
}

/// Point-in-Time Checkpoint representing an agent step or transaction boundary.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateCheckpoint {
    pub tx_id: TxId,
    pub timestamp_ms: u64,
    #[serde(default)]
    pub namespace: Option<String>,
}

#[cfg(test)]
#[allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]
mod tests {
    use super::*;

    #[allow(non_snake_case)]
    #[test]
    fn checkpoint_meta_CASE_serialization_roundtrip() {
        let meta = CheckpointMeta {
            name: "cp_test".to_string(),
            collection_id: "col_test".to_string(),
            seq_no: 99,
            tx_id: TxId::new(1001),
            metadata: serde_json::json!({"step": 42, "env": "prod"}),
            created_at: 1690000000,
        };

        let json = serde_json::to_string(&meta).expect("// expect #[cfg(test)]");
        let deserialized: CheckpointMeta =
            serde_json::from_str(&json).expect("// expect #[cfg(test)]");

        // Independent evaluation without referencing meta in comparison construction
        assert_eq!(deserialized.name, "cp_test");
        assert_eq!(deserialized.collection_id, "col_test");
        assert_eq!(deserialized.seq_no, 99);
        assert_eq!(deserialized.tx_id, TxId::new(1001));
        assert_eq!(deserialized.metadata["step"], 42);
        assert_eq!(deserialized.created_at, 1690000000);
    }

    #[allow(non_snake_case)]
    #[test]
    fn state_checkpoint_CASE_serialization_roundtrip() {
        let cp = StateCheckpoint {
            tx_id: TxId::new(888),
            timestamp_ms: 1700000000123,
            namespace: Some("test_ns".to_string()),
        };

        let json = serde_json::to_string(&cp).expect("// expect #[cfg(test)]");
        let deserialized: StateCheckpoint =
            serde_json::from_str(&json).expect("// expect #[cfg(test)]");

        assert_eq!(deserialized.tx_id, TxId::new(888));
        assert_eq!(deserialized.timestamp_ms, 1700000000123);
        assert_eq!(deserialized.namespace, Some("test_ns".to_string()));
    }

    #[test]
    fn state_checkpoint_deserializes_legacy_json_without_namespace() {
        let legacy_json = r#"{"tx_id": 999, "timestamp_ms": 1700000000000}"#;
        let deserialized: StateCheckpoint = serde_json::from_str(legacy_json)
            .expect("Legacy JSON without namespace must deserialize");

        assert_eq!(deserialized.tx_id, TxId::new(999));
        assert_eq!(deserialized.timestamp_ms, 1700000000000);
        assert_eq!(deserialized.namespace, None);
    }

    #[allow(non_snake_case)]
    #[test]
    fn into_workflow_state_CASE_valid_conversion() {
        let meta = CheckpointMeta {
            name: "wf_cp".to_string(),
            collection_id: "wf_col".to_string(),
            seq_no: 15,
            tx_id: TxId::new(2026),
            metadata: serde_json::json!({"agent_phase": "reasoning"}),
            created_at: 5000,
        };

        let state = meta.into_workflow_state();

        // Independent expected value assertions
        assert_eq!(state.tx, TxId::new(2026));
        assert_ne!(state.graph_hash, [0u8; 32]);
    }

    #[test]
    fn test_validate_identifier_boundary_oversized_257() {
        let name_256 = "a".repeat(256);
        assert!(validate_identifier("field", &name_256).is_ok());

        let name_257 = "a".repeat(257);
        let res = validate_identifier("field", &name_257);
        assert!(matches!(res, Err(ContextraError::InvalidInput(_))));
    }

    #[test]
    fn test_validate_identifier_empty_or_whitespace() {
        let res_empty = validate_identifier("field", "");
        assert!(matches!(res_empty, Err(ContextraError::InvalidInput(_))));

        let res_ws = validate_identifier("field", "   \t\n  ");
        assert!(matches!(res_ws, Err(ContextraError::InvalidInput(_))));
    }
}
