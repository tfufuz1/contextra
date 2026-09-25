//! Serializable DTO representation of `ContextraError` for IPC and FFI boundaries (ADR-028).

// FILE-CONTEXT
// STAND: 2026-08-30T21:51:46Z (SESSION: a43b7682)
// ZWECK: Serialisierbares Error-DTO für IPC/FFI-Schichten ohne Typverlust (ADR-028).
// INVARIANTEN: Behält error kind, message und strukturierte JSON details verlustfrei über FFI.
// HOTSPOTS: 20-80
// NICHT-OFFENSICHTLICH: DTO erlaubt die Rekonstruktion strukturierter Fehler in FFI/Bindings und Python.
// SIEHE AUCH: rules/tag_taxonomy.md, DECISIONS.md (ADR-028)

use crate::error::ContextraError;
use serde::{Deserialize, Serialize};

/// Serializable data transfer object representing a [`ContextraError`].
///
/// Used across IPC and API boundaries (e.g. Tauri frontend IPC) to preserve structured
/// error kinds and optional JSON detail fields without losing error typing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContextraErrorDto {
    /// Stable string identifier for the error variant (e.g. `"NotFound"`, `"PolicyViolation"`).
    pub kind: String,
    /// Human-readable error message.
    pub message: String,
    /// Structured detail payload for complex error variants (e.g. offset/reason for WAL corruption).
    pub details: Option<serde_json::Value>,
}

impl ContextraErrorDto {
    /// Creates a new `ContextraErrorDto` with custom kind and message.
    pub fn new(kind: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            message: message.into(),
            details: None,
        }
    }

    /// Creates a new `ContextraErrorDto` with custom kind, message, and details payload.
    pub fn with_details(
        kind: impl Into<String>,
        message: impl Into<String>,
        details: serde_json::Value,
    ) -> Self {
        Self {
            kind: kind.into(),
            message: message.into(),
            details: Some(details),
        }
    }
}

impl std::fmt::Display for ContextraErrorDto {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}]: {}", self.kind, self.message)
    }
}

impl std::error::Error for ContextraErrorDto {}

impl From<ContextraError> for ContextraErrorDto {
    fn from(err: ContextraError) -> Self {
        Self::from(&err)
    }
}

impl From<String> for ContextraErrorDto {
    fn from(msg: String) -> Self {
        Self {
            kind: "InvalidInput".to_string(),
            message: msg,
            details: None,
        }
    }
}

impl From<&str> for ContextraErrorDto {
    fn from(msg: &str) -> Self {
        Self {
            kind: "InvalidInput".to_string(),
            message: msg.to_string(),
            details: None,
        }
    }
}

