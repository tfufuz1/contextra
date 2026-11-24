//! Local domain types and errors decoupled from the removed `contextra-types` crate.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Error types for `contextra-checkpoint`.
#[derive(Debug)]
pub enum ContextraError {
    /// Invalid input or argument.
    InvalidInput(String),
    /// Serialization or deserialization failure.
    Serialization(String),
    /// Transaction error.
    Transaction(String),
    /// Requested checkpoint was not found.
    CheckpointNotFound(String),
    /// Storage backend error.
    Storage(String),
    /// Internal error.
    Internal(String),
    /// I/O error.
    Io(std::io::Error),
    /// Cross-device link error during hardlink cloning.
    CrossDeviceLink(String),
}

impl ContextraError {
    /// Creates an invalid input error.
    pub fn invalid_input(msg: impl Into<String>) -> Self {
        Self::InvalidInput(msg.into())
    }

    /// Creates a cross-device link error.
    pub fn cross_device_link(msg: impl Into<String>) -> Self {
        Self::CrossDeviceLink(msg.into())
    }
}

impl fmt::Display for ContextraError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidInput(m) => write!(f, "invalid input: {m}"),
            Self::Serialization(m) => write!(f, "serialization error: {m}"),
            Self::Transaction(m) => write!(f, "transaction error: {m}"),
            Self::CheckpointNotFound(m) => write!(f, "checkpoint not found: {m}"),
            Self::Storage(m) => write!(f, "storage error: {m}"),
            Self::Internal(m) => write!(f, "internal error: {m}"),
            Self::Io(e) => write!(f, "io error: {e}"),
            Self::CrossDeviceLink(m) => write!(f, "cross-device link: {m}"),
        }
    }
}

impl std::error::Error for ContextraError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

/// Result alias using [`ContextraError`].
pub type Result<T> = std::result::Result<T, ContextraError>;

/// Transaction identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TxId(pub u64);

impl TxId {
    /// Base ID for internal transactions.
    pub const INTERNAL_BASE: u64 = 1 << 62;

    /// Creates a new `TxId`.
    #[inline]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Returns the inner `u64`.
    #[inline]
    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Returns the inner `u64`.
    #[inline]
    pub const fn inner(self) -> u64 {
        self.0
    }
}

impl From<u64> for TxId {
    #[inline]
    fn from(v: u64) -> Self {
        Self(v)
    }
}

impl From<TxId> for u64 {
    #[inline]
    fn from(v: TxId) -> Self {
        v.0
    }
}

impl fmt::Display for TxId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Snapshot of workflow state at a transaction boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkflowState {
    /// Transaction identifier.
    pub tx: TxId,
    /// Deterministic BLAKE3 graph state hash.
    pub graph_hash: [u8; 32],
}
