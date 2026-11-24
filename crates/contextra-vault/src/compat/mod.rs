//! Kompatibilitäts-Brücke für das Feature `workspace`.
//!
//! Stellt Konvertierungen zwischen lokalen Vault-Typen und `contextra_types` bereit.
//! Gemäß R4 der Spezifikation existiert dieser Bezug ausschließlich hier hinter dem Feature `workspace`.

use contextra_types::error::ContextraError;
use contextra_types::{DocId, TenantId, TxId};

use crate::error::VaultError;
use crate::vault::{VaultDocId, VaultTenantId, VaultTxId};

impl From<VaultDocId> for DocId {
    #[inline]
    fn from(id: VaultDocId) -> Self {
        DocId(id.0)
    }
}

impl From<DocId> for VaultDocId {
    #[inline]
    fn from(id: DocId) -> Self {
        VaultDocId(id.0)
    }
}

impl From<VaultTxId> for TxId {
    #[inline]
    fn from(id: VaultTxId) -> Self {
        TxId(id.0)
    }
}

impl From<TxId> for VaultTxId {
    #[inline]
    fn from(id: TxId) -> Self {
        VaultTxId(id.0)
    }
}

impl From<VaultTenantId> for TenantId {
    #[inline]
    fn from(id: VaultTenantId) -> Self {
        TenantId::try_new(id.0).unwrap_or(TenantId::SYSTEM)
    }
}

impl From<TenantId> for VaultTenantId {
    #[inline]
    fn from(id: TenantId) -> Self {
        VaultTenantId(id.as_u64())
    }
}

impl From<VaultError> for ContextraError {
    fn from(err: VaultError) -> Self {
        match err {
            VaultError::CapacityExceeded { .. } => ContextraError::PolicyViolation(err.to_string()),
            VaultError::AlreadyConsumed => ContextraError::InvalidInput(err.to_string()),
        }
    }
}
