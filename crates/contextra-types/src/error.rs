//! Error types for `Contextra`.

// FILE-CONTEXT
// STAND: 2026-08-30T21:51:46Z (SESSION: a43b7682)
// ZWECK: Kanonische unified ContextraError Enum für den gesamten Workspace.
// INVARIANTEN: Zero-Panic via ? propagation; #[non_exhaustive] für binäre Abwärtskompatibilität. KEINE neue Error-Enum in anderen Crates anlegen.
// HOTSPOTS: 20-180
// NICHT-OFFENSICHTLICH: Neue Fehler-Varianten NUR unten anhängen; Downstream-Crates brauchen Wildcard-Match arm.
// SIEHE AUCH: rules/tag_taxonomy.md, DECISIONS.md

// INVARIANT: Einzige Error-Enum für den gesamten Workspace.
// Neue Varianten nur ANHÄNGEN (niemals umsortieren) → binäre Kompatibilität.
// DOWNSTREAM: contextra-store, contextra-index, contextra-db konvertieren via `?` und `From`.

use thiserror::Error;

/// Convenience alias for `Result<T, ContextraError>`.
pub type Result<T> = std::result::Result<T, ContextraError>;

/// Unified error type for all `Contextra` operations across the entire workspace.
///
/// # Non-Exhaustive Variant Guarantee
/// This enum is marked `#[non_exhaustive]` to allow appending new error variants
/// in future minor releases without breaking downstream `match` statements across crate boundaries
/// (such as `contextra-py` and `contextra-mcp`).
///
/// Downstream callers matching on `ContextraError` must include a wildcard arm (`_ => ...`).
/// New variants are appended strictly to the bottom of the enum to preserve binary and FFI compatibility.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum ContextraError {
    // ═══ Core & Logic ═══
    /// Internal engine logic error.
    #[error("Internal error: {0}")]
    Internal(String),

    /// Invalid user or API argument input.
    #[error("Invalid input: {0}")]
    InvalidInput(String),

    /// Resource, key, or document not found.
    #[error("Not found: {0}")]
    NotFound(String),

    /// Security or execution policy violation.
    #[error("Policy violation: {0}")]
    PolicyViolation(String),

    // ═══ Storage Engine ═══
    /// Storage layer failure.
    #[error("Storage error: {0}")]
    Storage(String),

    /// Standard I/O error wrapper.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Data integrity corruption detected in Write-Ahead Log.
    #[error("WAL corruption detected at offset {offset}: {reason}")]
    #[non_exhaustive]
    WalCorruption {
        /// Byte offset in WAL file where corruption occurred.
        offset: u64,
        /// Detail text describing corruption cause.
        reason: String,
    },

    /// Block checksum validation failure.
    #[error("Checksum mismatch: file={path}, block={block_id}")]
    #[non_exhaustive]
    ChecksumMismatch {
        /// File path of corrupted storage block.
        path: String,
        /// Block identifier.
        block_id: u64,
    },

    // ═══ Transactions & Consistency ═══
    /// Transaction lifecycle or execution failure.
    #[error("Transaction error: {0}")]
    Transaction(String),

    /// Transaction execution timeout exceeded.
    #[error("Transaction {tx_id} timed out after {elapsed_ms}ms")]
    TransactionTimeout {
        /// Identifier of timed out transaction.
        tx_id: u64,
        /// Elapsed time in milliseconds before timeout.
        elapsed_ms: u64,
    },

    /// Data conflict during commit or mutation.
    #[error("Conflict: {0}")]
    Conflict(String),

    /// Out-of-order or invalid sequence number.
    #[error("Invalid sequence number: {0}")]
    InvalidSequenceNumber(u64),

    // ═══ Index & Search ═══
    /// General index operation failure.
    #[error("Index error: {0}")]
    Index(String),

    /// Vector or embedding dimension mismatch.
    #[error("Embedding dimension mismatch: expected {expected}, got {got}")]
    EmbeddingDimensionMismatch {
        /// Expected vector dimension.
        expected: usize,
        /// Actual vector dimension.
        got: usize,
    },

    /// HNSW graph degradation warning threshold reached.
    #[error("HNSW graph connectivity degraded: {deleted_ratio:.1}% deleted nodes")]
    HnswConnectivityDegraded {
        /// Ratio of deleted tombstone nodes in graph.
        deleted_ratio: f64,
    },

    /// Text search engine operation failure.
    #[error("Text engine error: {0}")]
    Text(String),

    // ═══ Resources & Sandbox ═══
    /// Configured memory allocation budget exceeded.
    #[error("Memory budget exceeded: {used_mb}MB / {limit_mb}MB")]
    MemoryBudgetExceeded {
        /// Currently used memory in MB.
        used_mb: u64,
        /// Configured memory limit in MB.
        limit_mb: u64,
    },

    /// Code or query sandbox execution error.
    #[error("Sandbox error: {0}")]
    Sandbox(String),

    /// Sandbox memory limit breach.
    #[error("Memory limit exceeded in sandbox: {0}")]
    MemoryLimitExceeded(String),

    /// Sandbox execution timeout breach.
    #[error("Timeout exceeded in sandbox: {0}")]
    SandboxTimeout(String),

    /// General operation execution timeout breach.
    #[error("Operation timed out: {operation} (limit: {timeout_ms}ms)")]
    Timeout {
        /// Identifier of the timed out operation.
        operation: String,
        /// Configured timeout limit in milliseconds.
        timeout_ms: u64,
    },

    // ═══ Infrastructure ═══
    /// Data serialization or deserialization error.
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// JSON processing error wrapper.
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Cryptographic operation error.
    #[error("Crypto error: {0}")]
    Crypto(String),

    /// Requested checkpoint missing or deleted.
    #[error("Checkpoint not found")]
    CheckpointNotFound,

    /// Distributed cluster operation error.
    #[error("Cluster error: {0}")]
    Cluster(String),

    /// Data parsing error.
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Bincode serialization or deserialization error wrapper.
    #[error("Bincode error: {0}")]
    Bincode(#[from] bincode::Error),

    /// Capability requested is not supported by this engine or implementation.
    #[error("Capability unsupported: {capability} - {reason}")]
    CapabilityUnsupported {
        /// Unique capability identifier.
        capability: String,
        /// Detail text describing why capability is unsupported.
        reason: String,
    },

    /// Snapshot isolation is unsupported for the requested search signal or graph strategy.
    #[error("Snapshot unsupported for signal/strategy: {0}")]
    SnapshotUnsupportedForSignal(String),

    /// Optimistic concurrency control stale read or version conflict.
    #[error("Stale read / OCC conflict: {0}")]
    StaleRead(String),

    /// Scan or query result limit exceeded.
    #[error("Limit exceeded: {context} (limit: {limit})")]
    LimitExceeded {
        /// Configured or default result limit.
        limit: usize,
        /// Context description of the scan or operation.
        context: String,
    },

    /// GGUF-/Modell-Ladefehler (Candle-Backend). Nutzung: korrupte, unvollständige
    /// oder dimensions-inkompatible Modell-Dateien beim Laden über contextra-candle.
    #[error("Model load error for {path}: {reason}")]
    ModelLoad {
        /// Path to model file.
        path: String,
        /// Detailed reason for failure.
        reason: String,
    },

    /// Strukturelle Inkonsistenz zwischen Vektor-Index und Dokumentenspeicher:
    /// eine Vektor-ID im Index verweist auf ein DocId, das im Store nicht (mehr)
    /// existiert (Split-Brain-Zustand). Unterscheidet sich bewusst von `NotFound`,
    /// das für normale, erwartbare Lookup-Misses (z.B. TTL-Expiry) reserviert bleibt.
    #[error("Orphaned vector reference: index_id={index_id} -> doc_id={doc_id}")]
    OrphanedVectorReference {
        /// Identifier of document referenced by index.
        doc_id: String,
        /// Identifier of vector in index.
        index_id: String,
    },

    /// Group commit execution timeout exceeded for a follower transaction.
    #[error("Group commit timed out for transaction {tx_id}")]
    CommitTimeout {
        /// Identifier of timed out transaction.
        tx_id: u64,
    },
}

