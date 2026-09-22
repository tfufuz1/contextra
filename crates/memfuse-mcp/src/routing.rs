use crate::config::RouterConfig;
use memfuse::MemFuse;
use memfuse_core::MemFuseError;
use std::sync::Arc;

/// Holds strong Arc references to routing and calibration components to maintain live Weak references in `MemFuse`.
pub struct RoutingHandle {
    pub router: Arc<memfuse::router::DefaultRouterEngine>,
    pub calibrator: Arc<parking_lot::Mutex<memfuse_rank::IsotonicCalibrator>>,
    pub pid_controller: Arc<parking_lot::Mutex<memfuse_adapt::PidController>>,
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
    let search_provider = Arc::new(CollectionSearchAdapter(default_col));
    let community_resolver = Arc::new(memfuse::router::ports_local::NoopCommunityResolver);
    let context_preparer = Arc::new(memfuse::router::ports_local::PassthroughContextPreparer);

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

    let router_weak = Arc::downgrade(&router) as std::sync::Weak<dyn memfuse_core::DriftStatusProvider>;
    db.set_router(router_weak);
    db.set_calibrator(Arc::downgrade(&calibrator));
    db.set_pid_controller(Arc::downgrade(&pid_controller));

    Ok(Some(Arc::new(RoutingHandle {
        router,
        calibrator,
        pid_controller,
    })))
}

/// Conditionally sets up `KvBridgeAdapter` when feature `kv-bridge` is enabled.
#[cfg(feature = "kv-bridge")]
pub fn setup_kv_bridge(_db: &Arc<MemFuse>) -> Option<Arc<memfuse_infer_candle::KvBridgeAdapter>> {
    // AI-TAG[SMELL][RESOLVED] audit-kv-bridge: Cipher-Integration wenn MemFuse::kv_cipher() API existiert
    tracing::info!(
        "kv-bridge feature aktiv, aber keine Verschlüsselung konfiguriert — KvBridgeAdapter deaktiviert"
    );
    None
}
