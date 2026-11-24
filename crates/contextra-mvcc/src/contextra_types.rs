//! Local domain types and errors decoupled from the removed `contextra-types` crate.

/// Error types for `contextra-mvcc`.
pub mod error {
    use thiserror::Error;

    /// Core error type.
    #[derive(Debug, Error, Clone, PartialEq, Eq)]
    pub enum ContextraError {
        /// Transaction conflict (e.g. SSI write-skew).
        #[error("conflict: {0}")]
        Conflict(String),
        /// Invalid input or argument.
        #[error("invalid input: {0}")]
        InvalidInput(String),
        /// Limit exceeded.
        #[error("limit exceeded: {0}")]
        LimitExceeded(String),
        /// Not found.
        #[error("not found: {0}")]
        NotFound(String),
        /// Storage error.
        #[error("storage error: {0}")]
        Storage(String),
        /// Transaction error.
        #[error("transaction error: {0}")]
        Transaction(String),
        /// Internal error.
        #[error("internal error: {0}")]
        Internal(String),
    }

    impl ContextraError {
        /// Creates a conflict error.
        pub fn conflict(msg: impl Into<String>) -> Self {
            Self::Conflict(msg.into())
        }

        /// Creates an invalid input error.
        pub fn invalid_input(msg: impl Into<String>) -> Self {
            Self::InvalidInput(msg.into())
        }

        /// Creates a limit exceeded error.
        pub fn limit_exceeded(_limit: usize, msg: impl Into<String>) -> Self {
            Self::LimitExceeded(msg.into())
        }
    }

    /// Result alias using [`ContextraError`].
    pub type Result<T> = std::result::Result<T, ContextraError>;
}

/// Tombstone bit-flag helpers.
pub mod tombstone {
    /// Marker trait for sequence-bit tombstones.
    pub trait SeqBitTombstone {
        /// Returns true if the sequence number has the tombstone bit set.
        fn is_tombstone(&self) -> bool;
        /// Returns the raw sequence number without the tombstone bit.
        fn raw_seq(&self) -> u64;
    }

    /// Tombstone semantics verification marker.
    pub trait TombstoneSemanticsCheck {}

    impl SeqBitTombstone for u64 {
        #[inline]
        fn is_tombstone(&self) -> bool {
            (*self & super::types::TOMBSTONE_BIT) != 0
        }

        #[inline]
        fn raw_seq(&self) -> u64 {
            *self & !super::types::TOMBSTONE_BIT
        }
    }
}

/// Core identifier types.
pub mod types {
    use serde::{Deserialize, Serialize};
    use std::fmt;

    /// High bit used to mark tombstones in sequence numbers.
    pub const TOMBSTONE_BIT: u64 = 1 << 63;

    /// Document identifier.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    pub struct DocId(pub u64);

    impl DocId {
        /// Creates a new `DocId`.
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

        /// Parses or hashes a `DocId` from a string key.
        pub fn from_key(s: &str) -> super::error::Result<Self> {
            if s.is_empty() {
                return Err(super::error::ContextraError::invalid_input("empty DocId key"));
            }
            if let Ok(id) = s.parse::<u64>() {
                return Ok(Self(id));
            }
            // Deterministic FNV-1a 64-bit hash for arbitrary string keys.
            let mut hash: u64 = 0xcbf29ce484222325;
            for &b in s.as_bytes() {
                hash ^= u64::from(b);
                hash = hash.wrapping_mul(0x100000001b3);
            }
            Ok(Self(hash))
        }
    }

    impl From<u64> for DocId {
        #[inline]
        fn from(v: u64) -> Self {
            Self(v)
        }
    }

    impl From<DocId> for u64 {
        #[inline]
        fn from(v: DocId) -> Self {
            v.0
        }
    }

    impl fmt::Display for DocId {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    /// Tenant identifier.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
    pub struct TenantId(pub u64);

    impl TenantId {
        /// Creates a new `TenantId`.
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

        /// Parses or hashes a `DocId` from a string key.
        pub fn from_key(s: &str) -> super::error::Result<Self> {
            if s.is_empty() {
                return Err(super::error::ContextraError::invalid_input("empty DocId key"));
            }
            if let Ok(id) = s.parse::<u64>() {
                return Ok(Self(id));
            }
            // Deterministic FNV-1a 64-bit hash for arbitrary string keys.
            let mut hash: u64 = 0xcbf29ce484222325;
            for &b in s.as_bytes() {
                hash ^= u64::from(b);
                hash = hash.wrapping_mul(0x100000001b3);
            }
            Ok(Self(hash))
        }
    }

    impl From<u64> for TenantId {
        #[inline]
        fn from(v: u64) -> Self {
            Self(v)
        }
    }

    impl From<TenantId> for u64 {
        #[inline]
        fn from(v: TenantId) -> Self {
            v.0
        }
    }

    impl fmt::Display for TenantId {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }

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

        /// Parses or hashes a `DocId` from a string key.
        pub fn from_key(s: &str) -> super::error::Result<Self> {
            if s.is_empty() {
                return Err(super::error::ContextraError::invalid_input("empty DocId key"));
            }
            if let Ok(id) = s.parse::<u64>() {
                return Ok(Self(id));
            }
            // Deterministic FNV-1a 64-bit hash for arbitrary string keys.
            let mut hash: u64 = 0xcbf29ce484222325;
            for &b in s.as_bytes() {
                hash ^= u64::from(b);
                hash = hash.wrapping_mul(0x100000001b3);
            }
            Ok(Self(hash))
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
}