impl ContextraError {
    /// Creates a `LimitExceeded` error.
    pub fn limit_exceeded(limit: usize, context: impl Into<String>) -> Self {
        Self::LimitExceeded {
            limit,
            context: context.into(),
        }
    }

    /// Creates a `ModelLoad` error.
    pub fn model_load(path: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::ModelLoad {
            path: path.into(),
            reason: reason.into(),
        }
    }

    /// Creates a `SnapshotUnsupportedForSignal` error.
    pub fn snapshot_unsupported_for_signal(strategy: impl Into<String>) -> Self {
        Self::SnapshotUnsupportedForSignal(strategy.into())
    }

    /// Creates an `OrphanedVectorReference` error.
    pub fn orphaned_vector_reference(
        doc_id: impl Into<String>,
        index_id: impl Into<String>,
    ) -> Self {
        Self::OrphanedVectorReference {
            doc_id: doc_id.into(),
            index_id: index_id.into(),
        }
    }

    /// Creates a `CapabilityUnsupported` error.
    pub fn capability_unsupported(
        capability: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self::CapabilityUnsupported {
            capability: capability.into(),
            reason: reason.into(),
        }
    }
    /// Creates an `InvalidInput` error from any displayable value.
    pub fn invalid_input(msg: impl Into<String>) -> Self {
        Self::InvalidInput(msg.into())
    }

