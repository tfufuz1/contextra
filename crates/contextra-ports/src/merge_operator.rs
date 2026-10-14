//! Merge operator and merge operator factory port traits (§4.3).

#![forbid(unsafe_code)]

use contextra_types::Result;

/// Trait for folding existing values and operand sequences during compaction or mutation (CRDT / associative merge).
pub trait MergeOperator: Send + Sync {
    /// Merges an optional existing value with a sequence of operand values for a given key.
    fn full_merge(
        &self,
        key: &[u8],
        existing_value: Option<&[u8]>,
        operands: &[&[u8]],
    ) -> Result<Vec<u8>>;
}

/// Factory port for instantiating WASM or native merge operators (§4.3).
///
/// Decouples `contextra-store` from `contextra-sandbox`.
pub trait MergeOperatorFactory: Send + Sync {
    /// Instantiates a merge operator identified by `operator_name`.
    fn create_operator(&self, operator_name: &str) -> Result<Box<dyn MergeOperator>>;
}
