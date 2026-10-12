// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: Operations representation for DST workloads.
// INVARIANTEN: Fully serializable, deterministic, equality checkable.

/// Operations executed during Deterministic Simulation Testing (DST).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DstOp {
    /// Inserts or overwrites a key-value pair.
    Put { key: Vec<u8>, value: Vec<u8> },
    /// Deletes a key-value pair.
    Delete { key: Vec<u8> },
    /// Permanently shreds/forgets a key and its history.
    Forget { key: Vec<u8> },
    /// Commits all staged operations.
    Commit,
    /// Rolls back all staged operations.
    Rollback,
    /// Flushes/fsyncs state to persistent storage.
    Sync,
}
