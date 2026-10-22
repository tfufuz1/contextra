use contextra::Contextra;
use std::sync::Arc;

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