impl From<&ContextraError> for ContextraErrorDto {
    fn from(err: &ContextraError) -> Self {
        // NOTE: Strictly no catch-all `_ => ...` wildcard arm in this match expression.
        // Every single variant of ContextraError must be explicitly listed below.
        // If a new variant is added to ContextraError, Rust compilation will fail here
        // until From<&ContextraError> is deliberately updated.
        match err {
            ContextraError::Internal(msg) => Self {
                kind: "Internal".to_string(),
                message: msg.clone(),
                details: None,
            },
            ContextraError::InvalidInput(msg) => Self {
                kind: "InvalidInput".to_string(),
                message: msg.clone(),
                details: None,
            },
            ContextraError::NotFound(msg) => Self {
                kind: "NotFound".to_string(),
                message: msg.clone(),
                details: None,
            },
            ContextraError::PolicyViolation(msg) => Self {
                kind: "PolicyViolation".to_string(),
                message: msg.clone(),
                details: None,
            },
            ContextraError::Storage(msg) => Self {
                kind: "Storage".to_string(),
                message: msg.clone(),
                details: None,
            },
            ContextraError::Io(io_err) => Self {
                kind: "Io".to_string(),
                message: io_err.to_string(),
                details: None,
            },
            ContextraError::WalCorruption { offset, reason } => Self {
                kind: "WalCorruption".to_string(),
                message: err.to_string(),
                details: Some(serde_json::json!({
                    "offset": offset,
                    "reason": reason,
                })),
            },
            ContextraError::ChecksumMismatch { path, block_id } => Self {
                kind: "ChecksumMismatch".to_string(),
                message: err.to_string(),
                details: Some(serde_json::json!({
                    "path": path,
                    "block_id": block_id,
                })),
            },
            ContextraError::Transaction(msg) => Self {
                kind: "Transaction".to_string(),
                message: msg.clone(),
                details: None,
            },
            ContextraError::TransactionTimeout { tx_id, elapsed_ms } => Self {
                kind: "TransactionTimeout".to_string(),
                message: err.to_string(),
                details: Some(serde_json::json!({
                    "tx_id": tx_id,
                    "elapsed_ms": elapsed_ms,
                })),
            },
            ContextraError::Conflict(msg) => Self {
                kind: "Conflict".to_string(),
                message: msg.clone(),
                details: None,
            },
            ContextraError::InvalidSequenceNumber(seq_no) => Self {
                kind: "InvalidSequenceNumber".to_string(),
                message: err.to_string(),
                details: Some(serde_json::json!({
                    "seq_no": seq_no,
                })),
            },
            ContextraError::Index(msg) => Self {
                kind: "Index".to_string(),
                message: msg.clone(),
                details: None,
            },
            ContextraError::EmbeddingDimensionMismatch { expected, got } => Self {
                kind: "EmbeddingDimensionMismatch".to_string(),
                message: err.to_string(),
                details: Some(serde_json::json!({
                    "expected": expected,
                    "got": got,
                })),
            },
            ContextraError::HnswConnectivityDegraded { deleted_ratio } => Self {
                kind: "HnswConnectivityDegraded".to_string(),
                message: err.to_string(),
                details: Some(serde_json::json!({
                    "deleted_ratio": deleted_ratio,
                })),
            },
            ContextraError::Text(msg) => Self {
                kind: "Text".to_string(),
                message: msg.clone(),
                details: None,
            },
            ContextraError::MemoryBudgetExceeded { used_mb, limit_mb } => Self {
                kind: "MemoryBudgetExceeded".to_string(),
                message: err.to_string(),
                details: Some(serde_json::json!({
                    "used_mb": used_mb,
                    "limit_mb": limit_mb,
                })),
            },
            ContextraError::Sandbox(msg) => Self {
                kind: "Sandbox".to_string(),
                message: msg.clone(),
                details: None,
            },
            ContextraError::MemoryLimitExceeded(msg) => Self {
                kind: "MemoryLimitExceeded".to_string(),
                message: msg.clone(),
                details: None,
            },
            ContextraError::SandboxTimeout(msg) => Self {
                kind: "SandboxTimeout".to_string(),
                message: msg.clone(),
                details: None,
            },
            ContextraError::Timeout {
                operation,
                timeout_ms,
            } => Self {
                kind: "Timeout".to_string(),
                message: err.to_string(),
                details: Some(serde_json::json!({
                    "operation": operation,
                    "timeout_ms": timeout_ms,
                })),
            },
            ContextraError::Serialization(msg) => Self {
                kind: "Serialization".to_string(),
                message: msg.clone(),
                details: None,
            },
            ContextraError::Json(json_err) => Self {
                kind: "Json".to_string(),
                message: json_err.to_string(),
                details: None,
            },
            ContextraError::Crypto(msg) => Self {
                kind: "Crypto".to_string(),
                message: msg.clone(),
                details: None,
            },
            ContextraError::CheckpointNotFound => Self {
                kind: "CheckpointNotFound".to_string(),
                message: "Checkpoint not found".to_string(),
                details: None,
            },
            ContextraError::Cluster(msg) => Self {
                kind: "Cluster".to_string(),
                message: msg.clone(),
                details: None,
            },
            ContextraError::ParseError(msg) => Self {
                kind: "ParseError".to_string(),
                message: msg.clone(),
                details: None,
            },
            ContextraError::Bincode(bincode_err) => Self {
                kind: "Bincode".to_string(),
                message: bincode_err.to_string(),
                details: None,
            },
            ContextraError::CapabilityUnsupported { capability, reason } => Self {
                kind: "CapabilityUnsupported".to_string(),
                message: err.to_string(),
                details: Some(serde_json::json!({
                    "capability": capability,
                    "reason": reason,
                })),
            },
            ContextraError::StaleRead(msg) => Self {
                kind: "StaleRead".to_string(),
                message: msg.clone(),
                details: None,
            },
            ContextraError::LimitExceeded { limit, context } => Self {
                kind: "LimitExceeded".to_string(),
                message: err.to_string(),
                details: Some(serde_json::json!({
                    "limit": limit,
                    "context": context,
                })),
            },
            ContextraError::ModelLoad { path, reason } => Self {
                kind: "ModelLoad".to_string(),
                message: err.to_string(),
                details: Some(serde_json::json!({
                    "path": path,
                    "reason": reason,
                })),
            },
            ContextraError::OrphanedVectorReference { doc_id, index_id } => Self {
                kind: "OrphanedVectorReference".to_string(),
                message: err.to_string(),
                details: Some(serde_json::json!({
                    "doc_id": doc_id,
                    "index_id": index_id,
                })),
            },
            ContextraError::SnapshotUnsupportedForSignal(msg) => Self {
                kind: "SnapshotUnsupportedForSignal".to_string(),
                message: msg.clone(),
                details: None,
            },
            ContextraError::CommitTimeout { tx_id } => Self {
                kind: "CommitTimeout".to_string(),
                message: err.to_string(),
                details: Some(serde_json::json!({
                    "tx_id": tx_id,
                })),
            },
        }
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_dto_exhaustive_match_coverage() {
        let variants = vec![
            (ContextraError::Internal("test".into()), "Internal"),
            (ContextraError::InvalidInput("test".into()), "InvalidInput"),
            (ContextraError::NotFound("test".into()), "NotFound"),
            (
                ContextraError::PolicyViolation("test".into()),
                "PolicyViolation",
            ),
            (ContextraError::Storage("test".into()), "Storage"),
            (
                ContextraError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "file not found",
                )),
                "Io",
            ),
            (
                ContextraError::WalCorruption {
                    offset: 100,
                    reason: "bad header".into(),
                },
                "WalCorruption",
            ),
            (
                ContextraError::ChecksumMismatch {
                    path: "/tmp/a".into(),
                    block_id: 42,
                },
                "ChecksumMismatch",
            ),
            (ContextraError::Transaction("test".into()), "Transaction"),
            (
                ContextraError::TransactionTimeout {
                    tx_id: 1,
                    elapsed_ms: 500,
                },
                "TransactionTimeout",
            ),
            (ContextraError::Conflict("test".into()), "Conflict"),
            (
                ContextraError::InvalidSequenceNumber(10),
                "InvalidSequenceNumber",
            ),
            (ContextraError::Index("test".into()), "Index"),
            (
                ContextraError::EmbeddingDimensionMismatch {
                    expected: 1536,
                    got: 768,
                },
                "EmbeddingDimensionMismatch",
            ),
            (
                ContextraError::HnswConnectivityDegraded { deleted_ratio: 0.2 },
                "HnswConnectivityDegraded",
            ),
            (ContextraError::Text("test".into()), "Text"),
            (
                ContextraError::MemoryBudgetExceeded {
                    used_mb: 200,
                    limit_mb: 100,
                },
                "MemoryBudgetExceeded",
            ),
            (ContextraError::Sandbox("test".into()), "Sandbox"),
            (
                ContextraError::MemoryLimitExceeded("test".into()),
                "MemoryLimitExceeded",
            ),
            (
                ContextraError::SandboxTimeout("test".into()),
                "SandboxTimeout",
            ),
            (
                ContextraError::Timeout {
                    operation: "tool:test".into(),
                    timeout_ms: 50,
                },
                "Timeout",
            ),
            (
                ContextraError::Serialization("test".into()),
                "Serialization",
            ),
            (
                ContextraError::Json(
                    serde_json::from_str::<serde_json::Value>("{bad}").unwrap_err(),
                ),
                "Json",
            ),
            (ContextraError::Crypto("test".into()), "Crypto"),
            (ContextraError::CheckpointNotFound, "CheckpointNotFound"),
            (ContextraError::Cluster("test".into()), "Cluster"),
            (ContextraError::ParseError("test".into()), "ParseError"),
            (
                ContextraError::Bincode(Box::new(bincode::ErrorKind::Custom("err".into()))),
                "Bincode",
            ),
            (
                ContextraError::CapabilityUnsupported {
                    capability: "cap".into(),
                    reason: "reason".into(),
                },
                "CapabilityUnsupported",
            ),
            (ContextraError::StaleRead("test".into()), "StaleRead"),
            (
                ContextraError::ModelLoad {
                    path: "path.gguf".into(),
                    reason: "corrupt".into(),
                },
                "ModelLoad",
            ),
            (
                ContextraError::OrphanedVectorReference {
                    doc_id: "doc1".into(),
                    index_id: "idx1".into(),
                },
                "OrphanedVectorReference",
            ),
        ];

