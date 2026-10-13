//! Wiring entry point for database transaction orchestrator.

use super::db_transaction::DbTransaction;
use crate::Collection;
use contextra_ports::{StorageEngine, VectorIndex};
use contextra_types::{Result, TenantId, TxId};
use std::sync::Arc;

/// Dependencies required to construct [`TransactionWiring`].
pub struct TransactionWiringDeps<S: StorageEngine, V: VectorIndex = contextra_vector::HnswIndex> {
    pub collection: Collection<S, V>,
    pub ssi_validator: Option<Arc<dyn contextra_mvcc::SsiValidator + Send + Sync>>,
}

/// Facade entry point for database transaction management.
pub struct TransactionWiring<S: StorageEngine, V: VectorIndex = contextra_vector::HnswIndex> {
    collection: Collection<S, V>,
    ssi_validator: Option<Arc<dyn contextra_mvcc::SsiValidator + Send + Sync>>,
}

impl<S: StorageEngine, V: VectorIndex> TransactionWiring<S, V> {
    /// Builds a new [`TransactionWiring`] instance from supplied dependencies.
    pub fn build(deps: TransactionWiringDeps<S, V>) -> Result<Self> {
        Ok(Self {
            collection: deps.collection,
            ssi_validator: deps.ssi_validator,
        })
    }

    /// Begins a new transaction for a specific tenant ID.
    pub fn begin_for_tenant(&self, tx_id: TxId, tenant_id: TenantId) -> DbTransaction<S, V> {
        let mut tx = DbTransaction::new_for_tenant(self.collection.clone(), tx_id, tenant_id);
        if let Some(ref val) = self.ssi_validator {
            tx = tx.with_ssi_validator(Arc::clone(val));
        }
        tx
    }

    /// Begins a new transaction using the collection's default tenant ID.
    pub fn begin(&self, tx_id: TxId) -> DbTransaction<S, V> {
        let tenant = self.collection.tenant_id();
        self.begin_for_tenant(tx_id, tenant)
    }
}
