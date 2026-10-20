// FILE-CONTEXT
// STAND: 2026-10-14
// ZWECK: Restore-Filter Adapter (`DpiRestoreFilter`) fuer Anti-Resurrection (§C4.8, WP-1.1, Task B10).
// INVARIANTEN: Zero-Panic, Fail-Closed bei unlesbarem/defektem RevocationLog oder widerrufener Shredding-Gruppe / Key, sowie bei ErasureLedger-Löschentscheidungen.

#![forbid(unsafe_code)]

use contextra_checkpoint::restore_filter::CpyRestoreFilter;
use contextra_crypto::kv_shredding::KeyRegistry;
use contextra_crypto::revocation_log::{RevocationLog, RevocationTarget};
use contextra_store::erasure_ledger::{ErasureLedger, ErasureLedgerHandle, LedgerDecision};
use std::sync::Arc;

/// Restore filter adapter implementing [`CpyRestoreFilter`] backed by [`RevocationLog`], [`KeyRegistry`] and [`ErasureLedger`].
pub struct DpiRestoreFilter {
    ledger: Option<Arc<ErasureLedger>>,
    ledger_handle: Option<Arc<ErasureLedgerHandle>>,
    revocation_log: Option<Arc<RevocationLog>>,
    key_registry: Option<Arc<KeyRegistry>>,
    storage_scope: Vec<u8>,
}

impl DpiRestoreFilter {
    /// Creates a new `DpiRestoreFilter` wrapping `ledger` and optional revocation sources for `storage_scope`.
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

    /// Creates a new `DpiRestoreFilter` wrapping `ledger_handle` and optional revocation sources for `storage_scope`.
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

    /// Creates a `DpiRestoreFilter` using explicit runtime registries for LSM scope `b"lsm"`.
    #[must_use]
    pub fn from_runtime(
        ledger: Arc<ErasureLedger>,
        revocation_log: Option<Arc<RevocationLog>>,
        key_registry: Option<Arc<KeyRegistry>>,
    ) -> Self {
        Self::new(ledger, revocation_log, key_registry, b"lsm".to_vec())
    }

    /// Creates a default `DpiRestoreFilter` for LSM storage scope `b"lsm"` with ledger only.
    #[must_use]
    pub fn for_lsm(ledger: Arc<ErasureLedger>) -> Self {
        Self::new(ledger, None, None, b"lsm".to_vec())
    }

    /// Creates a default `DpiRestoreFilter` for LSM storage scope `b"lsm"` with `ErasureLedgerHandle`.
    #[must_use]
    pub fn for_lsm_handle(ledger_handle: Arc<ErasureLedgerHandle>) -> Self {
        Self::new_with_handle(ledger_handle, None, None, b"lsm".to_vec())
    }

    /// Attaches an optional `RevocationLog`.
    #[must_use]
    pub fn with_revocation_log(mut self, log: Arc<RevocationLog>) -> Self {
        self.revocation_log = Some(log);
        self
    }

    /// Attaches an optional `KeyRegistry`.
    #[must_use]
    pub fn with_key_registry(mut self, registry: Arc<KeyRegistry>) -> Self {
        self.key_registry = Some(registry);
        self
    }

    /// Helper to extract group ID from key if key is encoded as `group:<u64>` or `group:<u64>:...` or binary u64 prefix.
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

impl CpyRestoreFilter for DpiRestoreFilter {
    fn drop_record(&self, key: &[u8], tx: u64) -> bool {
        // 1. Fail-closed RevocationLog verification & query
        if let Some(ref log) = self.revocation_log {
            if log.verify_integrity().is_err() {
                return true;
            }
            if let Some(group_id) = Self::extract_group_id(key) {
                if log.is_revoked(&RevocationTarget::Group(group_id)) {
                    return true;
                }
            }
            if let Ok(key_str) = std::str::from_utf8(key) {
                if log.is_revoked(&RevocationTarget::Record(key_str.to_string())) {
                    return true;
                }
            }
        }

        // 2. KeyRegistry query if configured
        if let Some(ref registry) = self.key_registry {
            if let Some(group_id) = Self::extract_group_id(key) {
                if registry.is_group_revoked(group_id) {
                    return true;
                }
            }
        }

        // 3. ErasureLedger decision query
        if let Some(ref ledger) = self.ledger {
            ledger.decide(&self.storage_scope, key, tx) == LedgerDecision::Drop
        } else if let Some(ref handle) = self.ledger_handle {
            handle.decide(&self.storage_scope, key, tx) == LedgerDecision::Drop
        } else {
            false
        }
    }
}
