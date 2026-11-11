// FILE-CONTEXT
// ZWECK: Varianten- und Display-Paritätstests für VectorError und contract::Error (WP-VEC-02).
// INVARIANTEN: Zero-Panic, exakte Display-Musterprüfung, Send + Sync + 'static + Error.
// HOTSPOTS: test_vector_error_display, test_contract_error_display
// STAND: 2026-11-11

use contextra_vector::contract::Error as ContractError;
use contextra_vector::error::VectorError;
use std::error::Error;

#[test]
fn test_vector_error_display_parity() {
    assert_eq!(
        VectorError::InvalidInput("bad param".to_string()).to_string(),
        "Invalid input: bad param"
    );

    assert_eq!(
        VectorError::Index("corrupt graph".to_string()).to_string(),
        "Index error: corrupt graph"
    );

    assert_eq!(
        VectorError::Storage("disk full".to_string()).to_string(),
        "Storage error: disk full"
    );

    assert_eq!(
        VectorError::DimensionMismatch {
            expected: 128,
            actual: 64
        }
        .to_string(),
        "Embedding dimension mismatch: expected 128, got 64"
    );

    assert_eq!(
        VectorError::ConnectivityDegraded {
            deleted_ratio: 42.5
        }
        .to_string(),
        "HNSW graph connectivity degraded: 42.5% deleted nodes"
    );

    assert_eq!(
        VectorError::GraphRepairFailed("unreachable island".to_string()).to_string(),
        "graph repair failed: unreachable island"
    );

    assert_eq!(
        VectorError::NotFound("doc_42".to_string()).to_string(),
        "Not found: doc_42"
    );

    assert_eq!(
        VectorError::Conflict("write conflict".to_string()).to_string(),
        "Conflict: write conflict"
    );

    assert_eq!(
        VectorError::Concurrency("lock timeout".to_string()).to_string(),
        "Concurrency error: lock timeout"
    );

    assert_eq!(
        VectorError::Io("permission denied".to_string()).to_string(),
        "I/O error: permission denied"
    );

    assert_eq!(
        VectorError::Unsupported("dynamic dimensions".to_string()).to_string(),
        "Capability unsupported: vector - dynamic dimensions"
    );

    assert_eq!(
        VectorError::Internal("unexpected sentinel".to_string()).to_string(),
        "Internal error: unexpected sentinel"
    );
}

#[test]
fn test_vector_error_traits() {
    let err = VectorError::InvalidInput("test".to_string());
    let cloned = err.clone();
    assert_eq!(err, cloned);

    fn assert_error_traits<E: Error + Send + Sync + 'static>() {}
    assert_error_traits::<VectorError>();

    let helper = VectorError::invalid_input("msg");
    assert_eq!(helper, VectorError::InvalidInput("msg".to_string()));

    let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file missing");
    let from_io = VectorError::from(io_err);
    assert_eq!(from_io, VectorError::Io("file missing".to_string()));
}

#[test]
fn test_contract_error_display_and_traits() {
    assert_eq!(
        ContractError::InvalidInput("falsche Dimension").to_string(),
        "ungültige Eingabedaten: falsche Dimension"
    );

    assert_eq!(
        ContractError::CapacityExceeded {
            requested: 200,
            maximum: 100
        }
        .to_string(),
        "Kapazitätsgrenze überschritten: gefordert 200, maximal 100"
    );

    let io_err = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "pipe broken");
    assert_eq!(
        ContractError::from(io_err).to_string(),
        "E/A-Fehler: pipe broken"
    );

    assert_eq!(
        ContractError::StateConflict("insert failed").to_string(),
        "interner Zustandskonflikt: insert failed"
    );

    assert_eq!(
        ContractError::Corrupt("header invalid").to_string(),
        "beschädigte Daten: header invalid"
    );

    fn assert_contract_error_traits<E: Error + Send + Sync + 'static>() {}
    assert_contract_error_traits::<ContractError>();
}
