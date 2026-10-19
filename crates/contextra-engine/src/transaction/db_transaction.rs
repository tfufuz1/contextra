use super::intent::StagedKeyOp;
use crate::Collection;
use bytes::Bytes;
use contextra_ports::{StorageEngine, TextIndex, VectorIndex};
use contextra_types::{DocId, Edge, Entity, EntityId, TenantId, TxId};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Debug, Clone)]
pub(super) enum StagedTextOp {
    Upsert(DocId, String),
    Delete(DocId),
}

/// A transaction wrapper that ensures atomic multi-index commits across LSM-Store, HNSW-Index, Text-Index, and Graph-Index.
pub struct DbTransaction<S: StorageEngine, V: VectorIndex = contextra_vector::HnswIndex> {
    pub tx_id: TxId,
    pub tenant_id: TenantId,
    pub(super) collection: Collection<S, V>,
    pub(super) read_set: Mutex<contextra_mvcc::ReadSet>,
    pub(super) ssi_validator: Option<Arc<dyn contextra_mvcc::SsiValidator + Send + Sync>>,
    pub(super) staged_forward_keys: Mutex<Vec<StagedKeyOp>>,
    pub(super) staged_reverse_keys: Mutex<Vec<StagedKeyOp>>,
    pub(super) staged_doc_ids: Mutex<Arc<Vec<DocId>>>,
    pub(super) staged_text_ops: Mutex<Vec<(DocId, String)>>,
    pub(super) staged_text_deletes: Mutex<Vec<DocId>>,
    pub(super) staged_text_sequence: Mutex<Vec<StagedTextOp>>,
    pub(super) staged_graph_entities: Mutex<Vec<Entity>>,
    pub(super) staged_graph_edges: Mutex<Vec<Edge>>,
    pub(super) staged_graph_entity_deletes: Mutex<Vec<EntityId>>,
    pub(super) staged_graph_edge_deletes: Mutex<Vec<(EntityId, EntityId)>>,
    pub(super) staged_hyperedges: Mutex<Vec<contextra_graph::hyperedge::HyperEdge>>,
    pub(super) committed: AtomicBool,
}

impl<S: StorageEngine, V: VectorIndex> DbTransaction<S, V> {
    pub fn new(collection: Collection<S, V>, tx_id: TxId) -> Self {
        let tenant_id = collection.tenant_id();
        Self::new_for_tenant(collection, tx_id, tenant_id)
    }

    pub fn new_for_tenant(collection: Collection<S, V>, tx_id: TxId, tenant_id: TenantId) -> Self {
        Self {
            tx_id,
            tenant_id,
            collection,
            read_set: Mutex::new(contextra_mvcc::ReadSet::new()),
            ssi_validator: None,
            staged_forward_keys: Mutex::new(Vec::with_capacity(16)),
            staged_reverse_keys: Mutex::new(Vec::with_capacity(16)),
            staged_doc_ids: Mutex::new(Arc::new(Vec::with_capacity(16))),
            staged_text_ops: Mutex::new(Vec::with_capacity(16)),
            staged_text_deletes: Mutex::new(Vec::with_capacity(16)),
            staged_text_sequence: Mutex::new(Vec::with_capacity(16)),
            staged_graph_entities: Mutex::new(Vec::with_capacity(16)),
            staged_graph_edges: Mutex::new(Vec::with_capacity(16)),
            staged_graph_entity_deletes: Mutex::new(Vec::with_capacity(16)),
            staged_graph_edge_deletes: Mutex::new(Vec::with_capacity(16)),
            staged_hyperedges: Mutex::new(Vec::with_capacity(16)),
            committed: AtomicBool::new(false),
        }
    }

    /// Records keys and IDs that have been operated on for potential compensating rollback.
    pub fn record_keys(&self, forward: Vec<u8>, reverse: Vec<u8>, doc_id: DocId) {
        self.record_keys_with_old_values(forward, None, reverse, None, doc_id);
    }

