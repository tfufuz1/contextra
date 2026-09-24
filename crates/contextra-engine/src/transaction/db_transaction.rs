use super::intent::StagedKeyOp;
use crate::Collection;
use bytes::Bytes;
use contextra_types::{DocId, Edge, Entity, EntityId, TxId};
use contextra_ports::{GraphIndex, StorageEngine, TextIndex, VectorIndex};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::sync::Mutex;

/// A transaction wrapper that ensures atomic multi-index commits across LSM-Store, HNSW-Index, Text-Index, and Graph-Index.
pub struct DbTransaction<S: StorageEngine, V: VectorIndex = contextra_vector::HnswIndex> {
    pub tx_id: TxId,
    pub(super) collection: Collection<S, V>,
    pub(super) staged_forward_keys: Mutex<Vec<StagedKeyOp>>,
    pub(super) staged_reverse_keys: Mutex<Vec<StagedKeyOp>>,
    pub(super) staged_doc_ids: Mutex<Arc<Vec<DocId>>>,
    pub(super) staged_text_ops: Mutex<Vec<(DocId, String)>>,
    pub(super) staged_text_deletes: Mutex<Vec<DocId>>,
    pub(super) staged_graph_entities: Mutex<Vec<Entity>>,
    pub(super) staged_graph_edges: Mutex<Vec<Edge>>,
    pub(super) staged_graph_entity_deletes: Mutex<Vec<EntityId>>,
    pub(super) staged_graph_edge_deletes: Mutex<Vec<(EntityId, EntityId)>>,
    pub(super) staged_hyperedges: Mutex<Vec<contextra_graph::hyperedge::HyperEdge>>,
    pub(super) committed: AtomicBool,
}

impl<S: StorageEngine, V: VectorIndex> DbTransaction<S, V> {
    pub fn new(collection: Collection<S, V>, tx_id: TxId) -> Self {
        Self {
            tx_id,
            collection,
            staged_forward_keys: Mutex::new(Vec::with_capacity(16)),
            staged_reverse_keys: Mutex::new(Vec::with_capacity(16)),
            staged_doc_ids: Mutex::new(Arc::new(Vec::with_capacity(16))),
            staged_text_ops: Mutex::new(Vec::with_capacity(16)),
            staged_text_deletes: Mutex::new(Vec::with_capacity(16)),
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
        guard.push((doc_id, text));
    }

    pub fn stage_text_delete(&self, doc_id: DocId) {
        let mut guard = match self.staged_text_deletes.lock() {
            Ok(g) => g,
            Err(p) => p.into_inner(),
        };
        guard.push(doc_id);
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
                    if let Err(e) = collection.graph_index.rollback(tx_id).await {
                        tracing::error!(
                            "[INV-DB-3] Drop cleanup: Graph index rollback failed: {}",
                            e
                        );
                    }
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
