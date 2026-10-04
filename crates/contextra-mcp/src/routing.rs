use crate::config::RouterConfig;
use contextra::Contextra;
use contextra_types::{ContextChunk, ContextWindow, ContextraError, EntityId, TokenBudget};
use std::sync::Arc;

struct CollectionAdapter(Arc<contextra::Collection>);

impl contextra::router::ports_local::HybridSearchProvider for CollectionAdapter {
    fn search_hybrid<'a>(
        &'a self,
        query_text: &'a str,
        query_embedding: &'a [f32],
        top_k: usize,
    ) -> contextra_ports::BoxFuture<'a, Result<Vec<ContextChunk>, ContextraError>> {
        Box::pin(async move {
            let search_results = self
                .0
                .query()
                .text(query_text)
                .vector(query_embedding)
                .k(top_k)
                .execute()
                .await?;

            let mut chunks = Vec::with_capacity(search_results.len());
            for res in search_results {
                if let Ok(chunk) = ContextChunk::try_from(res) {
                    chunks.push(chunk);
                }
            }
            Ok(chunks)
        })
    }
}

/// Adapter resolving entity-to-community mappings.
///
/// Currently returns `Ok(None)` because neither the `Contextra` facade nor `Collection`
/// exposes a public entity-to-community lookup API (`get_community` / `get_communities_batch`).
struct CommunityAdapter;

impl contextra::router::ports_local::CommunityResolver for CommunityAdapter {
    fn get_community<'a>(
        &'a self,
        _entity_id: EntityId,
    ) -> contextra_ports::BoxFuture<'a, Result<Option<u64>, ContextraError>> {
        Box::pin(async move { Ok(None) })
    }
}

struct ContextPreparerAdapter;

impl contextra::router::ports_local::ContextPreparer for ContextPreparerAdapter {
    fn prepare_context(
        &self,
        chunks: Vec<ContextChunk>,
        budget: &TokenBudget,
        relevance_threshold: f32,
    ) -> Result<ContextWindow, ContextraError> {
        let initial_count = chunks.len();
        let usable_budget = budget.limit.saturating_sub(budget.reserved);

        let mut filtered_chunks: Vec<ContextChunk> = chunks
            .into_iter()
            .filter(|c| c.relevance >= relevance_threshold)
            .collect();

        // Stable sort descending by relevance, breaking ties deterministically by doc_id ascending.
        filtered_chunks.sort_by(|a, b| {
            b.relevance
                .partial_cmp(&a.relevance)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.doc_id.cmp(&b.doc_id))
        });

        let mut retained_chunks = Vec::new();
        let mut accumulated_tokens: usize = 0;

        for chunk in filtered_chunks {
            let next_total = accumulated_tokens.saturating_add(chunk.token_count);
            if next_total <= usable_budget {
                accumulated_tokens = next_total;
                retained_chunks.push(chunk);
            }
        }

        let truncated = retained_chunks.len() < initial_count;

        Ok(ContextWindow {
            chunks: retained_chunks,
            total_tokens: accumulated_tokens,
            truncated,
        })
    }
}

struct RouterDriftAdapter(Arc<contextra::router::DefaultRouterEngine>);

impl contextra_ports::DriftStatusProvider for RouterDriftAdapter {
    fn overall_drift_status(&self) -> String {
        self.0.overall_drift_status()
    }
}

/// Holds strong Arc references to routing and calibration components to maintain live Weak references in `Contextra`.
pub struct RoutingHandle {
    pub router: Arc<contextra::router::DefaultRouterEngine>,
    pub calibrator: Arc<parking_lot::Mutex<contextra_rank::IsotonicCalibrator>>,
    pub pid_controller: Arc<parking_lot::Mutex<contextra_adapt::PidController>>,
    pub _drift_adapter: Arc<dyn contextra_ports::DriftStatusProvider>,
}

pub async fn setup_routing(
    db: &Arc<Contextra>,
    config: &RouterConfig,
) -> Result<Option<Arc<RoutingHandle>>, ContextraError> {
    if config.profiles.is_empty() {
        return Ok(None);
    }

    let default_col = db.collection("default").await?;

    let search_provider = Arc::new(CollectionAdapter(default_col.clone()));
    let community_resolver = Arc::new(CommunityAdapter);
    let context_preparer = Arc::new(ContextPreparerAdapter);

    let router = Arc::new(contextra::router::RouterEngine::new(
        search_provider,
        community_resolver,
        context_preparer,
        config.profiles.clone(),
        config.calibration_store_path.clone(),
    ));

    let calibrator = Arc::new(parking_lot::Mutex::new(
        contextra_rank::IsotonicCalibrator::with_defaults(),
    ));

    let pid_controller = Arc::new(parking_lot::Mutex::new(
        contextra_adapt::PidController::default(),
    ));

    let drift_adapter: Arc<dyn contextra_ports::DriftStatusProvider> =
        Arc::new(RouterDriftAdapter(router.clone()));
    let router_weak = Arc::downgrade(&drift_adapter);
    db.set_router(router_weak);
    db.set_calibrator(Arc::downgrade(&calibrator));
    db.set_pid_controller(Arc::downgrade(&pid_controller));

    Ok(Some(Arc::new(RoutingHandle {
        router,
        calibrator,
        pid_controller,
        _drift_adapter: drift_adapter,
    })))
}

