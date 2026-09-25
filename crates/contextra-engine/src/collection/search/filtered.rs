// FILE-CONTEXT
// ZWECK: Filter-Familie (Selectivity, Match-DocIDs, search_filtered) für Collection.

use super::checkpoint::with_pinned_checkpoint_at_latest;
use super::{Collection, StoredDocument, StoredDocumentMeta};
use contextra_ports::{StorageEngine, VectorIndex};
use contextra_types::{DocId, FilterExpr, Result};

impl<S: StorageEngine, V: VectorIndex> Collection<S, V> {
    /// Estimates the selectivity (fraction of documents matching `filter`) over the collection metadata.
    ///
    /// For collections under 1,000 documents, this performs an exact metadata evaluation.
    /// For larger collections, it samples up to 200 document metadata entries to derive an estimate `s in (0.0, 1.0]`.
    pub(super) async fn estimate_filter_selectivity(
        &self,
        filter: &FilterExpr,
        seq: u64,
        total_docs: usize,
    ) -> Result<f64> {
        if total_docs == 0 {
            return Ok(1.0);
        }
        if total_docs < 1000 {
            let matched = self.get_matching_doc_ids_at(filter, seq).await?;
            return Ok(matched.len() as f64 / total_docs as f64);
        }

        const SAMPLE_SIZE: usize = 200;
        let prefix = if self.name == "default" {
            b"__docid:".to_vec()
        } else {
            let mut p = self.prefix.clone();
            p.push(1); // docid mapping type
            p
        };

        let entries = self.storage.scan_prefix_at(&prefix, seq).await?;
        if entries.is_empty() {
            return Ok(1.0);
        }

        let mut total_sampled = 0;
        let mut matched_sampled = 0;

        for (_, v) in entries.iter().take(SAMPLE_SIZE) {
            total_sampled += 1;
            let doc_metadata = if let Ok(meta) = serde_json::from_slice::<StoredDocumentMeta>(v) {
                meta.metadata
            } else if let Ok(full) = serde_json::from_slice::<StoredDocument>(v) {
                full.metadata
            } else {
                None
            };
            let metadata = doc_metadata.as_ref().unwrap_or(&serde_json::Value::Null);
            if filter.evaluate(metadata) {
                matched_sampled += 1;
            }
        }

        if total_sampled == 0 {
            return Ok(1.0);
        }

        let selectivity = matched_sampled as f64 / total_sampled as f64;
        if selectivity == 0.0 {
            Ok((1.0 / total_docs as f64).max(0.0005))
        } else {
            Ok(selectivity)
        }
    }

    pub(super) async fn get_matching_doc_ids_at(
        &self,
        filter: &FilterExpr,
        seq: u64,
    ) -> Result<std::collections::HashSet<DocId>> {
        let prefix = if self.name == "default" {
            b"__docid:".to_vec()
        } else {
            let mut p = self.prefix.clone();
            p.push(1); // docid mapping type
            p
        };

        let entries = self.storage.scan_prefix_at(&prefix, seq).await?;
        let mut matched = std::collections::HashSet::new();

        for (_, v) in entries {
            let (id, doc_metadata) =
                if let Ok(meta) = serde_json::from_slice::<StoredDocumentMeta>(&v) {
                    (meta.id, meta.metadata)
                } else if let Ok(full) = serde_json::from_slice::<StoredDocument>(&v) {
                    (full.id, full.metadata)
                } else {
                    continue;
                };
            let metadata = doc_metadata.as_ref().unwrap_or(&serde_json::Value::Null);
            if filter.evaluate(metadata) {
                matched.insert(DocId::from_key(&id)?);
            }
        }

        Ok(matched)
    }

