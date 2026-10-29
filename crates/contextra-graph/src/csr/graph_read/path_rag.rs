use super::super::graph_write::CsrGraph;
use ahash::AHashMap;
use contextra_types::{EntityId, Result, TxId};

impl CsrGraph {
    pub fn path_rag_at<'a>(
        &'a self,
        start_nodes: &'a [EntityId],
        max_hops: usize,
        seq_no: u64,
    ) -> contextra_ports::BoxFuture<'a, Result<Vec<(EntityId, f32)>>> {
        Box::pin(async move {
            let as_of_tx = TxId::new(seq_no);
            if super::super::visibility::is_suspicious_tx_id(as_of_tx) {
                tracing::warn!(
                    tx_id = seq_no,
                    hint = if as_of_tx == TxId::INVALID {
                        "Sentinel TxId(0)"
                    } else {
                        "Wall-Clock-ns-Bereich"
                    },
                    "AGT-GRAPH-001: Verdächtiger oder unallozierter seq_no in path_rag_at"
                );
            }
            if let Err(err) = self.compact_async().await {
                tracing::warn!(
                    error = %err,
                    "compact_async failed during path_rag_at compaction"
                );
            }

            let inner = self.inner_read();
            let snapshot_graph = crate::path_rag::SnapshotPathGraph::new(&inner, as_of_tx);
            let engine = crate::path_rag::PathRAGEngine::new(
                snapshot_graph,
                max_hops,
                crate::path_rag::DEFAULT_SUFFICIENCY_THRESHOLD,
            );

            let mut combined: AHashMap<EntityId, f32> = AHashMap::default();

            for &start in start_nodes {
                let paths = engine.find_all_paths(start);
                let rrf_signal = engine.to_rrf_signal(&paths);
                for (doc_id, score) in rrf_signal {
                    #[cfg(not(feature = "docid-128"))]
                    let entity_id = EntityId::new(doc_id.0);
                    #[cfg(feature = "docid-128")]
                    let entity_id = EntityId::new(doc_id.0 as u64);

                    combined
                        .entry(entity_id)
                        .and_modify(|s| *s = s.max(score))
                        .or_insert(score);
                }
            }

            let mut results: Vec<(EntityId, f32)> = combined.into_iter().collect();
            results.sort_by(|a, b| {
                b.1.total_cmp(&a.1)
                    .then_with(|| a.0.inner().cmp(&b.0.inner()))
            });

            Ok(results)
        })
    }
}
