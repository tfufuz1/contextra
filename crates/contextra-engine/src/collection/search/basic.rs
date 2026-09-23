// FILE-CONTEXT
// ZWECK: Einfache Suchvarianten (search, search_with_filter, search_with_filter_expr, search_text) für Collection.

use super::checkpoint::with_pinned_checkpoint_at_latest;
use super::Collection;
#[allow(deprecated)]
use crate::filter::MetadataFilter;
use contextra_core::{DocId, FilterExpr, Result, StorageEngine, VectorIndex};

impl<S: StorageEngine, V: VectorIndex> Collection<S, V> {
    /// Performs semantic k-NN search over stored embeddings.
    #[deprecated(since = "0.1.0", note = "use Collection::query() instead")]
    #[allow(deprecated)]
    #[tracing::instrument(level = "trace", skip(self, query_embedding))]
    pub async fn search(
        &self,
        query_embedding: &[f32],
        k: usize,
    ) -> Result<Vec<crate::SearchResult>> {
        if k == 0 {
            return Err(contextra_core::ContextraError::invalid_input(
                "Search k must be greater than 0",
            ));
        }
        let k = k.min(contextra_core::MAX_SEARCH_K);
        self.search_with_filter_expr(query_embedding, k, None).await
    }

    /// Performs semantic search with an advanced metadata filter.
    #[deprecated(since = "0.1.0", note = "use Collection::query() instead")]
    #[allow(deprecated)]
    #[tracing::instrument(level = "trace", skip(self, query, filter))]
    pub async fn search_with_filter(
        &self,
        query: &[f32],
        k: usize,
        filter: Option<MetadataFilter>,
    ) -> Result<Vec<crate::SearchResult>> {
        let expr = match filter {
            Some(f) => Some(FilterExpr::try_from(f)?),
            None => None,
        };
        self.search_with_filter_expr(query, k, expr).await
    }

    /// Performs semantic search with an advanced metadata filter expression (`FilterExpr`).
    #[deprecated(since = "0.1.0", note = "use Collection::query() instead")]
    #[allow(deprecated)]
    #[tracing::instrument(level = "trace", skip(self, query, filter))]
    pub async fn search_with_filter_expr(
        &self,
        query: &[f32],
        k: usize,
        filter: Option<FilterExpr>,
    ) -> Result<Vec<crate::SearchResult>> {
        if k == 0 {
            return Err(contextra_core::ContextraError::invalid_input(
                "Search k must be greater than 0",
            ));
        }
        if query.len() != self.dimension {
            return Err(contextra_core::ContextraError::invalid_input(format!(
                "Dimension mismatch: expected {}, got {}",
                self.dimension,
                query.len()
            )));
        }
        let k = k.min(contextra_core::MAX_SEARCH_K);
        // 🛡️ SICHERUNG: Snapshot-Isolation (FIND-DB-003)
        // Pin-first, read-after: the seq is read under the protection of the pin,
        // eliminating the TOCTOU window between snapshot_seq() and pin activation.
        with_pinned_checkpoint_at_latest(self.storage.as_ref(), |seq| async move {
            let filter = match filter {
                Some(f) => f,
                None => return self.search_filtered_at(query, k, None, seq).await,
            };

            let matched_ids = self.get_matching_doc_ids_at(&filter, seq).await?;

            // If no docs match the filter, return early
            if matched_ids.is_empty() {
                return Ok(Vec::new());
            }

            let filter_fn = move |id: DocId| matched_ids.contains(&id);
            self.search_filtered_at(query, k, Some(&filter_fn), seq)
                .await
        })
        .await
    }

    /// Performs semantic search using a raw text query (automatically embedded).
    #[deprecated(since = "0.1.0", note = "use Collection::query() instead")]
    #[allow(deprecated)]
    #[tracing::instrument(level = "trace", skip(self, query_text))]
    pub async fn search_text(
        &self,
        query_text: &str,
        k: usize,
    ) -> Result<Vec<crate::SearchResult>> {
        let k = k.min(contextra_core::MAX_SEARCH_K);
        let embedding = {
            let embedder = {
                let guard = self.embedder.read();
                guard
                    .as_ref()
                    .ok_or_else(|| {
                        contextra_core::ContextraError::Internal(
                            "No embedder configured for this collection".into(),
                        )
                    })?
                    .clone()
            };
            embedder.embed(query_text).await?
        };
        self.search(&embedding, k).await
    }
}