    pub(super) async fn get_matching_doc_ids_for_query_at(
        &self,
        query: &contextra_types::HybridQuery,
        seq: u64,
    ) -> Result<Option<std::collections::HashSet<DocId>>> {
        if query.filter.is_none() && query.memory_type_filter.is_none() {
            return Ok(None);
        }

        let prefix = if self.name == "default" {
            b"__docid:".to_vec()
        } else {
            let mut p = self.prefix.clone();
            p.push(1); // docid mapping type
            p
        };

        let entries = self.storage.scan_prefix_at(&prefix, seq).await?;
        let mut matched = std::collections::HashSet::new();

        for (_, v) in entries {
            let (id_str, doc_metadata) =
                if let Ok(meta) = serde_json::from_slice::<StoredDocumentMeta>(&v) {
                    (meta.id, meta.metadata)
                } else if let Ok(full) = serde_json::from_slice::<StoredDocument>(&v) {
                    (full.id, full.metadata)
                } else {
                    continue;
                };

            if let Some(ref filter_expr) = query.filter {
                let metadata = doc_metadata.as_ref().unwrap_or(&serde_json::Value::Null);
                if !filter_expr.evaluate(metadata) {
                    continue;
                }
            }

            if let Some(ref type_filter) = query.memory_type_filter {
                let memory_type = crate::filter::extract_memory_type(&doc_metadata);
                if !type_filter.contains(&memory_type) {
                    continue;
                }
            }

            if let Ok(doc_id) = DocId::from_key(&id_str) {
                matched.insert(doc_id);
            }
        }

        Ok(Some(matched))
    }

    /// Performs filtered semantic vector search in the collection.
    // AI-TAG[SMELL][RESOLVED] audit-5.1: Vector-Suche clampt k stets auf k.min(contextra_types::MAX_SEARCH_K) über alle Einstiegspunkte hinweg.
    // AI-TAG[SMELL][RESOLVED] audit-M-7: CheckpointPinGuard handles unpinning safely across all error return paths.
    #[deprecated(since = "0.1.0", note = "use Collection::query() instead")]
    #[allow(deprecated)]
    #[tracing::instrument(level = "trace", skip(self, query, filter))]
    pub async fn search_filtered(
        &self,
        query: &[f32],
        k: usize,
        filter: Option<&(dyn Fn(DocId) -> bool + Send + Sync)>,
    ) -> Result<Vec<crate::SearchResult>> {
        let k = k.min(contextra_types::MAX_SEARCH_K);
        // Pin-first, read-after: the seq is read under the protection of the pin,
        // eliminating the TOCTOU window between snapshot_seq() and pin activation.
        with_pinned_checkpoint_at_latest(self.storage.as_ref(), |seq| async move {
            self.search_filtered_at(query, k, filter, seq).await
        })
        .await
    }

    /// Performs filtered semantic vector search at a specific MVCC sequence number.
    #[deprecated(since = "0.1.0", note = "use Collection::query() instead")]
    pub async fn search_filtered_at(
        &self,
        query: &[f32],
        k: usize,
        filter: Option<&(dyn Fn(DocId) -> bool + Send + Sync)>,
        seq: u64,
    ) -> Result<Vec<crate::SearchResult>> {
        if query.len() != self.dimension {
            return Err(contextra_types::ContextraError::invalid_input(format!(
                "Dimension mismatch: expected {}, got {}",
                self.dimension,
                query.len()
            )));
        }
        if k == 0 {
            return Err(contextra_types::ContextraError::invalid_input(
                "Search k must be greater than 0",
            ));
        }
        let target_k = k.min(contextra_types::MAX_SEARCH_K);
        let mut fetch_k = target_k;

        loop {
            let scored_docs = self.index.search_filtered(query, fetch_k, filter).await?;
            let scored_count = scored_docs.len();

            let (results, skipped) = self.hydrate_from_scored_at(scored_docs, seq).await?;

            if results.len() >= target_k || scored_count < fetch_k {
                if skipped > 0 {
                    tracing::debug!(
                        skipped_tombstones = skipped,
                        target_k = target_k,
                        hydrated_count = results.len(),
                        "Tombstones skipped during vector search hydration"
                    );
                }
                let mut final_results = results;
                final_results.truncate(target_k);
                return Ok(final_results);
            }

            // Backfill: we need more candidates because tombstones reduced the valid result count below target_k.
            let next_fetch_k = (fetch_k * 2)
                .max(target_k + skipped * 2)
                .min(contextra_types::MAX_SEARCH_K);

            if next_fetch_k <= fetch_k {
                if skipped > 0 {
                    tracing::debug!(
                        skipped_tombstones = skipped,
                        target_k = target_k,
                        hydrated_count = results.len(),
                        "Tombstones skipped during vector search hydration (max search k reached)"
                    );
                }
                let mut final_results = results;
                final_results.truncate(target_k);
                return Ok(final_results);
            }

            fetch_k = next_fetch_k;
        }
    }
}