    /// Creates a `WalCorruption` error.
    pub fn wal_corruption(offset: u64, reason: impl Into<String>) -> Self {
        Self::WalCorruption {
            offset,
            reason: reason.into(),
        }
    }

    /// Creates a `ChecksumMismatch` error.
    pub fn checksum_mismatch(path: impl Into<String>, block_id: u64) -> Self {
        Self::ChecksumMismatch {
            path: path.into(),
            block_id,
        }
    }

    /// Returns `true` if this error represents an optimistic concurrency control (OCC) conflict or stale read.
    pub fn is_occ_conflict(&self) -> bool {
        matches!(self, Self::StaleRead(_) | Self::Conflict(_))
    }
}

impl From<std::array::TryFromSliceError> for ContextraError {
    fn from(e: std::array::TryFromSliceError) -> Self {
        Self::ParseError(e.to_string())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn contextra_error_display_no_panic() {
        let variants = [
            ContextraError::Internal("test".into()),
            ContextraError::InvalidInput("test".into()),
            ContextraError::NotFound("test".into()),
            ContextraError::Storage("test".into()),
            ContextraError::PolicyViolation("test".into()),
            ContextraError::WalCorruption {
                offset: 10,
                reason: "test".into(),
            },
            ContextraError::ChecksumMismatch {
                path: "file".into(),
                block_id: 1,
            },
            ContextraError::Transaction("test".into()),
            ContextraError::TransactionTimeout {
                tx_id: 1,
                elapsed_ms: 100,
            },
            ContextraError::Conflict("test".into()),
            ContextraError::InvalidSequenceNumber(1),
            ContextraError::Index("test".into()),
            ContextraError::EmbeddingDimensionMismatch {
                expected: 1536,
                got: 768,
            },
            ContextraError::HnswConnectivityDegraded { deleted_ratio: 0.1 },
            ContextraError::Text("test".into()),
            ContextraError::MemoryBudgetExceeded {
                used_mb: 10,
                limit_mb: 5,
            },
            ContextraError::Sandbox("test".into()),
            ContextraError::MemoryLimitExceeded("test".into()),
            ContextraError::SandboxTimeout("test".into()),
            ContextraError::Serialization("test".into()),
            ContextraError::Crypto("test".into()),
            ContextraError::CheckpointNotFound,
            ContextraError::Cluster("test".into()),
            ContextraError::ParseError("test".into()),
            ContextraError::ModelLoad {
                path: "model.gguf".into(),
                reason: "corrupt header".into(),
            },
            ContextraError::OrphanedVectorReference {
                doc_id: "doc_123".into(),
                index_id: "idx_456".into(),
            },
        ];
        for v in &variants {
            let _ = format!("{v}");
            let _ = format!("{v:?}");
        }
    }

    #[test]
    fn test_invalid_input_helper() {
        let err = ContextraError::invalid_input("bad param");
        match err {
            ContextraError::InvalidInput(msg) => assert_eq!(msg, "bad param"),
            _ => panic!("Expected InvalidInput, got {:?}", err),
        }
    }

    #[test]
    fn test_capability_unsupported_helper() {
        let err = ContextraError::capability_unsupported("snapshot_read_at", "ADR-024");
        assert_eq!(
            err.to_string(),
            "Capability unsupported: snapshot_read_at - ADR-024"
        );
        match err {
            ContextraError::CapabilityUnsupported { capability, reason } => {
                assert_eq!(capability, "snapshot_read_at");
                assert_eq!(reason, "ADR-024");
            }
            _ => panic!("Expected CapabilityUnsupported, got {:?}", err),
        }
    }

    #[test]
    fn test_from_try_from_slice_error() {
        let slice: &[u8] = &[1, 2, 3];
        let try_from_res: std::result::Result<[u8; 4], _> = slice.try_into();
        assert!(try_from_res.is_err());

        let parse_err: ContextraError = try_from_res.unwrap_err().into();
        match parse_err {
            ContextraError::ParseError(msg) => {
                assert!(msg.contains("could not convert slice to array") || msg.contains("slice"));
            }
            _ => panic!("Expected ParseError, got {:?}", parse_err),
        }
    }

    #[test]
    fn test_error_display_all_variants() {
        // Core & Logic
        assert_eq!(
            ContextraError::Internal("test".into()).to_string(),
            "Internal error: test"
        );
        assert_eq!(
            ContextraError::InvalidInput("test".into()).to_string(),
            "Invalid input: test"
        );
        assert_eq!(
            ContextraError::NotFound("doc_1".into()).to_string(),
            "Not found: doc_1"
        );
        assert_eq!(
            ContextraError::PolicyViolation("test".into()).to_string(),
            "Policy violation: test"
        );

        // Storage Engine
        assert_eq!(
            ContextraError::Storage("test".into()).to_string(),
            "Storage error: test"
        );
        assert_eq!(
            ContextraError::WalCorruption {
                offset: 1024,
                reason: "invalid header".into()
            }
            .to_string(),
            "WAL corruption detected at offset 1024: invalid header"
        );
        assert_eq!(
            ContextraError::ChecksumMismatch {
                path: "f".into(),
                block_id: 1
            }
            .to_string(),
            "Checksum mismatch: file=f, block=1"
        );

        // Transactions & Consistency
        assert_eq!(
            ContextraError::Transaction("test".into()).to_string(),
            "Transaction error: test"
        );
        assert_eq!(
            ContextraError::TransactionTimeout {
                tx_id: 1,
                elapsed_ms: 50
            }
            .to_string(),
            "Transaction 1 timed out after 50ms"
        );
        assert_eq!(
            ContextraError::Conflict("test".into()).to_string(),
            "Conflict: test"
        );
        assert_eq!(
            ContextraError::InvalidSequenceNumber(42).to_string(),
            "Invalid sequence number: 42"
        );

        // Index & Search
        assert_eq!(
            ContextraError::Index("test".into()).to_string(),
            "Index error: test"
        );
        assert_eq!(
            ContextraError::EmbeddingDimensionMismatch {
                expected: 1536,
                got: 768
            }
            .to_string(),
            "Embedding dimension mismatch: expected 1536, got 768"
        );
        assert_eq!(
            ContextraError::HnswConnectivityDegraded {
                deleted_ratio: 25.0
            }
            .to_string(),
            "HNSW graph connectivity degraded: 25.0% deleted nodes"
        );
        assert_eq!(
            ContextraError::Text("test".into()).to_string(),
            "Text engine error: test"
        );

        // Resources & Sandbox
        assert_eq!(
            ContextraError::MemoryBudgetExceeded {
                used_mb: 100,
                limit_mb: 50
            }
            .to_string(),
            "Memory budget exceeded: 100MB / 50MB"
        );
        assert_eq!(
            ContextraError::Sandbox("test".into()).to_string(),
            "Sandbox error: test"
        );
        assert_eq!(
            ContextraError::MemoryLimitExceeded("test".into()).to_string(),
            "Memory limit exceeded in sandbox: test"
        );
        assert_eq!(
            ContextraError::SandboxTimeout("test".into()).to_string(),
            "Timeout exceeded in sandbox: test"
        );

        // Infrastructure
        assert_eq!(
            ContextraError::Serialization("test".into()).to_string(),
            "Serialization error: test"
        );
        assert_eq!(
            ContextraError::Crypto("test".into()).to_string(),
            "Crypto error: test"
        );
        assert_eq!(
            ContextraError::CheckpointNotFound.to_string(),
            "Checkpoint not found"
        );
        assert_eq!(
            ContextraError::Cluster("test".into()).to_string(),
            "Cluster error: test"
        );
        assert_eq!(
            ContextraError::ParseError("test".into()).to_string(),
            "Parse error: test"
        );
        let bincode_err: bincode::Error = Box::new(bincode::ErrorKind::Custom("test".into()));
        assert_eq!(
            ContextraError::Bincode(bincode_err).to_string(),
            "Bincode error: test"
        );
    }

    #[test]
    fn test_from_conversions() {
        use std::error::Error;

        // Io
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let m_err: ContextraError = io_err.into();
        assert!(matches!(m_err, ContextraError::Io(_)));
        assert!(m_err.source().is_some());

        // Json
        let json_str = "{ invalid }";
        let json_err: serde_json::Error =
            serde_json::from_str::<serde_json::Value>(json_str).unwrap_err();
        let m_err2: ContextraError = json_err.into();
        assert!(matches!(m_err2, ContextraError::Json(_)));
        assert!(m_err2.source().is_some());

        // Bincode
        let bincode_err: bincode::Error =
            Box::new(bincode::ErrorKind::Custom("test bincode".into()));
        let m_err_bc: ContextraError = bincode_err.into();
        assert!(matches!(m_err_bc, ContextraError::Bincode(_)));
        assert!(m_err_bc.source().is_some());

        // Other variants should return None for source()
        let m_err3 = ContextraError::Internal("test".into());
        assert!(m_err3.source().is_none());
    }

    #[test]
    fn test_result_alias() {
        fn fail() -> Result<()> {
            Err(ContextraError::Internal("failed".to_string()))
        }
        let res = fail();
        assert!(res.is_err());
    }

    /// Mutation-robustness test: verifies that `From<std::io::Error>` preserves the
    /// original I/O error message in the Display output.
    ///
    /// # Anti-Mirroring
    /// Expected string `"I/O error: "` is a hand-written prefix, independent of the format
    /// impl string `"I/O error: {0}"`. If the format string changed (e.g. prefix dropped),
    /// this test would catch it.
    ///
    /// # Mutation robustness
    /// Removing the `#[from]` attribute would break this test (Io variant would stop matching).
    /// Changing the prefix from "I/O error" to anything else would break the `starts_with` assert.
    #[test]
    fn test_io_error_message_preserved() {
        let sentinel = "unique_sentinel_message_for_mutation_test_42";
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, sentinel);
        let mf_err: ContextraError = io_err.into();

        // Must be the Io variant — not Internal or Storage
        assert!(
            matches!(mf_err, ContextraError::Io(_)),
            "Expected ContextraError::Io, got: {:?}",
            mf_err
        );

        // The original message must survive in the Display output (no message loss)
        let display = mf_err.to_string();
        assert!(
            display.contains(sentinel),
            "io::Error message must be preserved. Display was: {:?}",
            display
        );
        // The ContextraError prefix must also be present (structural check)
        assert!(
            display.starts_with("I/O error:"),
            "Expected 'I/O error:' prefix, got: {:?}",
            display
        );
    }

