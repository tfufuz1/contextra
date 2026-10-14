// FILE-CONTEXT
// ZWECK: KvShreddingWiring — Einstiegspunkt zur Anbindung von Segment-Schlüsseln an Löschereignisse und Crypto-Shredding.
// STAND: TS:2026-10-13T00:00:00Z

#![forbid(unsafe_code)]

use contextra_crypto::kv_shredding::{shred_segment_key, GroupRevocationListener, KeyRegistry};
use contextra_types::{ContextraError, TenantId};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

use crate::segment::ShreddableSegmentKey;

/// Abhängigkeiten für das Aufsetzen der `KvShreddingWiring`.
pub struct KvShreddingWiringDeps {
    /// In-Memory oder durabler `KeyRegistry` für Crypto-Shredding.
    pub key_registry: Arc<KeyRegistry>,
}

/// In-Memory Revocation Listener, der bei Gruppen-Löschereignissen (`on_group_revoked`)
/// automatisch registrierte `ShreddableSegmentKey` Instanzen gezeroized/shredded.
pub struct ShreddingRevocationListener {
    keys: Arc<RwLock<HashMap<(TenantId, u64), ShreddableSegmentKey>>>,
}

impl GroupRevocationListener for ShreddingRevocationListener {
    fn on_group_revoked(&self, group_id: u64) {
        let mut guard = self.keys.write();
        let target_keys: Vec<((TenantId, u64), ShreddableSegmentKey)> = guard
            .iter()
            .filter(|((_tenant, seg_id), _key)| *seg_id == group_id)
            .map(|(k, v)| (*k, v.clone()))
            .collect();

        for (k, key) in target_keys {
            key.shred();
            guard.remove(&k);
        }
    }
}

/// `KvShreddingWiring` verbindet Segment-Schlüssel, Löschereignisse (Mandant/Collection/Gruppe)
/// und `contextra_crypto::kv_shredding`.
///
/// Nach einem Löschereignis (`handle_tenant_deletion`, `handle_collection_deletion` oder `shred_group`)
/// wird der Segmentschlüssel vernichtet, wodurch verknüpfte Segmente dauerhaft unlesbar werden.
pub struct KvShreddingWiring {
    key_registry: Arc<KeyRegistry>,
    registered_keys: Arc<RwLock<HashMap<(TenantId, u64), ShreddableSegmentKey>>>,
    collection_keys: Arc<RwLock<HashMap<(TenantId, String), Vec<u64>>>>,
}

impl KvShreddingWiring {
    /// Erzeugt eine neue `KvShreddingWiring` Instanz mit den übergebenen Abhängigkeiten.
    pub fn build(deps: KvShreddingWiringDeps) -> Result<Self, ContextraError> {
        let registered_keys = Arc::new(RwLock::new(HashMap::new()));
        let collection_keys = Arc::new(RwLock::new(HashMap::new()));

        let _listener = Arc::new(ShreddingRevocationListener {
            keys: Arc::clone(&registered_keys),
        });

        Ok(Self {
            key_registry: deps.key_registry,
            registered_keys,
            collection_keys,
        })
    }

    /// Registriert einen shred-fähigen Segment-Schlüssel für einen Mandanten und ein Segment.
    pub fn register_segment_key(
        &self,
        tenant_id: TenantId,
        segment_id: u64,
        key: ShreddableSegmentKey,
    ) {
        self.registered_keys
            .write()
            .insert((tenant_id, segment_id), key);
    }

    /// Verknüpft ein Segment mit einem Collection-Namen für zielgerichtetes Collection-Shredding.
    pub fn register_collection_segment(
        &self,
        tenant_id: TenantId,
        collection_name: &str,
        segment_id: u64,
    ) {
        self.collection_keys
            .write()
            .entry((tenant_id, collection_name.to_string()))
            .or_default()
            .push(segment_id);
    }

    /// Löst Crypto-Shredding für alle Segmente eines Mandanten aus.
    /// Vernichtet alle verknüpften Segment-Schlüssel im Speicher.
    pub fn handle_tenant_deletion(&self, tenant_id: TenantId) -> Result<usize, ContextraError> {
        let mut keys_guard = self.registered_keys.write();
        let target_keys: Vec<(TenantId, u64)> = keys_guard
            .keys()
            .filter(|(t, _)| *t == tenant_id)
            .copied()
            .collect();

        let count = target_keys.len();
        for (t, seg_id) in target_keys {
            if let Some(key) = keys_guard.remove(&(t, seg_id)) {
                key.shred();
                let _ = shred_segment_key(&self.key_registry, t, seg_id);
            }
        }

        let mut col_guard = self.collection_keys.write();
        col_guard.retain(|(t, _), _| *t != tenant_id);

        Ok(count)
    }

