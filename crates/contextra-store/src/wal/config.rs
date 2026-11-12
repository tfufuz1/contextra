//! Konfiguration des Write-Ahead-Logs.

use super::flusher::WalFlusherConfig;
use super::encode::WalVersion;
use contextra_crypto::crypto::KeyManager;
use std::sync::Arc;

#[derive(Clone)]
// ANCHOR[contextra-store:WAL-CORE-101]
/// Konfigurationsoptionen für das Write-Ahead-Log (WAL).
///
/// Steuert Schlüsselmanager, Integritätsschlüssel-Rückfallebenen, Mindestversion
/// und Flusher-Einstellungen für asynchrone Schreibvorgänge.
pub struct WalConfig {
    pub key_manager: Option<Arc<KeyManager>>,
    pub(crate) allow_legacy_integrity_key_fallback: bool,
    pub min_wal_version: WalVersion,
    pub flusher_config: WalFlusherConfig,
    pub id_gen: Option<Arc<dyn contextra_env::IdGen>>,
}

impl std::fmt::Debug for WalConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WalConfig")
            .field("key_manager", &self.key_manager)
            .field(
                "allow_legacy_integrity_key_fallback",
                &self.allow_legacy_integrity_key_fallback,
            )
            .field("min_wal_version", &self.min_wal_version)
            .field("flusher_config", &self.flusher_config)
            .field("id_gen", &self.id_gen.as_ref().map(|_| "Arc<dyn IdGen>"))
            .finish()
    }
}

impl Default for WalConfig {
    fn default() -> Self {
        Self {
            key_manager: None,
            allow_legacy_integrity_key_fallback: false,
            min_wal_version: WalVersion::V3,
            flusher_config: WalFlusherConfig::default(),
            id_gen: None,
        }
    }
}

impl WalConfig {
    pub fn with_legacy_fallback(mut self, allow: bool) -> Self {
        self.allow_legacy_integrity_key_fallback = allow;
        self
    }

    pub fn with_id_gen(mut self, id_gen: Arc<dyn contextra_env::IdGen>) -> Self {
        self.id_gen = Some(id_gen);
        self
    }
}