    /// Mutation-robustness test: verifies `From<serde_json::Error>` carries the parse error info.
    ///
    /// # Anti-Mirroring
    /// `"JSON error:"` is a hand-written known-stable string; not derived from the format macro.
    #[test]
    fn test_json_error_message_preserved() {
        let sentinel_json = "{\"key\": }"; // Valid JSON prefix, invalid tail
        let json_err = serde_json::from_str::<serde_json::Value>(sentinel_json).unwrap_err();
        let original_msg = json_err.to_string();
        let mf_err: ContextraError = json_err.into();

        assert!(matches!(mf_err, ContextraError::Json(_)));

        let display = mf_err.to_string();
        // Message from the JSON parser must survive in the output
        assert!(
            display.contains(&original_msg) || display.contains("expected value"),
            "Json error detail must be preserved. Display: {:?}, original: {:?}",
            display,
            original_msg
        );
        assert!(
            display.starts_with("JSON error:"),
            "Expected 'JSON error:' prefix, got: {:?}",
            display
        );
    }

    /// Mutation-robustness test: WalCorruption fields must not be transposed.
    ///
    /// # Invariant
    /// Swapping `offset` and `reason` in the struct definition would break this test.
    /// Using the same value for both fields (lazy test) would not — this test uses distinct
    /// types to make field-swapping impossible, and distinct values to catch display-level bugs.
    #[test]
    fn test_wal_corruption_fields_not_transposed() {
        let err = ContextraError::WalCorruption {
            offset: 98765,
            reason: "corrupted hmac chain".to_string(),
        };
        let display = err.to_string();

        // The numeric offset must appear in the display — not be silently replaced by the reason
        assert!(
            display.contains("98765"),
            "WalCorruption offset must appear in display: {:?}",
            display
        );
        // The reason text must also appear
        assert!(
            display.contains("corrupted hmac chain"),
            "WalCorruption reason must appear in display: {:?}",
            display
        );
        // The offset must NOT accidentally appear where the reason should be
        // (catches field-transposition bug in format string)
        assert!(
            !display.starts_with("WAL corruption detected at offset corrupted"),
            "Offset and reason must not be transposed: {:?}",
            display
        );
    }

