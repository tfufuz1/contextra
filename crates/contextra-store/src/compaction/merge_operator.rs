//! Merge Operator trait abstraction for two-way SSTable value merging (§4.12).

use contextra_core::Result;

/// Trait for merging two raw byte values (`existing_val` from older tier, `new_val` from newer tier)
/// for the same key across different SSTable tiers.
pub trait MergeOperator: Send + Sync {
    /// Merges two raw byte values for the same key.
    ///
    /// # Fail-Safe Requirement
    /// Implementations running untrusted guest code (e.g. WASM via `contextra-sandbox`) MUST
    /// implement fail-safe handling: if execution fails (e.g. fuel exhaustion or trap),
    /// the caller or operator must swallow the error for that single entry, log a `tracing::warn!`,
    /// and retain both original versions in the compacted SSTable without aborting the overall compaction.
    fn merge(&self, existing_val: &[u8], new_val: &[u8]) -> Result<Vec<u8>>;
}
