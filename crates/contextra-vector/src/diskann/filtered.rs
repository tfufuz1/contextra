// FILE-CONTEXT
// ZWECK: DiskANN-gefilterte Vektorsuche via adaptivem Oversampling mit Post-Filter (Spec §7.3 / ADR-013).
// INVARIANTEN: MAX_SEARCH_K Schranke; deterministische Ordnung (Score absteigend, DocId aufsteigend); kein unbegrenzter Loop.

use super::types::DiskAnnIndex;
use contextra_core::{ContextraError, DocId, Result, ScoredDocument};

impl DiskAnnIndex {
    /// Executes a predicate-filtered vector search using adaptive oversampling with post-filtering.
    ///
    /// # Oversampling Strategy
    /// Starts by fetching `fetch_k = (k * 4).min(MAX_SEARCH_K)` raw candidates via `search_internal`.
    /// Predicates are evaluated synchronously. If fewer than `k` matching documents are found,
    /// `fetch_k` is doubled up to `MAX_SEARCH_K`.
    ///
    /// # Limitations
    /// - Non-prefix-stable ANN search: A larger `fetch_k` does not strictly guarantee a superset of candidates.
    /// - Snapshot search (`search_at` / `snapshot_read_at`) is **not supported** by DiskANN because DiskANN
    ///   does not maintain an append-only sequence log; calling `search_at` will return
    ///   [`ContextraError::CapabilityUnsupported`][contextra_core::ContextraError::CapabilityUnsupported].
    pub(crate) async fn search_filtered_internal(
        &self,
        query: &[f32],
        k: usize,
        filter: Option<&(dyn Fn(DocId) -> bool + Send + Sync)>,
    ) -> Result<Vec<ScoredDocument>> {
        if k == 0 || k > contextra_core::MAX_SEARCH_K {
            return Err(ContextraError::invalid_input(format!(
                "Requested k ({k}) is invalid or exceeds maximum allowed search limit ({})",
                contextra_core::MAX_SEARCH_K
            )));
        }

        let Some(f) = filter else {
            return self.search_internal(query, k).await;
        };

        let mut fetch_k = (k * 4).min(contextra_core::MAX_SEARCH_K);
        let mut filtered_results = Vec::new();

        for _ in 0..10 {
            let raw = self.search_internal(query, fetch_k).await?;
            let raw_len = raw.len();

            filtered_results = raw
                .into_iter()
                .filter(|doc| f(doc.doc_id))
                .collect::<Vec<_>>();

            filtered_results.sort_by(|a, b| {
                b.score
                    .total_cmp(&a.score)
                    .then_with(|| a.doc_id.cmp(&b.doc_id))
            });

            if filtered_results.len() >= k
                || raw_len < fetch_k
                || fetch_k >= contextra_core::MAX_SEARCH_K
            {
                break;
            }

            let next_fetch_k = (fetch_k * 2).min(contextra_core::MAX_SEARCH_K);
            if next_fetch_k == fetch_k {
                break;
            }
            fetch_k = next_fetch_k;
        }

        filtered_results.truncate(k);
        Ok(filtered_results)
    }
}
