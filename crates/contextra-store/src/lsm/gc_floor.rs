//! Re-export of `GcFloor` from `contextra-mvcc`.

/// Type alias for the MVCC GC floor implementation in `contextra-store`.
pub type GcFloor = contextra_mvcc::GcFloor<(Vec<u8>, Vec<u8>)>;
