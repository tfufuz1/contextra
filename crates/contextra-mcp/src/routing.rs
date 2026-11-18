// =========================================================================
// SECTION: routing
// =========================================================================
// ANCHOR[contextra-mcp:ROUTING-001]
use crate::config::RouterConfig;
use contextra::Contextra;
use contextra_types::ContextraError;
use std::sync::Arc;

/// Holds strong Arc references to routing and calibration components.
pub struct RoutingHandle {
    pub calibrator: Arc<parking_lot::Mutex<contextra_rank::IsotonicCalibrator>>,
}

pub async fn setup_routing(
    _db: &Arc<Contextra>,
    _config: &RouterConfig,
) -> Result<Option<Arc<RoutingHandle>>, ContextraError> {
    // Router-Funktionalität entfernt — Routing ist deaktiviert.
    Ok(None)
}

// !SECTION: routing
