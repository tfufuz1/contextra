//! Mandantenspezifische Speicherisolation und Schlüsselgrenzen (SPEC §5.2 AR-6, T-ARCH-2).

use contextra_ports::keys::FileId;
use contextra_ports::BoxFuture;
use contextra_types::key::{KeyBuf, Matcher};
use contextra_types::tags::TenantTag;
use contextra_types::{ContextraError, Result};
use std::sync::Arc;

use crate::health::reserve::DiskReserve;
use crate::lsm::LsmStorage;
use crate::purge::backend::{DrainStats, PurgePrimitives, RewriteOutcome, SealPoint, WalFloor};

/// Gekapselter Zugriff auf den LSM-Speicher für einen spezifischen Mandanten (AR-6).
pub struct TenantScopedStorage {
    tenant: TenantTag,
    inner: Arc<LsmStorage>,
}

impl TenantScopedStorage {
    /// Erstellt eine neue mandantenisolierte Speicherinstanz.
    pub fn new(tenant: TenantTag, inner: Arc<LsmStorage>) -> Self {
        Self { tenant, inner }
    }

    /// Liefert den zugeordneten Mandanten.
    pub fn tenant(&self) -> TenantTag {
        self.tenant
    }

    /// Validiert, dass ein Schlüssel zum Mandanten gehört.
    pub fn validate_key(&self, key: &KeyBuf) -> Result<()> {
        let key_bytes = key.as_bytes();
        if key_bytes.len() >= 17 && key_bytes[1..17] == *self.tenant.as_bytes() {
            Ok(())
        } else {
            Err(ContextraError::Storage(format!(
                "Key tenant does not match scoped tenant {:?}",
                self.tenant
            )))
        }
    }
}

impl PurgePrimitives for TenantScopedStorage {
    fn drain_range<'a>(
        &'a self,
        matcher: &'a Matcher,
    ) -> BoxFuture<'a, Result<DrainStats>> {
        PurgePrimitives::drain_range(&*self.inner, matcher)
    }

    fn seal_wal_at<'a>(&'a self) -> BoxFuture<'a, Result<SealPoint>> {
        PurgePrimitives::seal_wal_at(&*self.inner)
    }

    fn drop_wal_until<'a>(
        &'a self,
        f_seq: u64,
    ) -> BoxFuture<'a, Result<WalFloor>> {
        PurgePrimitives::drop_wal_until(&*self.inner, f_seq)
    }

    fn rewrite_without<'a>(
        &'a self,
        file_id: FileId,
        matcher: &'a Matcher,
        reserve: DiskReserve,
    ) -> BoxFuture<'a, Result<RewriteOutcome>> {
        PurgePrimitives::rewrite_without(&*self.inner, file_id, matcher, reserve)
    }

    fn unlink_verified<'a>(
        &'a self,
        file_id: FileId,
    ) -> BoxFuture<'a, Result<()>> {
        PurgePrimitives::unlink_verified(&*self.inner, file_id)
    }
}