    /// Löst Crypto-Shredding für alle Segmente einer spezifischen Collection eines Mandanten aus.
    pub fn handle_collection_deletion(
        &self,
        tenant_id: TenantId,
        collection_name: &str,
    ) -> Result<usize, ContextraError> {
        let segment_ids = {
            let mut col_guard = self.collection_keys.write();
            col_guard
                .remove(&(tenant_id, collection_name.to_string()))
                .unwrap_or_default()
        };

        let mut keys_guard = self.registered_keys.write();
        let mut count = 0;

        for seg_id in segment_ids {
            if let Some(key) = keys_guard.remove(&(tenant_id, seg_id)) {
                key.shred();
                let _ = shred_segment_key(&self.key_registry, tenant_id, seg_id);
                count += 1;
            }
        }

        Ok(count)
    }

    /// Löst Crypto-Shredding für eine einzelne Gruppe / Segment-ID aus.
    pub fn shred_group(&self, tenant_id: TenantId, segment_id: u64) -> Result<bool, ContextraError> {
        let key = self.registered_keys.write().remove(&(tenant_id, segment_id));
        if let Some(key) = key {
            key.shred();
            let _ = shred_segment_key(&self.key_registry, tenant_id, segment_id);
            Ok(true)
        } else {
            let _ = shred_segment_key(&self.key_registry, tenant_id, segment_id);
            Ok(false)
        }
    }

    /// Prüft, ob der Segment-Schlüssel für ein bestimmtes Segment ge-shredded wurde.
    pub fn is_shredded(&self, tenant_id: TenantId, segment_id: u64) -> bool {
        let guard = self.registered_keys.read();
        match guard.get(&(tenant_id, segment_id)) {
            Some(key) => key.is_shredded(),
            None => true, // Wenn kein Key mehr registriert ist (entfernt & ge-shredded), gilt er als ge-shredded
        }
    }

    /// Gibt die Referenz auf die unterliegende KeyRegistry zurück.
    pub fn key_registry(&self) -> &Arc<KeyRegistry> {
        &self.key_registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::segment::Tier2EncryptedSegment;

    #[test]
    fn test_kv_shredding_wiring_tenant_deletion() {
        let key_registry = Arc::new(KeyRegistry::new());
        let deps = KvShreddingWiringDeps {
            key_registry: Arc::clone(&key_registry),
        };
        let wiring = KvShreddingWiring::build(deps).unwrap();

        let tenant_a = TenantId::try_new(100).unwrap();
        let tenant_b = TenantId::try_new(200).unwrap();

        let key_a = ShreddableSegmentKey::try_new_random("secret-a").unwrap();
        let key_b = ShreddableSegmentKey::try_new_random("secret-b").unwrap();

        let seg_a = Tier2EncryptedSegment::new(tenant_a, 1001, b"tenant-a-payload", key_a.clone()).unwrap();
        let seg_b = Tier2EncryptedSegment::new(tenant_b, 2001, b"tenant-b-payload", key_b.clone()).unwrap();

        wiring.register_segment_key(tenant_a, 1001, key_a);
        wiring.register_segment_key(tenant_b, 2001, key_b);

        // Before deletion, both readable
        assert_eq!(seg_a.read_and_decrypt().unwrap(), b"tenant-a-payload");
        assert_eq!(seg_b.read_and_decrypt().unwrap(), b"tenant-b-payload");

        // Action: Deletion event for Tenant A
        let shredded_count = wiring.handle_tenant_deletion(tenant_a).unwrap();
        assert_eq!(shredded_count, 1);

        // Postcondition: Tenant A's segment is permanently unreadable (key shredded)
        assert!(seg_a.read_and_decrypt().is_err());
        assert!(wiring.is_shredded(tenant_a, 1001));

        // Tenant B's segment remains intact and readable
        assert_eq!(seg_b.read_and_decrypt().unwrap(), b"tenant-b-payload");
        assert!(!wiring.is_shredded(tenant_b, 2001));
    }

    #[test]
    fn test_kv_shredding_wiring_collection_deletion() {
        let key_registry = Arc::new(KeyRegistry::new());
        let deps = KvShreddingWiringDeps {
            key_registry: Arc::clone(&key_registry),
        };
        let wiring = KvShreddingWiring::build(deps).unwrap();

        let tenant = TenantId::try_new(1).unwrap();
        let key1 = ShreddableSegmentKey::try_new_random("pass1").unwrap();
        let key2 = ShreddableSegmentKey::try_new_random("pass2").unwrap();

        let seg1 = Tier2EncryptedSegment::new(tenant, 10, b"col1_data", key1.clone()).unwrap();
        let seg2 = Tier2EncryptedSegment::new(tenant, 20, b"col2_data", key2.clone()).unwrap();

        wiring.register_segment_key(tenant, 10, key1);
        wiring.register_segment_key(tenant, 20, key2);

        wiring.register_collection_segment(tenant, "collection_docs", 10);
        wiring.register_collection_segment(tenant, "collection_notes", 20);

        // Action: Delete collection_docs
        let shredded = wiring.handle_collection_deletion(tenant, "collection_docs").unwrap();
        assert_eq!(shredded, 1);

        // collection_docs segment is shredded
        assert!(seg1.read_and_decrypt().is_err());
        // collection_notes segment remains readable
        assert_eq!(seg2.read_and_decrypt().unwrap(), b"col2_data");
    }
}
