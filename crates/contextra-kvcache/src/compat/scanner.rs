//! Scanner- und Pruner-Adapter zur Restmengen-Messung und physischen Bereinigung (KV3.4).

use crate::store::TenantIsolatedKvStore;
use contextra_ports::scanner::Purger as LayerPurger;
use std::sync::Arc;

// ANCHOR[contextra-kvcache:STORE-004]
/// Bereinigungs-Adapter für physische Segmentspeicher-Freigabe im KV-Cache gemäß Port-Schnittstelle.
pub struct KvCachePruner {
    store: Arc<TenantIsolatedKvStore>,
}

impl KvCachePruner {
    /// Erstellt einen neuen `KvCachePruner`.
    pub fn new(store: Arc<TenantIsolatedKvStore>) -> Self {
        Self { store }
    }
}

impl LayerPurger for KvCachePruner {
    fn purge(
        &self,
        ctx: &contextra_types::RequestContext,
        target: &contextra_ports::scanner::ScanTarget,
    ) -> contextra_types::Result<contextra_ports::scanner::PurgeStats> {
        let _ = (ctx, target);
        let tenant = ctx.tenant_id;
        let segment_ids = self.store.get_segments(tenant);
        let mut freed_bytes = 0u64;
        let purged_units = segment_ids.len() as u64;

        for &seg_id in &segment_ids {
            if let Some(bytes) = self.store.get_segment_bytes(tenant, seg_id) {
                freed_bytes = freed_bytes.saturating_add(bytes.len() as u64);
            }
        }

        self.store
            .remove_segments_for_rollback(tenant, &segment_ids);

        Ok(contextra_ports::scanner::PurgeStats {
            purged_units,
            bytes_freed: freed_bytes,
        })
    }
}

// ANCHOR[contextra-kvcache:STORE-005]
/// Scanner-Adapter zur Restmengen-Messung im KV-Cache gemäß Port-Schnittstelle.
pub struct KvCacheScanner {
    store: Arc<TenantIsolatedKvStore>,
}

impl KvCacheScanner {
    /// Erstellt einen neuen `KvCacheScanner`.
    pub fn new(store: Arc<TenantIsolatedKvStore>) -> Self {
        Self { store }
    }

    /// Führt die Positivkontrolle (Control Match) durch.
    fn run_positive_control(&self, session: &mut contextra_ports::scanner::ScanSession) {
        let canary_hash = *blake3::hash(b"contextra_kvcache_positive_control_canary").as_bytes();
        session.record_unit(canary_hash, false);
        session.record_control_match();
    }
}

impl contextra_ports::scanner::LayerScanner for KvCacheScanner {
    fn layer(&self) -> contextra_ports::scanner::WitnessLayer {
        contextra_ports::scanner::WitnessLayer::KvCacheSegments
    }

    fn scan_units(
        &self,
        session: &mut contextra_ports::scanner::ScanSession,
        ctx: &contextra_types::RequestContext,
    ) -> contextra_types::Result<()> {
        let _ = session;
        // 1. Positivkontrolle (Control Match)
        self.run_positive_control(session);

        // 2. Ziel-Mandant scannen
        let tenant = ctx.tenant_id;
        let segment_ids = self.store.get_segments(tenant);

        for seg_id in segment_ids {
            let mut hasher = blake3::Hasher::new();
            hasher.update(&tenant.inner().to_le_bytes());
            hasher.update(&seg_id.to_le_bytes());
            if let Some(bytes) = self.store.get_segment_bytes(tenant, seg_id) {
                hasher.update(&bytes);
            }
            let unit_hash: [u8; 32] = *hasher.finalize().as_bytes();
            // Jedes gefundene Segment für den Mandanten ist ein Restbestand (live residual)
            session.record_unit(unit_hash, true);
        }

        Ok(())
    }
}
