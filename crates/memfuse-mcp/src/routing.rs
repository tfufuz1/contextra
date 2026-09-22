use crate::config::RouterConfig;
use memfuse::MemFuse;
use memfuse_core::{ContextChunk, ContextWindow, EntityId, MemFuseError, TokenBudget};
use std::sync::Arc;

struct CollectionAdapter(Arc<memfuse::Collection>);

impl memfuse::router::ports_local::HybridSearchProvider for CollectionAdapter {
    fn search_hybrid<'a>(
        &'a self,
        query_text: &'a str,
        query_embedding: &'a [f32],
        top_k: usize,
    ) -> memfuse_core::BoxFuture<'a, Result<Vec<ContextChunk>, MemFuseError>> {
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

struct CommunityAdapter;

impl memfuse::router::ports_local::CommunityResolver for CommunityAdapter {
    fn get_community<'a>(
        &'a self,
        _entity_id: EntityId,
    ) -> memfuse_core::BoxFuture<'a, Result<Option<u64>, MemFuseError>> {
        Box::pin(async move { Ok(None) })
    }
}

struct ContextPreparerAdapter;

impl memfuse::router::ports_local::ContextPreparer for ContextPreparerAdapter {
    fn prepare_context(
        &self,
        chunks: Vec<ContextChunk>,
        budget: &TokenBudget,
        _relevance_threshold: f32,
    ) -> Result<ContextWindow, MemFuseError> {
        let total_tokens = chunks.iter().map(|c| c.token_count).sum();
        let limit = budget.limit;
        let truncated = total_tokens > limit;
        Ok(ContextWindow {
            chunks,
            total_tokens,
            truncated,
        })
    }
}

struct RouterDriftAdapter(Arc<memfuse::router::DefaultRouterEngine>);

impl memfuse_core::DriftStatusProvider for RouterDriftAdapter {
    fn overall_drift_status(&self) -> String {
        self.0.overall_drift_status()
    }
}

/// Holds strong Arc references to routing and calibration components to maintain live Weak references in `MemFuse`.
pub struct RoutingHandle {
    pub router: Arc<memfuse::router::DefaultRouterEngine>,
    pub calibrator: Arc<parking_lot::Mutex<memfuse::calibration::IsotonicCalibrator>>,
    pub pid_controller: Arc<parking_lot::Mutex<memfuse_adapt::PidController>>,
    pub _drift_adapter: Arc<dyn memfuse_core::DriftStatusProvider>,
}

/// Conditionally sets up `RouterEngine`, `IsotonicCalibrator`, and `PidController` if routing profiles are configured.
/// Attaches their `Weak` pointers to `db` via `set_router`, `set_calibrator`, and `set_pid_controller`.
/// Returns `Some(RoutingHandle)` if profiles were present, or `None` if no profiles were configured.
struct CollectionSearchAdapter(Arc<memfuse::Collection>);

impl memfuse::router::ports_local::HybridSearchProvider for CollectionSearchAdapter {
    fn search_hybrid<'a>(
        &'a self,
        query_text: &'a str,
        query_embedding: &'a [f32],
        top_k: usize,
    ) -> memfuse_core::BoxFuture<'a, Result<Vec<memfuse_core::ContextChunk>, MemFuseError>> {
        let col = self.0.clone();
        Box::pin(async move {
            let search_results = col
                .query()
                .text(query_text)
                .vector(query_embedding)
                .k(top_k)
                .execute()
                .await?;
            search_results
                .into_iter()
                .map(memfuse_core::ContextChunk::try_from)
                .collect()
        })
    }
}

pub async fn setup_routing(
    db: &Arc<MemFuse>,
    config: &RouterConfig,
) -> Result<Option<Arc<RoutingHandle>>, MemFuseError> {
    if config.profiles.is_empty() {
        return Ok(None);
    }

    let default_col = db.collection("default").await?;

    let search_provider = Arc::new(CollectionAdapter(default_col.clone()));
    let community_resolver = Arc::new(CommunityAdapter);
    let context_preparer = Arc::new(ContextPreparerAdapter);

    let router = Arc::new(memfuse::router::RouterEngine::new(
        search_provider,
        community_resolver,
        context_preparer,
        config.profiles.clone(),
        config.calibration_store_path.clone(),
    ));

    let calibrator = Arc::new(parking_lot::Mutex::new(
        memfuse_rank::IsotonicCalibrator::with_defaults(),
    ));

    let pid_controller = Arc::new(parking_lot::Mutex::new(
        memfuse_adapt::PidController::default(),
    ));

    let drift_adapter: Arc<dyn memfuse_core::DriftStatusProvider> =
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
pub fn setup_kv_bridge(_db: &Arc<MemFuse>) -> Option<Arc<memfuse_infer_candle::KvBridgeAdapter>> {
    tracing::info!(
        "kv-bridge feature aktiv, aber keine Verschlüsselung konfiguriert — KvBridgeAdapter deaktiviert"
    );
    None
}
