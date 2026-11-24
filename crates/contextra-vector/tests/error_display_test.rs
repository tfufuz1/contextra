use contextra_vector::error::VectorError;

#[test]
fn test_vector_error_display() {
    let cases = vec![
        (
            VectorError::InvalidInput("bad value".to_string()),
            "Invalid input: bad value",
        ),
        (
            VectorError::Index("corrupted node".to_string()),
            "Index error: corrupted node",
        ),
        (
            VectorError::DimensionMismatch {
                expected: 128,
                actual: 64,
            },
            "Embedding dimension mismatch: expected 128, got 64",
        ),
        (
            VectorError::ConnectivityDegraded {
                deleted_ratio: 0.854321,
            },
            "HNSW graph connectivity degraded: 0.9% deleted nodes",
        ),
        (
            VectorError::GraphRepairFailed("unreachable island".to_string()),
            "graph repair failed: unreachable island",
        ),
        (
            VectorError::Concurrency("lock acquisition timed out".to_string()),
            "Concurrency error: lock acquisition timed out",
        ),
        (
            VectorError::Io("file not found".to_string()),
            "I/O error: file not found",
        ),
        (
            VectorError::Unsupported("feature X not available".to_string()),
            "Capability unsupported: vector - feature X not available",
        ),
    ];

    for (err, expected_display) in cases {
        assert_eq!(err.to_string(), expected_display);
    }
}

#[cfg(feature = "ports-compat")]
#[test]
fn test_vector_error_conversion_roundtrip() {
    use contextra_types::ContextraError;

    let v_err = VectorError::InvalidInput("test input".into());
    let c_err: ContextraError = v_err.clone().into();
    assert!(matches!(c_err, ContextraError::InvalidInput(_)));
    let back: VectorError = c_err.into();
    assert_eq!(back, v_err);

    let v_dim = VectorError::DimensionMismatch {
        expected: 128,
        actual: 64,
    };
    let c_dim: ContextraError = v_dim.clone().into();
    assert!(matches!(
        c_dim,
        ContextraError::EmbeddingDimensionMismatch { .. }
    ));
    let back_dim: VectorError = c_dim.into();
    assert_eq!(back_dim, v_dim);

    let v_conn = VectorError::ConnectivityDegraded { deleted_ratio: 0.5 };
    let c_conn: ContextraError = v_conn.clone().into();
    let back_conn: VectorError = c_conn.into();
    assert_eq!(back_conn, v_conn);

    let v_io = VectorError::Io("broken pipe".into());
    let c_io: ContextraError = v_io.clone().into();
    let back_io: VectorError = c_io.into();
    assert_eq!(back_io, v_io);
}
