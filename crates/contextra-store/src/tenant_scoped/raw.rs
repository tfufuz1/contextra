//! Rohe Speicherzugriffe unter Kapselung (SPEC §5.2 AR-6, T-ARCH-2).

use crate::lsm::LsmStorage;
use contextra_types::tags::TenantTag;
use std::sync::Arc;

/// Kapselt den ungeschützten Zugriff ausschließlich für interne Wiederherstellungspfade.
pub struct RawStorageHandle {
    inner: Arc<LsmStorage>,
}

impl RawStorageHandle {
    /// Erzeugt ein neues Rohes Handle.
    pub fn new(inner: Arc<LsmStorage>) -> Self {
        Self { inner }
    }

    /// Liefert eine Referenz auf das innere LSM-Speichertriebwerk.
    pub fn as_inner(&self) -> &Arc<LsmStorage> {
        &self.inner
    }

    /// Bindet das Rohe Handle an einen spezifischen Mandanten (AR-6).
    pub fn into_scoped(self, tenant: TenantTag) -> super::scoped::TenantScopedStorage {
        super::scoped::TenantScopedStorage::new(tenant, self.inner)
    }
}