    /// Mutation-robustness test: ChecksumMismatch fields must not be transposed.
    #[test]
    fn test_checksum_mismatch_fields_not_transposed() {
        let err = ContextraError::ChecksumMismatch {
            path: "data_segment_003.sst".to_string(),
            block_id: 887766,
        };
        let display = err.to_string();

        // Check path and block_id are present
        assert!(
            display.contains("data_segment_003.sst"),
            "ChecksumMismatch path must appear in display: {:?}",
            display
        );
        assert!(
            display.contains("887766"),
            "ChecksumMismatch block_id must appear in display: {:?}",
            display
        );

        // Verify correct field positions (path shouldn't be formatted into block position)
        assert!(
            display.contains("file=data_segment_003.sst"),
            "Path formatted incorrectly: {:?}",
            display
        );
        assert!(
            display.contains("block=887766"),
            "Block ID formatted incorrectly: {:?}",
            display
        );
    }

    /// Mutation-robustness test: TransactionTimeout fields must not be transposed.
    #[test]
    fn test_transaction_timeout_fields_not_transposed() {
        let err = ContextraError::TransactionTimeout {
            tx_id: 112233,
            elapsed_ms: 998877,
        };
        let display = err.to_string();

        assert!(
            display.contains("112233"),
            "TransactionTimeout tx_id must appear in display: {:?}",
            display
        );
        assert!(
            display.contains("998877"),
            "TransactionTimeout elapsed_ms must appear in display: {:?}",
            display
        );

        // Position checks
        assert!(
            display.starts_with("Transaction 112233 timed out"),
            "Transaction ID formatted in wrong place: {:?}",
            display
        );
        assert!(
            display.contains("after 998877ms"),
            "Elapsed ms formatted in wrong place: {:?}",
            display
        );
    }

