// FILE-CONTEXT
// STAND: 2026-10-14
// ZWECK: Anti-Resurrection Restore-Guard (`StreamingRestoreGuard`) für Checkpoint-Wiederherstellung (§5.9, §5.12, CRASH-08).
// INVARIANTEN: Zero-Panic, Fail-Closed bei widerrufenen Keys/Gruppen, Integritätsfehlern oder ErasureLedger-Einträgen.

#![forbid(unsafe_code)]

use contextra_crypto::kv_shredding::KeyRegistry;
use contextra_crypto::revocation_log::{RevocationLog, RevocationTarget};
use contextra_ports::{RestoreGuard, RestoreVerdict};
use contextra_types::TxId;
use crate::erasure_ledger::{ErasureLedger, ErasureLedgerHandle, LedgerDecision};
use std::sync::Arc;

/// Streamender Anti-Resurrection Guard zur Verhinderung der Wiederbelebung gelöschter Schlüssel
/// bei Checkpoint-Restores.
///
/// Evaluiert Schlüssel einzeln ("on-demand" / streaming) gegen den [`ErasureLedger`]
/// oder [`ErasureLedgerHandle`], den [`RevocationLog`] und die [`KeyRegistry`].
pub struct StreamingRestoreGuard {
    ledger: Option<Arc<ErasureLedger>>,
    ledger_handle: Option<Arc<ErasureLedgerHandle>>,
    revocation_log: Option<Arc<RevocationLog>>,
    key_registry: Option<Arc<KeyRegistry>>,
    storage_scope: Vec<u8>,
}

impl std::fmt::Debug for StreamingRestoreGuard {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StreamingRestoreGuard")
            .field("storage_scope", &self.storage_scope)
            .field("has_ledger", &self.ledger.is_some())
            .field("has_ledger_handle", &self.ledger_handle.is_some())
            .field("has_revocation_log", &self.revocation_log.is_some())
            .field("has_key_registry", &self.key_registry.is_some())
            .finish()
    }
}

impl StreamingRestoreGuard {
    /// Erzeugt einen neuen `StreamingRestoreGuard` mit Ledger und optionalen Revocation-Quellen.
    #[must_use]
    pub fn new(
        ledger: Arc<ErasureLedger>,
        revocation_log: Option<Arc<RevocationLog>>,
        key_registry: Option<Arc<KeyRegistry>>,
        storage_scope: impl Into<Vec<u8>>,
    ) -> Self {
        Self {
            ledger: Some(ledger),
            ledger_handle: None,
            revocation_log,
            key_registry,
            storage_scope: storage_scope.into(),
        }
    }

    /// Erzeugt einen neuen `StreamingRestoreGuard` mit `ErasureLedgerHandle` und optionalen Revocation-Quellen.
    #[must_use]
    pub fn new_with_handle(
        ledger_handle: Arc<ErasureLedgerHandle>,
        revocation_log: Option<Arc<RevocationLog>>,
        key_registry: Option<Arc<KeyRegistry>>,
        storage_scope: impl Into<Vec<u8>>,
    ) -> Self {
        Self {
            ledger: None,
            ledger_handle: Some(ledger_handle),
            revocation_log,
            key_registry,
            storage_scope: storage_scope.into(),
        }
    }

    /// Erzeugt einen Guard für den LSM Storage Scope `b"lsm"`.
    #[must_use]
    pub fn for_lsm(ledger: Arc<ErasureLedger>) -> Self {
        Self::new(ledger, None, None, b"lsm".to_vec())
    }

    /// Erzeugt einen Guard mit `ErasureLedgerHandle` für den LSM Storage Scope `b"lsm"`.
    #[must_use]
    pub fn for_lsm_handle(ledger_handle: Arc<ErasureLedgerHandle>) -> Self {
        Self::new_with_handle(ledger_handle, None, None, b"lsm".to_vec())
    }

    /// Fügt einen optionalen `RevocationLog` hinzu.
    #[must_use]
    pub fn with_revocation_log(mut self, log: Arc<RevocationLog>) -> Self {
        self.revocation_log = Some(log);
        self
    }

    /// Fügt eine optionale `KeyRegistry` hinzu.
    #[must_use]
    pub fn with_key_registry(mut self, registry: Arc<KeyRegistry>) -> Self {
        self.key_registry = Some(registry);
        self
    }

    /// Extrahiert eine 64-Bit Group-ID aus dem Schlüssel.
    fn extract_group_id(key: &[u8]) -> Option<u64> {
        if key.starts_with(b"group:") {
            let s = std::str::from_utf8(&key[6..]).ok()?;
            let num_str = s.split(':').next()?;
            num_str.parse::<u64>().ok()
        } else if key.len() >= 8 {
            let mut arr = [0u8; 8];
            arr.copy_from_slice(&key[..8]);
            Some(u64::from_be_bytes(arr))
        } else {
            None
        }
    }
}

impl RestoreGuard for StreamingRestoreGuard {
    fn admit(&self, key: &[u8], source_tx: TxId) -> RestoreVerdict {
        let cp_tx = source_tx.inner();

        // 1. Fail-closed RevocationLog Prüfungen
        if let Some(ref log) = self.revocation_log {
            if log.verify_integrity().is_err() {
                return RestoreVerdict::Reject;
            }
            if let Some(group_id) = Self::extract_group_id(key) {
                if log.is_revoked(&RevocationTarget::Group(group_id)) {
                    return RestoreVerdict::Reject;
                }
            }
            if let Ok(key_str) = std::str::from_utf8(key) {
                if log.is_revoked(&RevocationTarget::Record(key_str.to_string())) {
                    return RestoreVerdict::Reject;
                }
            }
        }

        // 2. KeyRegistry Gruppenprüfung
        if let Some(ref registry) = self.key_registry {
            if let Some(group_id) = Self::extract_group_id(key) {
                if registry.is_group_revoked(group_id) {
                    return RestoreVerdict::Reject;
                }
            }
        }

        // 3. ErasureLedger Entscheidung
        // Abgewiesen, wenn ein Ledger-Eintrag den Key oder ein Präfix deckt und erased_at_tx >= cp_tx
        if let Some(ref ledger) = self.ledger {
            if ledger.decide(&self.storage_scope, key, cp_tx) == LedgerDecision::Drop {
                return RestoreVerdict::Reject;
            }
        } else if let Some(ref handle) = self.ledger_handle {
            if handle.decide(&self.storage_scope, key, cp_tx) == LedgerDecision::Drop {
                return RestoreVerdict::Reject;
            }
        }

        RestoreVerdict::Allow
    }
}