    /// Records keys and IDs along with pre-write values for potential compensating rollback.
    pub fn record_keys_with_old_values(
        &self,
        forward: Vec<u8>,
        old_forward: Option<Bytes>,
        reverse: Vec<u8>,
        old_reverse: Option<Bytes>,
        doc_id: DocId,
    ) {
        let mut fw = match self.staged_forward_keys.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        fw.push((forward, old_forward));

        let mut rev = match self.staged_reverse_keys.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        rev.push((reverse, old_reverse));

        let mut ids = match self.staged_doc_ids.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        Arc::make_mut(&mut ids).push(doc_id);
    }

    pub fn stage_text_insert(&self, doc_id: DocId, text: String) {
        let mut guard = match self.staged_text_ops.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        guard.push((doc_id, text.clone()));

        let mut seq = match self.staged_text_sequence.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        seq.push(StagedTextOp::Upsert(doc_id, text));
    }

    pub fn stage_text_delete(&self, doc_id: DocId) {
        let mut guard = match self.staged_text_deletes.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        guard.push(doc_id);

        let mut seq = match self.staged_text_sequence.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        seq.push(StagedTextOp::Delete(doc_id));
    }

    pub fn stage_graph_entity(&self, entity: Entity) {
        let mut guard = match self.staged_graph_entities.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        guard.push(entity);
    }

    pub fn stage_graph_edge(&self, edge: Edge) {
        let mut guard = match self.staged_graph_edges.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        guard.push(edge);
    }

    pub fn stage_graph_entity_delete(&self, entity_id: EntityId) {
        let mut guard = match self.staged_graph_entity_deletes.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        guard.push(entity_id);
    }

    pub fn stage_graph_edge_delete(&self, from: EntityId, to: EntityId) {
        let mut guard = match self.staged_graph_edge_deletes.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        guard.push((from, to));
    }

    pub fn stage_hyperedge(&self, hyperedge: contextra_graph::hyperedge::HyperEdge) {
        let mut guard = match self.staged_hyperedges.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        guard.push(hyperedge);
    }

    /// Attaches an explicit SSI validator implementation to this transaction.
    pub fn with_ssi_validator(
        mut self,
        validator: Arc<dyn contextra_mvcc::SsiValidator + Send + Sync>,
    ) -> Self {
        self.ssi_validator = Some(validator);
        self
    }

    /// Records a read operation for a key at `snapshot_seq` in this transaction's `ReadSet`.
    pub fn record_read(&self, key: impl Into<Vec<u8>>, snapshot_seq: u64) {
        let mut guard = match self.read_set.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        guard.record_read(key, snapshot_seq);
    }

    /// Records a range prefix read operation at `snapshot_seq` for phantom protection.
    pub fn record_prefix_read(&self, prefix: impl Into<Vec<u8>>, snapshot_seq: u64) {
        let mut guard = match self.read_set.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        guard.record_prefix(prefix, snapshot_seq);
    }
}

impl<S: StorageEngine, V: VectorIndex> Drop for DbTransaction<S, V> {
    fn drop(&mut self) {
        if !self.committed.load(Ordering::Acquire) {
            tracing::warn!(
                tx_id = ?self.tx_id,
                "DbTransaction dropped without commit — rollback signal sent"
            );
            let collection = self.collection.clone();
            let tx_id = self.tx_id;
            if let Ok(handle) = tokio::runtime::Handle::try_current() {
                handle.spawn(async move {
                    if let Err(e) = collection.text_index.rollback(tx_id).await {
                        tracing::error!(
                            "[INV-DB-3] Drop cleanup: Text index rollback failed: {}",
                            e
                        );
                    }
                    if let Err(e) = collection.index.rollback(tx_id).await {
                        tracing::error!(
                            "[INV-DB-3] Drop cleanup: Vector index rollback failed: {}",
                            e
                        );
                    }
                    if let Err(e) = collection.storage.rollback(tx_id).await {
                        tracing::error!("[INV-DB-3] Drop cleanup: Storage rollback failed: {}", e);
                    }
                });
            }
        }
    }
}
