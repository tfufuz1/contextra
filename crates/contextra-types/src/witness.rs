// FILE-CONTEXT
// ZWECK: Typsichere Restmessung und LayerWitness für physische Löschnachweise (§4.2, §20.5, INV-WITNESS-MINT-1).
// INVARIANTEN: LayerWitness hat keinen öffentlichen Konstruktor außerhalb der Scanner (§7.3). Kein Default, kein Deserialize.
// HOTSPOTS: [1-60]

#![forbid(unsafe_code)]
//! LayerWitness and residual measurement types (§4.2, §20.5).

use serde::Serialize;

/// Scanner methodology applied during storage residue verification.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum ScanMethod {
    /// Direct structural scan of storage files, indices, or tables.
    Structural,
    /// Layer was not measured during this execution with specified reason.
    NotMeasured(String),
    /// Layer measurement is disabled due to feature configuration.
    FeatureDisabled,
}

/// Verified witness measurement issued by a storage layer scanner.
///
/// Under INV-WITNESS-MINT-1, `LayerWitness` instances may only be minted
/// by authorized storage scanners (`store`, `vector`, `text`, etc.).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LayerWitness {
    layer: String,
    scanned: u64,
    remaining: u64,
    method: ScanMethod,
    digest: [u8; 32],
}

impl LayerWitness {
    /// Internal crate constructor for scanner implementations within contextra-types.
    pub(crate) fn issue_from_scanner_internal(
        layer: impl Into<String>,
        scanned: u64,
        remaining: u64,
        method: ScanMethod,
        digest: [u8; 32],
    ) -> Self {
        Self {
            layer: layer.into(),
            scanned,
            remaining,
            method,
            digest,
        }
    }

    /// Gate entrypoint for authorized scanner implementations in storage crates (INV-WITNESS-MINT-1).
    ///
    /// # Gate-Stelle
    /// This function is hidden from public documentation (`#[doc(hidden)]`) and serves as the audited
    /// gate entrypoint for issuing `LayerWitness` measurements. Instantiation outside authorized
    /// storage scanners is strictly prohibited by governance rules.
    #[doc(hidden)]
    pub fn issue_from_scanner(
        layer: impl Into<String>,
        scanned: u64,
        remaining: u64,
        method: ScanMethod,
        digest: [u8; 32],
    ) -> Self {
        Self::issue_from_scanner_internal(layer, scanned, remaining, method, digest)
    }

    /// Returns the identifier of the scanned storage layer.
    pub fn layer(&self) -> &str {
        &self.layer
    }

    /// Returns the number of items/keys scanned during verification.
    pub fn scanned(&self) -> u64 {
        self.scanned
    }

    /// Returns the number of remaining live items/keys found.
    pub fn remaining(&self) -> u64 {
        self.remaining
    }

    /// Returns the scan methodology used.
    pub fn method(&self) -> &ScanMethod {
        &self.method
    }

    /// Returns the BLAKE3 digest of the scan state.
    pub fn digest(&self) -> &[u8; 32] {
        &self.digest
    }
}

/// Request context containing tenant and transaction identity for deletion operations.
#[derive(Debug, Clone, PartialEq, Eq)]
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
#[derive(Debug, Clone, PartialEq, Eq)]
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
