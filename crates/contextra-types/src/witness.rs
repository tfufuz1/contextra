// FILE-CONTEXT
// ZWECK: Typsichere Restmessung und Basistypen für physische Löschnachweise (§4.2, §20.5, INV-WITNESS-MINT-1).
// INVARIANTEN: LayerWitness liegt ausschließlich in contextra-ports::scanner.
// HOTSPOTS: [1-40]

#![forbid(unsafe_code)]
//! Residual measurement and context types (§4.2, §20.5).

use serde::{Deserialize, Serialize};

/// Scanner methodology applied during storage residue verification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScanMethod {
    /// Direct structural scan of storage files, indices, or tables.
    Structural,
    /// Layer was not measured during this execution with specified reason.
    NotMeasured(String),
    /// Layer measurement is disabled due to feature configuration.
    FeatureDisabled,
}

/// Request context containing tenant and transaction identity for deletion operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestContext {
    /// Tenant identifier for authorization and scope isolation.
    pub tenant_id: crate::TenantId,
    /// Transaction ID allocated for the operation.
    pub tx_id: crate::TxId,
}

impl RequestContext {
    /// Creates a new `RequestContext`.
    pub fn new(tenant_id: crate::TenantId, tx_id: crate::TxId) -> Self {
        Self { tenant_id, tx_id }
    }
}

/// Saga record containing deletion tracking state and digests.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SagaRecord {
    /// BLAKE3 hash of deleted keys.
    pub deleted_keys_hash: [u8; 32],
}

impl SagaRecord {
    /// Creates a new `SagaRecord`.
    pub fn new(deleted_keys_hash: [u8; 32]) -> Self {
        Self { deleted_keys_hash }
    }
}