    /// Mutation-robustness test: MemoryBudgetExceeded fields must not be transposed.
    #[test]
    fn test_memory_budget_exceeded_fields_not_transposed() {
        let err = ContextraError::MemoryBudgetExceeded {
            used_mb: 4096,
            limit_mb: 8192,
        };
        let display = err.to_string();

        assert!(
            display.contains("4096"),
            "MemoryBudgetExceeded used_mb must appear in display: {:?}",
            display
        );
        assert!(
            display.contains("8192"),
            "MemoryBudgetExceeded limit_mb must appear in display: {:?}",
            display
        );

        // Position checks
        assert!(
            display.contains("4096MB / 8192MB"),
            "Used and limit MB fields are transposed or misformatted: {:?}",
            display
        );
    }

    /// Mutation-robustness test: asserts HnswConnectivityDegraded deleted_ratio is preserved as-is.
    ///
    /// Checks that the deleted_ratio is preserved without scaling (not divided/multiplied by 100)
    /// and that the formatting accurately reflects the ratio.
    #[test]
    fn test_hnsw_connectivity_degraded_preserves_ratio() {
        let sentinel_ratio = 37.42;
        let err = ContextraError::HnswConnectivityDegraded {
            deleted_ratio: sentinel_ratio,
        };

        if let ContextraError::HnswConnectivityDegraded { deleted_ratio } = err {
            assert!(
                (deleted_ratio - sentinel_ratio).abs() < f64::EPSILON,
                "deleted_ratio must be preserved as-is: expected {}, got {}",
                sentinel_ratio,
                deleted_ratio
            );
        } else {
            panic!("Expected ContextraError::HnswConnectivityDegraded");
        }

        let display = err.to_string();
        assert!(
            display.contains("37.4%"),
            "Display formatting must contain '37.4%': got {:?}",
            display
        );
    }

