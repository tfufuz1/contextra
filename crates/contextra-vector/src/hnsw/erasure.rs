//! Resumable document-level HNSW erasure and immediate RAM masking.

use std::{collections::BTreeSet, sync::Arc};

use contextra_ports::{
    erasure::{DocScope, ErasureLayer, PurgeCx, PurgeOutcome, ResolvedScope},
    scanner::LayerScanner,
    BoxFuture,
};
use contextra_types::{Cursor, DocId, ErasureError, LayerError, LayerProgress, WitnessLayer};

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

    fn purge<'a>(&'a self, scope: &'a ResolvedScope, cx: &'a PurgeCx<'a>) -> BoxFuture<'a, std::result::Result<PurgeOutcome, ErasureError>> {
        Box::pin(async move {
            let docs = self.scoped_docs(scope);
            let map = self.index.inner.hot.doc_to_node.read();
            let mut nodes: Vec<usize> = docs.iter().filter_map(|d| map.get(&d.inner()).copied()).collect();
            nodes.sort_unstable();
            nodes.dedup();
            let start = match cx.cursor.as_ref().map(Cursor::as_bytes) {
                Some(bytes) if bytes.len() == 17 && bytes[0] == 1 && bytes[1..9] == scope.plan_hash[..8] =>
                    usize::try_from(u64::from_le_bytes(bytes[9..17].try_into().map_err(|_| ErasureError::InvalidCursor)?)).map_err(|_| ErasureError::InvalidCursor)?,
                Some(_) => return Err(ErasureError::InvalidCursor),
                None => 0,
            };
            let end = nodes.len().min(start.saturating_add(cx.max_units.max(1)));
            drop(map);
            if cx.cancel.is_cancelled() { return Err(ErasureError::Cancelled); }
            let batch = &nodes[start.min(nodes.len())..end];
            if !batch.is_empty() {
                for &idx in batch {
                    let doc = self.index.inner.hot.nodes.read().get(idx).map(|n| n.doc_id);
                    if let Some(doc) = doc { self.index.remove(doc).map_err(|_| ErasureError::Io("HNSW node removal failed".into()))?; }
                }
            }
            let done = end as u64;
            let cursor = if end < nodes.len() {
                let mut raw = vec![1]; raw.extend_from_slice(&scope.plan_hash[..8]); raw.extend_from_slice(&done.to_le_bytes());
                Some(Cursor::new(raw)?)
            } else { None };
            let progress = LayerProgress { cursor, units_done: done, bytes_freed: 0 };
            let layer = WitnessLayer::HnswIndex;
            cx.progress.record(&layer, &progress).await?;
            Ok(PurgeOutcome { progress, units_committed: (end - start) as u64, bytes_freed: 0 })
        })
    }

    fn scanner(&self) -> &dyn LayerScanner { &self.scanner }
}
