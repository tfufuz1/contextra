//! Resumable document-level HNSW erasure and immediate RAM masking.

use std::{collections::BTreeSet, sync::Arc};

use contextra_ports::{
    erasure::{DocScope, ErasureLayer, PurgeCx, PurgeOutcome, ResolvedScope},
    scanner::{LayerScanner, WitnessLayer},
    LayerError,
    BoxFuture,
};
use contextra_types::{Cursor, DocId, LayerProgress};

use super::{scanner::HnswScanner, types::HnswIndex};

/// HNSW participant in the erasure protocol.
pub struct HnswErasure {
    index: Arc<HnswIndex>,
    scanner: HnswScanner,
}

impl HnswErasure {
    /// Creates an erasure layer for an index.
    pub fn new(index: Arc<HnswIndex>) -> Self {
        Self { scanner: HnswScanner::new(Arc::clone(&index)), index }
    }

    fn scoped_docs(&self, scope: &ResolvedScope) -> BTreeSet<DocId> {
        let broad = scope.docs.iter().any(|d| !matches!(d, DocScope::Set { .. }));
        let mut docs: BTreeSet<DocId> = scope.docs.iter().flat_map(|d| match d {
            DocScope::Set { docs, .. } => docs.iter().copied().collect::<Vec<_>>(),
            DocScope::Collection { .. } | DocScope::Tenant { .. } => Vec::new(),
        }).collect();
        if broad {
            docs.extend(self.index.inner.hot.doc_to_node.read().keys().copied().map(DocId::new));
        }
        docs
    }

    fn layer_error(kind: contextra_types::LayerErrorKind, detail: &'static str) -> LayerError {
        LayerError { kind, layer: WitnessLayer::HnswIndex, detail }
    }
}

impl ErasureLayer for HnswErasure {
    fn layer(&self) -> WitnessLayer { WitnessLayer::HnswIndex }

    fn on_gate<'a>(&'a self, scope: &'a ResolvedScope) -> BoxFuture<'a, std::result::Result<(), LayerError>> {
        Box::pin(async move {
            let docs = self.scoped_docs(scope);
            let mut mask = self.index.inner.cold.erased_docs.write();
            if docs.iter().all(|doc| mask.contains(doc)) { return Ok(()); }
            let mut next = (**mask).clone();
            next.extend(docs);
            *mask = Arc::new(next);
            Ok(())
        })
    }

    fn purge<'a>(&'a self, scope: &'a ResolvedScope, cx: &'a PurgeCx<'a>) -> BoxFuture<'a, std::result::Result<PurgeOutcome, LayerError>> {
        Box::pin(async move {
            let docs = self.scoped_docs(scope);
            let mut nodes: Vec<usize> = {
                let map = self.index.inner.hot.doc_to_node.read();
                docs.iter().filter_map(|d| map.get(&d.inner()).copied()).collect()
            };
            nodes.sort_unstable();
            nodes.dedup();
            let done_before = match cx.cursor.as_ref().map(Cursor::as_bytes) {
                Some(bytes) if bytes.len() == 17 && bytes[0] == 1 && bytes[1..9] == scope.plan_hash[..8] =>
                    u64::from_le_bytes(bytes[9..17].try_into().map_err(|_| Self::layer_error(contextra_types::LayerErrorKind::Permanent, "invalid_cursor"))?),
                Some(_) => return Err(Self::layer_error(contextra_types::LayerErrorKind::Permanent, "invalid_cursor")),
                None => 0,
            };
            // Deleting a node changes the sorted worklist, so a numeric offset would skip
            // nodes on resume. Always consume its current prefix and revisit until empty.
            let batch_len = nodes.len().min(cx.max_units.max(1));
            if cx.cancel.is_cancelled() { return Err(Self::layer_error(contextra_types::LayerErrorKind::Cancelled, "cancelled")); }
            let batch = &nodes[..batch_len];
            if !batch.is_empty() {
                for &idx in batch {
                    let doc = self.index.inner.hot.nodes.read().get(idx).map(|n| n.doc_id);
                    if let Some(doc) = doc { self.index.remove(doc).map_err(|_| Self::layer_error(contextra_types::LayerErrorKind::Retryable, "node_removal_failed"))?; }
                }
            }
            let done = done_before.saturating_add(batch_len as u64);
            let cursor = if nodes.len() > batch_len {
                let mut raw = vec![1]; raw.extend_from_slice(&scope.plan_hash[..8]); raw.extend_from_slice(&done.to_le_bytes());
                Some(Cursor::new(raw).map_err(|_| Self::layer_error(contextra_types::LayerErrorKind::Permanent, "cursor_invalid"))?)
            } else { None };
            let progress = LayerProgress { cursor, units_done: done, bytes_freed: 0 };
            let layer = WitnessLayer::HnswIndex;
            cx.progress.record(&layer, &progress).await.map_err(|_| Self::layer_error(contextra_types::LayerErrorKind::Retryable, "progress_record_failed"))?;
            Ok(PurgeOutcome { progress, units_committed: batch_len as u64, bytes_freed: 0 })
        })
    }

    fn scanner(&self) -> &dyn LayerScanner { &self.scanner }
}
