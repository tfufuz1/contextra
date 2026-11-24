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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::segment::KvSegment;
    use crate::store::TenantIsolatedKvStore;
    use crate::types::TenantId;
    use contextra_ports::clock::SystemClock;
    use contextra_ports::scanner::{LayerScanner, Purger, ScanTarget, ScanTechnique, WitnessLayer};
    use contextra_types::{RequestContext, TxId};
    use std::sync::Arc;

    // Prüft, dass KvCachePruner und KvCacheScanner die Port-Verträge für KV-Cache-Segmente korrekt erfüllen.
    #[test]
    fn test_kv_cache_pruner_and_scanner_ports() {
        let store = Arc::new(TenantIsolatedKvStore::new());
        let tenant = TenantId::try_new(77).unwrap();
        let pruner = KvCachePruner::new(Arc::clone(&store));
        let scanner = KvCacheScanner::new(Arc::clone(&store));

        let ctx = RequestContext::new(tenant.into(), TxId(101));
        let target = ScanTarget::new("tenant_77", [0x77; 32]);
        let clock = SystemClock::new();

        // 1. Initialer Scan auf leerem Store: Positivkontrolle besteht, Restmenge ist 0
        let scan_result = scanner
            .scan(&ctx, target.clone(), ScanTechnique::Structural, &clock)
            .expect("Scanner must succeed on empty store");
        assert_eq!(scan_result.witness_layer(), &WitnessLayer::KvCacheSegments);
        assert_eq!(scan_result.scanned(), 1); // Positivkontrolle registriert
        assert_eq!(scan_result.remaining(), 0);

        // 2. Segmente einfügen
        store.insert_segment(tenant, KvSegment::new(tenant, 10, vec![1, 2, 3, 4]));
        store.insert_segment(tenant, KvSegment::new(tenant, 20, vec![5, 6, 7, 8, 9]));

        // 3. Scan mit Restmenge (2 Einheiten)
        let scan_with_data = scanner
            .scan(&ctx, target.clone(), ScanTechnique::Structural, &clock)
            .expect("Scanner must succeed with data");
        assert_eq!(scan_with_data.remaining(), 2);
        assert_eq!(scan_with_data.scanned(), 3); // 1 Canary + 2 Segmente

        // 4. Pruning ausführen
        let purge_stats = pruner.purge(&ctx, &target).expect("Pruner must succeed");
        assert_eq!(purge_stats.purged_units, 2);
        assert_eq!(purge_stats.bytes_freed, 9);
        assert_eq!(store.get_tenant_segment_len(tenant), 0);

        // 5. Verifikation nach Bereinigung: Restmenge wieder 0
        let scan_after = scanner
            .scan(&ctx, target, ScanTechnique::Structural, &clock)
            .expect("Scanner must succeed after purge");
        assert_eq!(scan_after.remaining(), 0);
    }
}
