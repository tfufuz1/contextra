// FILE-CONTEXT
// ZWECK: Hydrierungs-Familie (hydrate_from_scored_at, hydrate_from_tuples_at) für Collection.

use super::{Collection, StoredDocument, StoredDocumentMeta};
use contextra_core::{DocId, Result, StorageEngine, VectorIndex};

impl<S: StorageEngine, V: VectorIndex> Collection<S, V> {
    pub(super) async fn hydrate_from_scored_at(
        &self,
        scored_docs: Vec<contextra_core::ScoredDocument>,
        seq: u64,
    ) -> Result<(Vec<crate::SearchResult>, usize)> {
        if scored_docs.is_empty() {
            return Ok((Vec::new(), 0));
        }

        let mut results = Vec::with_capacity(scored_docs.len());
        let mut skipped_tombstones = 0usize;
        for sd in scored_docs {
            let doc_key = self.namespaced_key(&sd.doc_id.inner().to_le_bytes(), 1);
            if let Some(bytes) = self.storage.get_at_seq(&doc_key, seq).await? {
                let (id, metadata) =
                    if let Ok(meta) = serde_json::from_slice::<StoredDocumentMeta>(&bytes) {
                        (meta.id, meta.metadata)
                    } else if let Ok(full) = serde_json::from_slice::<StoredDocument>(&bytes) {
                        (full.id, full.metadata)
                    } else {
                        tracing::warn!(doc_id = ?sd.doc_id, "Could not deserialize doc_key");
                        skipped_tombstones += 1;
                        continue;
                    };
                let rank = (results.len() + 1) as u32;
                let rrf_contrib = 1.0 / (60.0 + rank as f32);
                let prov = crate::fusion::ProvenanceBuilder::new(60.0)
                    .vector(sd.score, rank, 1.0)
                    .source_collection(self.name.clone())
                    .index_type("hnsw")
                    .expected_total(rrf_contrib)
                    .build();
                results.push(crate::SearchResult {
                    id,
                    score: sd.score,
                    metadata,
                    matched_signals: vec!["vector".to_string()],
                    provenance: Some(prov),
                });
            } else {
                skipped_tombstones += 1;
            }
        }
        Ok((results, skipped_tombstones))
    }

    pub(super) async fn hydrate_from_tuples_at(
        &self,
        scored_tuples: Vec<(DocId, f32)>,
        seq: u64,
    ) -> Result<Vec<crate::SearchResult>> {
        if scored_tuples.is_empty() {
            return Ok(Vec::new());
        }

        let mut results = Vec::with_capacity(scored_tuples.len());
        let mut skipped_tombstones = 0usize;
        for (doc_id, score) in scored_tuples {
            let doc_key = self.namespaced_key(&doc_id.inner().to_le_bytes(), 1);
            if let Some(bytes) = self.storage.get_at_seq(&doc_key, seq).await? {
                let (id, metadata) =
                    if let Ok(meta) = serde_json::from_slice::<StoredDocumentMeta>(&bytes) {
                        (meta.id, meta.metadata)
                    } else if let Ok(full) = serde_json::from_slice::<StoredDocument>(&bytes) {
                        (full.id, full.metadata)
                    } else {
                        tracing::warn!(doc_id = ?doc_id, "Could not deserialize doc_key");
                        skipped_tombstones += 1;
                        continue;
                    };
                let prov = crate::ProvenanceRecord {
                    source_collection: Some(self.name.clone()),
                    ..Default::default()
                };
                results.push(crate::SearchResult {
                    id,
                    score,
                    metadata,
                    matched_signals: vec![],
                    provenance: Some(prov),
                });
            } else {
                skipped_tombstones += 1;
            }
        }
        if skipped_tombstones > 0 {
            tracing::debug!(
                skipped_tombstones = skipped_tombstones,
                hydrated_count = results.len(),
                "Tombstones skipped during tuple search hydration"
            );
        }
        Ok(results)
    }
}
