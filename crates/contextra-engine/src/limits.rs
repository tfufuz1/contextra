//! Engine operational limits enforcement (§16).
//!
//! Enforces Spec §16 bounds on write pathways (`remember`, batch operations, collection creation):
//! - `max_document_bytes`: 16 MiB (16,777,216 bytes)
//! - `max_chunks_per_document`: 4,096
//! - `max_batch_ops`: 10,000
//! - `max_collections_per_tenant`: 256
//! - `max_in_flight_per_principal`: 8
//!
//! Exceeding limits produces structured, non-retryable errors.

use contextra_types::{ContextraError, Result};
use serde::{Deserialize, Serialize};

/// Start values for Engine limits laut Spec §16 (dokumentiert als "Startwerte, zu kalibrieren").
pub const DEFAULT_MAX_DOCUMENT_BYTES: usize = 16 * 1024 * 1024; // 16 MiB
pub const DEFAULT_MAX_CHUNKS_PER_DOCUMENT: usize = 4_096;
pub const DEFAULT_MAX_BATCH_OPS: usize = 10_000;
pub const DEFAULT_MAX_COLLECTIONS_PER_TENANT: usize = 256;
pub const DEFAULT_MAX_IN_FLIGHT_PER_PRINCIPAL: usize = 8;

/// Configurable Engine limits structure (§16).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct EngineLimits {
    pub max_document_bytes: usize,
    pub max_chunks_per_document: usize,
    pub max_batch_ops: usize,
    pub max_collections_per_tenant: usize,
    pub max_in_flight_per_principal: usize,
}

impl Default for EngineLimits {
    fn default() -> Self {
        Self {
            max_document_bytes: DEFAULT_MAX_DOCUMENT_BYTES,
            max_chunks_per_document: DEFAULT_MAX_CHUNKS_PER_DOCUMENT,
            max_batch_ops: DEFAULT_MAX_BATCH_OPS,
            max_collections_per_tenant: DEFAULT_MAX_COLLECTIONS_PER_TENANT,
            max_in_flight_per_principal: DEFAULT_MAX_IN_FLIGHT_PER_PRINCIPAL,
        }
    }
}

impl EngineLimits {
    /// Validates document payload byte size.
    pub fn check_document_bytes(&self, bytes_len: usize) -> Result<()> {
        if bytes_len > self.max_document_bytes {
            return Err(ContextraError::InvalidInput(format!(
                "NON_RETRYABLE: LimitExceeded — max_document_bytes ({}) exceeded with {} bytes",
                self.max_document_bytes, bytes_len
            )));
        }
        Ok(())
    }

    /// Validates chunk count per document.
    pub fn check_chunks_per_document(&self, chunk_count: usize) -> Result<()> {
        if chunk_count > self.max_chunks_per_document {
            return Err(ContextraError::InvalidInput(format!(
                "NON_RETRYABLE: LimitExceeded — max_chunks_per_document ({}) exceeded with {} chunks",
                self.max_chunks_per_document, chunk_count
            )));
        }
        Ok(())
    }

    /// Validates batch operations count.
    pub fn check_batch_ops(&self, ops_count: usize) -> Result<()> {
        if ops_count > self.max_batch_ops {
            return Err(ContextraError::InvalidInput(format!(
                "NON_RETRYABLE: LimitExceeded — max_batch_ops ({}) exceeded with {} ops",
                self.max_batch_ops, ops_count
            )));
        }
        Ok(())
    }

    /// Validates collections count per tenant.
    pub fn check_collections_per_tenant(&self, collections_count: usize) -> Result<()> {
        if collections_count > self.max_collections_per_tenant {
            return Err(ContextraError::InvalidInput(format!(
                "NON_RETRYABLE: LimitExceeded — max_collections_per_tenant ({}) exceeded with {} collections",
                self.max_collections_per_tenant, collections_count
            )));
        }
        Ok(())
    }

    /// Validates in-flight requests per principal.
    pub fn check_in_flight_per_principal(&self, in_flight_count: usize) -> Result<()> {
        if in_flight_count > self.max_in_flight_per_principal {
            return Err(ContextraError::InvalidInput(format!(
                "NON_RETRYABLE: LimitExceeded — max_in_flight_per_principal ({}) exceeded with {} in-flight requests",
                self.max_in_flight_per_principal, in_flight_count
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_limit_max_document_bytes_boundary() {
        let limits = EngineLimits {
            max_document_bytes: 100,
            ..Default::default()
        };

        // n - 1
        assert!(limits.check_document_bytes(99).is_ok());
        // n
        assert!(limits.check_document_bytes(100).is_ok());
        // n + 1
        let err = limits.check_document_bytes(101).unwrap_err();
        assert!(err.to_string().contains("NON_RETRYABLE"));
    }

    #[test]
    fn test_limit_max_chunks_per_document_boundary() {
        let limits = EngineLimits {
            max_chunks_per_document: 10,
            ..Default::default()
        };

        // n - 1
        assert!(limits.check_chunks_per_document(9).is_ok());
        // n
        assert!(limits.check_chunks_per_document(10).is_ok());
        // n + 1
        let err = limits.check_chunks_per_document(11).unwrap_err();
        assert!(err.to_string().contains("NON_RETRYABLE"));
    }

    #[test]
    fn test_limit_max_batch_ops_boundary() {
        let limits = EngineLimits {
            max_batch_ops: 50,
            ..Default::default()
        };

        // n - 1
        assert!(limits.check_batch_ops(49).is_ok());
        // n
        assert!(limits.check_batch_ops(50).is_ok());
        // n + 1
        let err = limits.check_batch_ops(51).unwrap_err();
        assert!(err.to_string().contains("NON_RETRYABLE"));
    }

    #[test]
    fn test_limit_max_collections_per_tenant_boundary() {
        let limits = EngineLimits {
            max_collections_per_tenant: 5,
            ..Default::default()
        };

        // n - 1
        assert!(limits.check_collections_per_tenant(4).is_ok());
        // n
        assert!(limits.check_collections_per_tenant(5).is_ok());
        // n + 1
        let err = limits.check_collections_per_tenant(6).unwrap_err();
        assert!(err.to_string().contains("NON_RETRYABLE"));
    }

    #[test]
    fn test_limit_max_in_flight_per_principal_boundary() {
        let limits = EngineLimits {
            max_in_flight_per_principal: 8,
            ..Default::default()
        };

        // n - 1
        assert!(limits.check_in_flight_per_principal(7).is_ok());
        // n
        assert!(limits.check_in_flight_per_principal(8).is_ok());
        // n + 1
        let err = limits.check_in_flight_per_principal(9).unwrap_err();
        assert!(err.to_string().contains("NON_RETRYABLE"));
    }
}