/// Conditionally sets up `KvBridgeAdapter` when feature `kv-bridge` is enabled.
#[cfg(feature = "kv-bridge")]
pub fn setup_kv_bridge(
    _db: &Arc<Contextra>,
) -> Option<Arc<contextra_infer_candle::KvBridgeAdapter>> {
    tracing::info!(
        "kv-bridge feature aktiv, aber keine Verschlüsselung konfiguriert — KvBridgeAdapter deaktiviert"
    );
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use contextra::router::ports_local::ContextPreparer;
    use contextra_types::DocId;

    fn make_chunk(doc_id: u64, relevance: f32, token_count: usize) -> ContextChunk {
        ContextChunk {
            doc_id: DocId::new(doc_id),
            content: format!("chunk_{doc_id}"),
            relevance,
            token_count,
            metadata: None,
            contextual_prefix: None,
            links: Vec::new(),
        }
    }

    #[test]
    fn test_relevance_threshold_removes_irrelevant_chunks() {
        let preparer = ContextPreparerAdapter;
        let budget = TokenBudget::new(1000, 0);
        let chunks = vec![
            make_chunk(1, 0.8, 100),
            make_chunk(2, 0.3, 100),
            make_chunk(3, 0.6, 100),
        ];

        let window = preparer
            .prepare_context(chunks, &budget, 0.5)
            .expect("prepare_context should succeed");

        assert_eq!(window.chunks.len(), 2);
        assert_eq!(window.chunks[0].doc_id, DocId::new(1));
        assert_eq!(window.chunks[1].doc_id, DocId::new(3));
        assert_eq!(window.total_tokens, 200);
        assert!(window.truncated);
    }

    #[test]
    fn test_budget_truncates_list_and_sets_truncated_flag() {
        let preparer = ContextPreparerAdapter;
        let budget = TokenBudget::new(250, 50); // Usable: 200 tokens
        let chunks = vec![
            make_chunk(1, 0.9, 100),
            make_chunk(2, 0.8, 100),
            make_chunk(3, 0.7, 100),
        ];

        let window = preparer
            .prepare_context(chunks, &budget, 0.0)
            .expect("prepare_context should succeed");

        assert_eq!(window.chunks.len(), 2);
        assert_eq!(window.chunks[0].doc_id, DocId::new(1));
        assert_eq!(window.chunks[1].doc_id, DocId::new(2));
        assert_eq!(window.total_tokens, 200);
        assert!(window.truncated);
    }

    #[test]
    fn test_empty_input_returns_empty_window() {
        let preparer = ContextPreparerAdapter;
        let budget = TokenBudget::new(1000, 100);
        let window = preparer
            .prepare_context(Vec::new(), &budget, 0.5)
            .expect("prepare_context should succeed");

        assert!(window.chunks.is_empty());
        assert_eq!(window.total_tokens, 0);
        assert!(!window.truncated);
    }

    #[test]
    fn test_relevance_tie_breaking_is_deterministic() {
        let preparer = ContextPreparerAdapter;
        let budget = TokenBudget::new(1000, 0);
        let chunks = vec![
            make_chunk(3, 0.8, 100),
            make_chunk(1, 0.8, 100),
            make_chunk(2, 0.8, 100),
        ];

        let window = preparer
            .prepare_context(chunks, &budget, 0.0)
            .expect("prepare_context should succeed");

        assert_eq!(window.chunks.len(), 3);
        assert_eq!(window.chunks[0].doc_id, DocId::new(1));
        assert_eq!(window.chunks[1].doc_id, DocId::new(2));
        assert_eq!(window.chunks[2].doc_id, DocId::new(3));
        assert!(!window.truncated);
    }

    #[test]
    fn test_reserved_greater_than_limit_does_not_panic() {
        let preparer = ContextPreparerAdapter;
        let budget = TokenBudget::new(100, 200); // Reserved > Limit => Usable = 0
        let chunks = vec![make_chunk(1, 0.9, 10)];

        let window = preparer
            .prepare_context(chunks, &budget, 0.0)
            .expect("prepare_context should succeed");

        assert!(window.chunks.is_empty());
        assert_eq!(window.total_tokens, 0);
        assert!(window.truncated);
    }
}