        for (err, expected_kind) in variants {
            let dto = ContextraErrorDto::from(&err);
            assert_eq!(dto.kind, expected_kind);
            assert!(!dto.message.is_empty());
        }
    }

    #[test]
    fn test_dto_details_serialization() {
        let err = ContextraError::WalCorruption {
            offset: 4096,
            reason: "corrupted block header".to_string(),
        };
        let dto = ContextraErrorDto::from(&err);
        assert_eq!(dto.kind, "WalCorruption");
        let details = dto.details.as_ref().expect("details should exist"); // unwrap allowed
        assert_eq!(details["offset"], 4096);
        assert_eq!(details["reason"], "corrupted block header");

        let json_str = serde_json::to_string(&dto).expect("serde serialize"); // unwrap allowed
        let deser_dto: ContextraErrorDto =
            serde_json::from_str(&json_str).expect("serde deserialize"); // unwrap allowed
        assert_eq!(dto, deser_dto);
    }

    #[test]
    fn test_dto_constructors_and_display() {
        let dto1 = ContextraErrorDto::new("CustomKind", "custom message");
        assert_eq!(dto1.kind, "CustomKind");
        assert_eq!(dto1.message, "custom message");
        assert!(dto1.details.is_none());
        assert_eq!(dto1.to_string(), "[CustomKind]: custom message");

        let details = serde_json::json!({"key": "value"});
        let dto2 = ContextraErrorDto::with_details("CustomKindWithDetails", "msg", details.clone());
        assert_eq!(dto2.kind, "CustomKindWithDetails");
        assert_eq!(dto2.details.as_ref(), Some(&details));

        let dto_str: ContextraErrorDto = "str err".into();
        assert_eq!(dto_str.kind, "InvalidInput");
        assert_eq!(dto_str.message, "str err");

        let dto_string: ContextraErrorDto = String::from("string err").into();
        assert_eq!(dto_string.kind, "InvalidInput");
        assert_eq!(dto_string.message, "string err");

        let owned_err = ContextraError::NotFound("item missing".into());
        let dto_owned: ContextraErrorDto = ContextraErrorDto::from(owned_err);
        assert_eq!(dto_owned.kind, "NotFound");
        assert_eq!(dto_owned.message, "item missing");

        // Test std::error::Error trait implementation
        let err_trait: &dyn std::error::Error = &dto1;
        assert_eq!(err_trait.to_string(), "[CustomKind]: custom message");
    }
}
