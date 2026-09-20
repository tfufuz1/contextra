use crate::config::RouterConfig;
use memfuse_core::MemFuseError;
use memfuse_db::MemFuse;
use std::sync::Arc;

/// Holds strong Arc references to routing and calibration components to maintain live Weak references in `MemFuse`.
pub struct RoutingHandle {
    pub router: Arc<memfuse_router::DefaultRouterEngine>,
    pub calibrator: Arc<parking_lot::Mutex<memfuse_calibration::IsotonicCalibrator>>,
    pub pid_controller: Arc<parking_lot::Mutex<memfuse_calibration::PidController>>,
}

/// Conditionally sets up `RouterEngine`, `IsotonicCalibrator`, and `PidController` if routing profiles are configured.
/// Attaches their `Weak` pointers to `db` via `set_router`, `set_calibrator`, and `set_pid_controller`.
/// Returns `Some(RoutingHandle)` if profiles were present, or `None` if no profiles were configured.
pub async fn setup_routing(
    db: &Arc<MemFuse>,
    config: &RouterConfig,
) -> Result<Option<Arc<RoutingHandle>>, MemFuseError> {
    if config.profiles.is_empty() {
        return Ok(None);
    }

    let default_col = db.collection("default").await?;
    let router = Arc::new(memfuse_router::RouterEngine::new(
        default_col,
        config.profiles.clone(),
        config.calibration_store_path.clone(),
    ));

    let calibrator = Arc::new(parking_lot::Mutex::new(
        memfuse_calibration::IsotonicCalibrator::with_defaults(),
    ));

    let pid_controller = Arc::new(parking_lot::Mutex::new(
        memfuse_calibration::PidController::default(),
    ));

    let router_weak =
        Arc::downgrade(&router) as std::sync::Weak<dyn memfuse_db::DriftStatusProvider>;
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
