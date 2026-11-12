//! Tabellengetriebener Fehlertest für `contextra-vector` (S-36).

#![cfg(feature = "ports-compat")]

use contextra_types::ContextraError;
use contextra_vector::error::VectorError;

#[test]
fn test_s36_error_mapping_bidirectional() {
    let test_cases: Vec<(VectorError, String)> = vec![
        (
            VectorError::InvalidInput("bad parameter".into()),
            "Invalid input: bad parameter".into(),
        ),
        (
            VectorError::Index("corrupt graph".into()),
            "Index error: corrupt graph".into(),
        ),
        (
            VectorError::Storage("io failure on disk".into()),
            "Storage error: io failure on disk".into(),
        ),
        (
            VectorError::DimensionMismatch {
                expected: 128,
                actual: 64,
            },
            "Embedding dimension mismatch: expected 128, got 64".into(),
        ),
        (
            VectorError::ConnectivityDegraded {
                deleted_ratio: 0.25,
            },
            "HNSW graph connectivity degraded: 0.2% deleted nodes".into(),
        ),
        (
            VectorError::NotFound("node 42".into()),
            "Not found: node 42".into(),
        ),
        (
            VectorError::Conflict("write conflict".into()),
            "Conflict: write conflict".into(),
        ),
        (
            VectorError::Internal("state machine invalid".into()),
            "Internal error: state machine invalid".into(),
        ),
        (
            VectorError::Unsupported("diskann in memory only".into()),
            "Capability unsupported: vector - diskann in memory only".into(),
        ),
    ];

    for (local_err, expected_display) in test_cases {
        let contextra_err: ContextraError = local_err.clone().into();
        assert_eq!(
            format!("{}", contextra_err),
            expected_display,
            "Display mismatch for {:?}",
            local_err
        );

        // Also test local display format
        assert_eq!(
            format!("{}", local_err),
            expected_display,
            "Local display mismatch for {:?}",
            local_err
        );
    }
}
