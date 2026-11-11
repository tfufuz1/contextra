//! Minimaler, autarker Umwelt- und Indexvertrag für `contextra-kvcache` (R2/R3-Lieferung).
//!
//! # Architektur
//! Der Kern definiert seine eigenen Umwelt-Abstraktionen ([`Clock`], [`Rng`], [`KvCacheEngine`])
//! und bietet mit [`MemoryKvCache`] eine I/O-freie, deterministische Referenzimplementierung.

use crate::error::Result;
use crate::radix::{KvReusePolicy, PrefixMatch};
use crate::segment::KvSegment;
use crate::store::TenantIsolatedKvStore;
use contextra_ids::TenantId;
use std::sync::Arc;

/// Minimaler Zeiterfassungs-Trait für das KV-Cache-Subsystem.
pub trait Clock: Send + Sync {
    /// Liefert die aktuellen Unix-Epochen-Millisekunden.
    fn now_unix_ms(&self) -> u64;
}

impl<T: contextra_env::Clock + ?Sized> Clock for T {
    #[inline]
    fn now_unix_ms(&self) -> u64 {
        self.now_unix_nanos() / 1_000_000
    }
}

/// Minimaler Zufallszahlen-Generator-Trait für Cache-Algorithmen.
pub trait Rng: Send {
    /// Erzeugt eine 64-Bit Pseudozufallszahl.
    fn next_u64(&mut self) -> u64;
}

/// Kernabstraktion für mandantenisolierte KV-Cache-Operationen.
pub trait KvCacheEngine: Send + Sync {
    /// Fügt ein KV-Segment ein.
    fn insert(&self, tenant_id: TenantId, segment: KvSegment) -> Result<()>;

    /// Ruft die Rohbytes eines Segments ab.
    fn get(&self, tenant_id: TenantId, segment_id: u64) -> Option<Vec<u8>>;

    /// Sucht den längsten übereinstimmenden Präfix für eine Token-Sequenz.
    fn find_longest_prefix(
        &self,
        tenant_id: TenantId,
        tokens: &[u32],
        policy: KvReusePolicy,
    ) -> Option<PrefixMatch>;
}

/// I/O-freie Referenzimplementierung eines KV-Caches im Hauptspeicher.
#[derive(Clone)]
pub struct MemoryKvCache {
    store: Arc<TenantIsolatedKvStore>,
}

impl MemoryKvCache {
    /// Erstellt eine neue Instanz des Memory-KV-Caches.
    pub fn new() -> Self {
        Self {
            store: Arc::new(TenantIsolatedKvStore::new()),
        }
    }

    /// Erstellt eine Instanz mit benutzerdefinierter Shard-Anzahl.
    pub fn with_shard_count(shard_count: usize) -> Self {
        Self {
            store: Arc::new(TenantIsolatedKvStore::with_shard_count(shard_count)),
        }
    }

    /// Liefert eine Referenz auf den internen Store.
    pub fn store(&self) -> &Arc<TenantIsolatedKvStore> {
        &self.store
    }
}

impl Default for MemoryKvCache {
    fn default() -> Self {
        Self::new()
    }
}

impl KvCacheEngine for MemoryKvCache {
    fn insert(&self, tenant_id: TenantId, segment: KvSegment) -> Result<()> {
        self.store.insert_segment(tenant_id, segment);
        Ok(())
    }

    fn get(&self, tenant_id: TenantId, segment_id: u64) -> Option<Vec<u8>> {
        self.store.get_segment_bytes(tenant_id, segment_id)
    }

    fn find_longest_prefix(
        &self,
        tenant_id: TenantId,
        tokens: &[u32],
        policy: KvReusePolicy,
    ) -> Option<PrefixMatch> {
        self.store.find_prefix_match(tenant_id, tokens, policy, None).map(|(pm, _guard)| pm)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_kv_cache_contract_roundtrip() {
        let cache = MemoryKvCache::new();
        if let Ok(tenant) = TenantId::try_new(42) {
            let segment = KvSegment::new(tenant, 10, vec![1, 2, 3, 4]);
            assert!(cache.insert(tenant, segment).is_ok());
            let bytes = cache.get(tenant, 10);
            assert_eq!(bytes, Some(vec![1, 2, 3, 4]));
        }
    }
}