    #[test]
    fn test_error_constructor_helpers() {
        let err_cap = ContextraError::capability_unsupported("vector_search", "no_gpu");
        assert!(matches!(
            err_cap,
            ContextraError::CapabilityUnsupported {
                ref capability,
                ref reason
            } if capability == "vector_search" && reason == "no_gpu"
        ));

        let err_inv = ContextraError::invalid_input("key cannot be empty");
        assert!(
            matches!(err_inv, ContextraError::InvalidInput(ref msg) if msg == "key cannot be empty")
        );

        let err_wal = ContextraError::wal_corruption(1024, "bad crc");
        assert!(matches!(
            err_wal,
            ContextraError::WalCorruption { offset: 1024, ref reason } if reason == "bad crc"
        ));

        let err_chk = ContextraError::checksum_mismatch("/var/data.sst", 7);
        assert!(matches!(
            err_chk,
            ContextraError::ChecksumMismatch { ref path, block_id: 7 } if path == "/var/data.sst"
        ));
    }

    #[test]
    fn test_dto_with_details_override() {
        use crate::error_dto::ContextraErrorDto;
        let dto = ContextraErrorDto::with_details(
            "CustomKind",
            "Custom message",
            serde_json::json!({"trace_id": "12345"}),
        );
        assert_eq!(dto.kind, "CustomKind");
        assert_eq!(dto.message, "Custom message");
        assert_eq!(dto.details.expect("details present")["trace_id"], "12345"); // expect
    }

    #[test]
    fn test_model_load_and_orphaned_vector_display() {
        let err_model = ContextraError::model_load("/models/llama3.gguf", "invalid tensor layout");
        assert_eq!(
            err_model.to_string(),
            "Model load error for /models/llama3.gguf: invalid tensor layout"
        );
        match err_model {
            ContextraError::ModelLoad { path, reason } => {
                assert_eq!(path, "/models/llama3.gguf");
                assert_eq!(reason, "invalid tensor layout");
            }
            _ => panic!("Expected ModelLoad, got {:?}", err_model),
        }

        let err_orphan = ContextraError::orphaned_vector_reference("doc_99", "vec_1001");
        assert_eq!(
            err_orphan.to_string(),
            "Orphaned vector reference: index_id=vec_1001 -> doc_id=doc_99"
        );
        match err_orphan {
            ContextraError::OrphanedVectorReference { doc_id, index_id } => {
                assert_eq!(doc_id, "doc_99");
                assert_eq!(index_id, "vec_1001");
            }
            _ => panic!("Expected OrphanedVectorReference, got {:?}", err_orphan),
        }
    }

    #[test]
    fn test_is_occ_conflict() {
        let stale = ContextraError::StaleRead("OCC conflict".into());
        let conflict = ContextraError::Conflict("Key conflict".into());
        let not_found = ContextraError::NotFound("Key not found".into());

        assert!(stale.is_occ_conflict());
        assert!(conflict.is_occ_conflict());
        assert!(!not_found.is_occ_conflict());
    }
}
