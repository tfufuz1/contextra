// FILE-CONTEXT
// ZWECK: Tenant-isolierter KV-Segment-Store (INV-TENANT Isolation) mit Prefix-Radix & Guard-Schutz.
// STAND: TS:2026-09-15T00:00:00Z

use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use ahash::AHashMap;
use contextra_types::{ContextraError, TenantId};
use lru::LruCache;
use parking_lot::RwLock;

use super::attention_score::{
    rank_for_eviction_weighted, AttentionScoreSource, NullAttentionScoreSource,
};
use super::eviction_worker::EvictionWorker;
use super::radix::{KvBlockGuard, KvReusePolicy, PrefixMatch, PrefixRadixTree};
use super::segment::KvSegment;

/// Optionaler Callback-Hook für Tier-2-LSM-Spill bei Eviction aus dem In-Memory LRU Cache.
pub type SpillHandler = Arc<dyn Fn(TenantId, u64, Vec<u8>) + Send + Sync>;

/// Kapselt den LRU-Cache und den Prefix-Radix-Baum aller KV-Segmente eines einzelnen Tenants.
struct TenantState {
    #[allow(dead_code)]
    tenant_id: TenantId,
    cache: LruCache<u64, KvSegment>,
    radix_tree: PrefixRadixTree,
    total_bytes: usize,
}

impl TenantState {
    fn new(tenant_id: TenantId, capacity: NonZeroUsize) -> Self {
        Self {
            tenant_id,
            cache: LruCache::new(capacity),
            radix_tree: PrefixRadixTree::new(tenant_id),
            total_bytes: 0,
        }
    }

    /// O(1) insert. Evictiert das älteste unreferenzierte Segment, falls capacity überschritten.
    /// Garantiert, dass aktive Blöcke (`active_refs > 0`) niemals verworfen werden.
    fn insert_returning_evicted(&mut self, segment: KvSegment) -> Option<KvSegment> {
        let id = segment.segment_id;
        let bytes = segment.len();

        let evicted = if self.cache.len() >= self.cache.cap().get() {
            self.pop_lru()
        } else {
            None
        };

        if self.cache.len() >= self.cache.cap().get() && evicted.is_none() {
            // Alle Segmente im Cache sind aktuell durch aktive Guards geschützt.
            // Erweitere die Kapazität vorübergehend, damit LruCache::push keine geschützten Blöcke verwirft.
            let new_cap =
                NonZeroUsize::new(self.cache.cap().get() + 1).unwrap_or(NonZeroUsize::MIN);
            self.cache.resize(new_cap);
        }

        if let Some((_, old)) = self.cache.push(id, segment) {
            self.total_bytes = self.total_bytes.saturating_sub(old.len());
        }
        self.total_bytes = self.total_bytes.saturating_add(bytes);
        evicted
    }

    /// O(1) get mit LRU-Update.
    fn get_bytes(&mut self, id: u64) -> Option<Vec<u8>> {
        self.cache.get(&id).map(|s| s.as_bytes().to_vec())
    }

    /// O(1) get für Entschlüsselung — benötigt &mut wegen LRU-Update.
    #[allow(dead_code)]
    fn get_segment_ref_mut(&mut self, id: u64) -> Option<&KvSegment> {
        self.cache.get(&id)
    }

    /// Erstellt einen `KvBlockGuard` für ein Segment.
    fn acquire_guard(
        &mut self,
        id: u64,
        worker: Option<Arc<EvictionWorker>>,
    ) -> Option<KvBlockGuard> {
        let seg = self.cache.get(&id)?;
        Some(seg.acquire_guard(worker))
    }

    /// O(1) entfernen. Gibt das Segment zurück (ZeroizeOnDrop beim Caller).
    fn remove(&mut self, id: u64) -> Option<KvSegment> {
        let seg = self.cache.pop(&id)?;
        self.total_bytes = self.total_bytes.saturating_sub(seg.len());
        Some(seg)
    }

    /// Candidate eviction selection incorporating LRU access age and attention scores.
    fn pop_eviction_candidate(
        &mut self,
        scores: &dyn AttentionScoreSource,
        attention_weight: f32,
    ) -> Option<KvSegment> {
        if self.cache.is_empty() {
            return None;
        }

        let unref_ids: Vec<u64> = self
            .cache
            .iter()
            .rev()
            .filter_map(|(&id, seg)| {
                if seg.active_refs() == 0 {
                    Some(id)
                } else {
                    None
                }
            })
            .collect();

        if unref_ids.is_empty() {
            return None;
        }

        if unref_ids.len() == 1 {
            return self.remove(unref_ids[0]);
        }

        let base_time = std::time::Instant::now();
        let candidates: Vec<(u64, std::time::Instant)> = unref_ids
            .into_iter()
            .enumerate()
            .map(|(i, id)| {
                (
                    id,
                    base_time + std::time::Duration::from_nanos(i as u64 * 1000),
                )
            })
            .collect();

        let ranked = rank_for_eviction_weighted(&candidates, scores, attention_weight);
        let target_id = ranked.first().copied()?;
        self.remove(target_id)
    }

    /// LRU-Eviction unter Schutz aktiver Referenzen (`active_refs == 0`).
    /// Iteriert von LRU (Least Recently Used) zu MRU.
    fn pop_lru(&mut self) -> Option<KvSegment> {
        self.pop_eviction_candidate(&NullAttentionScoreSource, 0.5)
    }

    /// Fügt eine Token-Sequenz in den Radix-Baum ein.
    fn insert_token_sequence(
        &mut self,
        tokens: &[u32],
        block_id: u64,
    ) -> Result<(), ContextraError> {
        self.radix_tree.insert(tokens, block_id)
    }

    /// Sucht ein Präfix-Match im Radix-Baum und erstellt einen Guard für das Caching-Segment.
    fn find_prefix_match(
        &mut self,
        tokens: &[u32],
        policy: KvReusePolicy,
        worker: Option<Arc<EvictionWorker>>,
    ) -> Option<(PrefixMatch, KvBlockGuard)> {
        let pm = self.radix_tree.find_longest_prefix(tokens, policy)?;
        let guard = self.acquire_guard(pm.block_id, worker)?;
        Some((pm, guard))
    }

    /// Listet alle Segment-IDs OHNE LRU-Update.
    fn segment_ids(&self) -> impl Iterator<Item = u64> + '_ {
        self.cache.iter().map(|(&id, _)| id)
    }

    fn len(&self) -> usize {
        self.cache.len()
    }

    fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

/// Cache-Line-Padding verhindert False-Sharing auf 64-Byte-Cache-Lines.
#[repr(align(64))]
struct Shard {
    lock: RwLock<AHashMap<TenantId, TenantState>>,
}

impl Shard {
    fn new() -> Self {
        Self {
            lock: RwLock::new(AHashMap::new()),
        }
    }
}

/// Tenant-isolierter KV-Segment-Store.
pub struct TenantIsolatedKvStore {
    shards: Box<[Shard]>,
    shard_count: usize,
    global_shard_offset: AtomicUsize,
    eviction_round_offsets: Box<[AtomicUsize]>,
    segment_capacity: NonZeroUsize,
    spill_handler: RwLock<Option<SpillHandler>>,
}

impl TenantIsolatedKvStore {
    pub const DEFAULT_SHARD_COUNT: usize = 32;
    pub const DEFAULT_SEGMENT_CAPACITY_PER_TENANT: usize = 256;

    /// Erstellt einen neuen tenant-isolierten KV-Store.
    pub fn new() -> Self {
        Self::with_shard_count(Self::DEFAULT_SHARD_COUNT)
    }

    /// Erstellt einen Store mit angegebener Shard-Anzahl.
    pub fn with_shard_count(n: usize) -> Self {
        let shard_count = n.next_power_of_two().max(1);
        let shards = (0..shard_count)
            .map(|_| Shard::new())
            .collect::<Vec<_>>()
            .into_boxed_slice();
        let offsets = (0..shard_count)
            .map(|_| AtomicUsize::new(0))
            .collect::<Vec<_>>()
            .into_boxed_slice();
        Self {
            shards,
            shard_count,
            global_shard_offset: AtomicUsize::new(0),
            eviction_round_offsets: offsets,
            segment_capacity: NonZeroUsize::new(Self::DEFAULT_SEGMENT_CAPACITY_PER_TENANT)
                .unwrap_or(NonZeroUsize::MIN),
            spill_handler: RwLock::new(None),
        }
    }

    /// Erstellt einen Store mit konfigurierter Segment-Kapazität pro Tenant.
    pub fn with_capacity(segment_capacity_per_tenant: usize) -> Self {
        let mut store = Self::new();
        store.segment_capacity =
            NonZeroUsize::new(segment_capacity_per_tenant.max(1)).unwrap_or(NonZeroUsize::MIN);
        store
    }

    /// Registriert einen Spill-Handler für evictierte Segmente.
    pub fn set_spill_handler(&self, handler: SpillHandler) {
        *self.spill_handler.write() = Some(handler);
    }

    #[inline]
    fn shard_idx(&self, tenant: TenantId) -> usize {
        (tenant.inner() as usize) & (self.shard_count - 1)
    }

    /// Fügt ein Segment für einen bestimmten Tenant ein.
    pub fn insert_segment(&self, tenant: TenantId, segment: KvSegment) {
        let idx = self.shard_idx(tenant);
        let capacity = self.segment_capacity;
        let evicted = {
            let mut shard = self.shards[idx].lock.write();
            let state = shard
                .entry(tenant)
                .or_insert_with(|| TenantState::new(tenant, capacity));
            state.insert_returning_evicted(segment)
        };

        if let Some(ev) = evicted {
            let handler_opt = self.spill_handler.read().clone();
            if let Some(handler) = handler_opt {
                handler(tenant, ev.segment_id, ev.to_spill_bytes());
            }
        }
    }

    /// Fügt eine Token-Sequenz und deren assoziierte Block-ID in den Prefix-Radix-Baum ein.
    pub fn insert_token_sequence(
        &self,
        tenant: TenantId,
        tokens: &[u32],
        block_id: u64,
    ) -> Result<(), ContextraError> {
        let idx = self.shard_idx(tenant);
        let capacity = self.segment_capacity;
        let mut shard = self.shards[idx].lock.write();
        let state = shard
            .entry(tenant)
            .or_insert_with(|| TenantState::new(tenant, capacity));
        state.insert_token_sequence(tokens, block_id)
    }

    /// Sucht ein Präfix-Match für die angegebenen Tokens unter der gewünschten `KvReusePolicy`.
    pub fn find_prefix_match(
        &self,
        tenant: TenantId,
        tokens: &[u32],
        policy: KvReusePolicy,
        worker: Option<Arc<EvictionWorker>>,
    ) -> Option<(PrefixMatch, KvBlockGuard)> {
        let idx = self.shard_idx(tenant);
        let mut shard = self.shards[idx].lock.write();
        let state = shard.get_mut(&tenant)?;
        state.find_prefix_match(tokens, policy, worker)
    }

    /// Erstellt einen `KvBlockGuard` für einen bestimmten Block.
    pub fn acquire_block_guard(
        &self,
        tenant: TenantId,
        block_id: u64,
        worker: Option<Arc<EvictionWorker>>,
    ) -> Option<KvBlockGuard> {
        let idx = self.shard_idx(tenant);
        let mut shard = self.shards[idx].lock.write();
        let state = shard.get_mut(&tenant)?;
        state.acquire_guard(block_id, worker)
    }

    /// Bereinigt assoziierte KV-Segmente bei einem transaktionalen Rollback.
    pub fn on_rollback(&self, tenant: TenantId, chunk_ids: &[u64]) {
        self.remove_segments_for_rollback(tenant, chunk_ids);
    }

    /// Entfernt Segmente mit den angegebenen `segment_ids` für einen Tenant.
    pub fn remove_segments_for_rollback(&self, tenant: TenantId, segment_ids: &[u64]) {
        if segment_ids.is_empty() {
            return;
        }
        let idx = self.shard_idx(tenant);
        let _deferred: Vec<KvSegment> = {
            let mut shard = self.shards[idx].lock.write();
            if let Some(state) = shard.get_mut(&tenant) {
                let removed = segment_ids
                    .iter()
                    .filter_map(|&id| state.remove(id))
                    .collect();
                if state.is_empty() {
                    shard.remove(&tenant);
                }
                removed
            } else {
                vec![]
            }
        };
        tracing::debug!(
            tenant_id = ?tenant,
            removed_segment_ids = ?segment_ids,
            "KvStore rollback: removed segments for failed transaction"
        );
    }

    /// Entfernt ein einzelnes Segment.
    pub fn remove_segment(&self, tenant: TenantId, segment_id: u64) {
        self.remove_segments_for_rollback(tenant, &[segment_id]);
    }

    /// Liefert unverschlüsselte Segment-Bytes für einen Tenant.
    pub fn get_segment_bytes(&self, tenant: TenantId, segment_id: u64) -> Option<Vec<u8>> {
        let idx = self.shard_idx(tenant);
        self.shards[idx]
            .lock
            .write()
            .get_mut(&tenant)?
            .get_bytes(segment_id)
    }

    /// Verschlüsselt einen Klartext-Tensor und fügt ein verschlüsseltes Segment ein.
    #[cfg(feature = "kv-encryption")]
    pub fn insert_encrypted_segment(
        &self,
        cipher: &contextra_crypto::KvSegmentCipher,
        tenant: TenantId,
        segment_id: u64,
        model_fingerprint: contextra_crypto::ModelFingerprint,
        rope_offset: Option<usize>,
        plaintext: &[u8],
    ) -> Result<(), contextra_crypto::CryptoError> {
        let segment = KvSegment::new_encrypted(
            cipher,
            tenant,
            segment_id,
            model_fingerprint,
            rope_offset,
            plaintext,
        )?;
        self.insert_segment(tenant, segment);
        Ok(())
    }

    /// Liefert alle gespeicherten Segment-IDs eines Tenants.
    pub fn get_segments(&self, tenant: TenantId) -> Vec<u64> {
        let idx = self.shard_idx(tenant);
        self.shards[idx]
            .lock
            .read()
            .get(&tenant)
            .map(|state| state.segment_ids().collect())
            .unwrap_or_default()
    }

    /// Liest ein Segment und entschlüsselt es falls nötig.
    #[cfg(feature = "kv-encryption")]
    pub fn get_decrypted_segment(
        &self,
        cipher: &contextra_crypto::KvSegmentCipher,
        tenant: TenantId,
        segment_id: u64,
    ) -> Result<Option<Vec<u8>>, contextra_crypto::CryptoError> {
        let idx = self.shard_idx(tenant);
        let mut shard = self.shards[idx].lock.write();
        if let Some(state) = shard.get_mut(&tenant) {
            if let Some(seg) = state.get_segment_ref_mut(segment_id) {
                let decrypted = seg.decrypt_data(cipher)?;
                return Ok(Some(decrypted));
            }
        }
        Ok(None)
    }

    /// Gibt die Anzahl der gespeicherten Segmente für einen Mandanten zurück.
    pub fn get_tenant_segment_len(&self, tenant: TenantId) -> usize {
        let idx = self.shard_idx(tenant);
        self.shards[idx]
            .lock
            .read()
            .get(&tenant)
            .map(|s| s.len())
            .unwrap_or(0)
    }

    /// Globale LRU Eviction unter Schutz aktiver Referenzen.
    #[allow(dead_code)]
    pub(crate) fn evict_lru_global(&self, target_free_bytes: usize) -> usize {
        let mut freed = 0;

        'outer: while freed < target_free_bytes {
            let mut made_progress = false;
            let start_shard =
                self.global_shard_offset.fetch_add(1, Ordering::Relaxed) % self.shard_count;

            for s_idx in 0..self.shard_count {
                if freed >= target_free_bytes {
                    break 'outer;
                }
                let shard_idx = (start_shard + s_idx) % self.shard_count;

                let evicted_opt: Option<(TenantId, KvSegment)> = {
                    let mut shard = self.shards[shard_idx].lock.write();
                    let tenant_opt = shard.keys().copied().next();
                    if let Some(tenant) = tenant_opt {
                        if let Some(state) = shard.get_mut(&tenant) {
                            if let Some(seg) = state.pop_lru() {
                                if state.is_empty() {
                                    shard.remove(&tenant);
                                }
                                Some((tenant, seg))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                };

                if let Some((tenant, evicted)) = evicted_opt {
                    freed += evicted.len();
                    made_progress = true;
                    tracing::debug!(
                        tenant_id = tenant.inner(),
                        segment_id = evicted.segment_id,
                        freed_bytes = evicted.len(),
                        "KV eviction worker: evicted segment"
                    );
                    drop(evicted);
                }
            }

            if !made_progress {
                break;
            }
        }

        freed
    }

    const MAX_ROUNDS_PER_LOCK_ACQUISITION: usize = 4;

    /// Evictiert KV-Segmente unter Erhaltung von Tenant-Fairness und Attention-Scores (Default 50/50 Gewichtung).
    pub fn evict_fair(&self, target_free_bytes: usize, scores: &dyn AttentionScoreSource) -> usize {
        self.evict_weighted_fair(target_free_bytes, scores, 0.5)
    }

    /// Evictiert KV-Segmente unter Erhaltung von Tenant-Fairness und gewichteten Attention-Scores.
    pub fn evict_weighted_fair(
        &self,
        target_free_bytes: usize,
        scores: &dyn AttentionScoreSource,
        attention_weight: f32,
    ) -> usize {
        #[cfg(test)]
        {
            self.evict_fair_internal(target_free_bytes, scores, attention_weight, None)
        }
        #[cfg(not(test))]
        {
            self.evict_fair_internal(target_free_bytes, scores, attention_weight)
        }
    }

    /// Evictiert KV-Segmente unter Erhaltung von Tenant-Fairness und Guard-Schutz (`active_refs == 0`).
    pub fn evict_lru_fair(&self, target_free_bytes: usize) -> usize {
        self.evict_fair(target_free_bytes, &NullAttentionScoreSource)
    }

    #[cfg(test)]
    pub fn evict_lru_fair_with_hook<F>(&self, target_free_bytes: usize, mut hook: F) -> usize
    where
        F: FnMut(),
    {
        self.evict_fair_internal(
            target_free_bytes,
            &NullAttentionScoreSource,
            0.5,
            Some(&mut hook),
        )
    }

    fn evict_fair_internal(
        &self,
        target_free_bytes: usize,
        scores: &dyn AttentionScoreSource,
        attention_weight: f32,
        #[cfg(test)] mut batch_released_hook: Option<&mut dyn FnMut()>,
    ) -> usize {
        let mut freed = 0;

        'outer: while freed < target_free_bytes {
            let mut made_progress_in_pass = false;
            let start_shard =
                self.global_shard_offset.fetch_add(1, Ordering::Relaxed) % self.shard_count;

            for _round in 0..Self::MAX_ROUNDS_PER_LOCK_ACQUISITION {
                if freed >= target_free_bytes {
                    break 'outer;
                }

                for s_idx in 0..self.shard_count {
                    if freed >= target_free_bytes {
                        break 'outer;
                    }

                    let shard_idx = (start_shard + s_idx) % self.shard_count;
                    let mut deferred_drop: Vec<KvSegment> = Vec::new();
                    #[cfg(test)]
                    let mut evicted_in_shard = false;

                    {
                        let mut shard = self.shards[shard_idx].lock.write();
                        if !shard.is_empty() {
                            let tenants: Vec<TenantId> = shard.keys().copied().collect();
                            let n = tenants.len();
                            if n > 0 {
                                let offset = self.eviction_round_offsets[shard_idx]
                                    .fetch_add(1, Ordering::Relaxed)
                                    % n;

                                for i in (offset..n).chain(0..offset) {
                                    if freed >= target_free_bytes {
                                        break;
                                    }
                                    let tenant = tenants[i];
                                    if let Some(state) = shard.get_mut(&tenant) {
                                        if let Some(evicted) =
                                            state.pop_eviction_candidate(scores, attention_weight)
                                        {
                                            freed += evicted.len();
                                            #[cfg(test)]
                                            {
                                                evicted_in_shard = true;
                                            }
                                            made_progress_in_pass = true;
                                            tracing::debug!(
                                                tenant_id = tenant.inner(),
                                                segment_id = evicted.segment_id,
                                                freed_bytes = evicted.len(),
                                                "KV eviction worker: evicted segment"
                                            );
                                            deferred_drop.push(evicted);
                                        }
                                        if state.is_empty() {
                                            shard.remove(&tenant);
                                        }
                                    }
                                }
                            }
                        }
                    }

                    let handler_opt = self.spill_handler.read().clone();
                    if let Some(ref handler) = handler_opt {
                        for ev in &deferred_drop {
                            handler(ev.tenant_id, ev.segment_id, ev.to_spill_bytes());
                        }
                    }
                    deferred_drop.clear();

                    #[cfg(test)]
                    if evicted_in_shard && freed < target_free_bytes {
                        if let Some(ref mut hook) = batch_released_hook {
                            hook();
                        }
                    }
                }
            }

            if !made_progress_in_pass {
                break;
            }
        }

        freed
    }

    pub fn clear_all(&self) {
        let mut all_segments: Vec<KvSegment> = Vec::new();
        for shard in self.shards.iter() {
            let mut s = shard.lock.write();
            for (_, mut state) in s.drain() {
                while let Some(seg) = state.pop_lru() {
                    all_segments.push(seg);
                }
            }
        }
        drop(all_segments);
        tracing::warn!("KV emergency_wipe: all segments zeroized synchronously");
    }
}

impl Default for TenantIsolatedKvStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_store_guarded_segments_never_force_evicted_on_overflow() {
        let store = TenantIsolatedKvStore::with_capacity(2);
        let tenant = TenantId::try_new(10).unwrap();

        store.insert_segment(tenant, KvSegment::new(tenant, 1, vec![0x11; 256]));
        store.insert_segment(tenant, KvSegment::new(tenant, 2, vec![0x22; 256]));

        let guard1 = store.acquire_block_guard(tenant, 1, None).unwrap();
        let guard2 = store.acquire_block_guard(tenant, 2, None).unwrap();

        assert_eq!(guard1.active_refs(), 1);
        assert_eq!(guard2.active_refs(), 1);

        // Insert 3rd segment into full capacity store (2) where all items are guarded
        store.insert_segment(tenant, KvSegment::new(tenant, 3, vec![0x33; 256]));

        let current_ids = store.get_segments(tenant);
        assert!(
            current_ids.contains(&1),
            "Guarded segment 1 must NOT be force-evicted!"
        );
        assert!(
            current_ids.contains(&2),
            "Guarded segment 2 must NOT be force-evicted!"
        );
        assert!(
            current_ids.contains(&3),
            "Newly inserted segment 3 must be present!"
        );

        drop(guard1);
        drop(guard2);
    }

    #[test]
    fn test_tenant_isolation_no_cross_read() {
        let store = TenantIsolatedKvStore::new();

        let tenant_a = TenantId::try_new(1).unwrap();
        let tenant_b = TenantId::try_new(2).unwrap();

        let seg_a = KvSegment::new(tenant_a, 101, vec![0x11; 128]);
        let seg_b = KvSegment::new(tenant_b, 202, vec![0x22; 256]);

        store.insert_segment(tenant_a, seg_a);
        store.insert_segment(tenant_b, seg_b);

        let segs_a = store.get_segments(tenant_a);
        assert_eq!(segs_a, vec![101]);
        assert_eq!(store.get_tenant_segment_len(tenant_a), 1);

        let segs_b = store.get_segments(tenant_b);
        assert_eq!(segs_b, vec![202]);
        assert_eq!(store.get_tenant_segment_len(tenant_b), 1);

        let tenant_c = TenantId::try_new(3).unwrap();
        assert!(store.get_segments(tenant_c).is_empty());
        assert_eq!(store.get_tenant_segment_len(tenant_c), 0);
    }

    #[test]
    fn test_evict_lru_fair_does_not_starve_inactive_tenant() {
        let store = TenantIsolatedKvStore::new();

        let tenant_a = TenantId::try_new(1).unwrap();
        let tenant_b = TenantId::try_new(2).unwrap();

        let seg_b = KvSegment::new(tenant_b, 201, vec![0x22; 256]);
        store.insert_segment(tenant_b, seg_b);

        std::thread::sleep(std::time::Duration::from_millis(5));

        for i in 1..=10 {
            let seg_a = KvSegment::new(tenant_a, i, vec![0x11; 256]);
            store.insert_segment(tenant_a, seg_a);
        }

        for i in 1..=10 {
            let _ = store.get_segment_bytes(tenant_a, i);
        }

        assert_eq!(store.get_tenant_segment_len(tenant_a), 10);
        assert_eq!(store.get_tenant_segment_len(tenant_b), 1);

        let freed = store.evict_lru_fair(256);
        assert!(freed >= 256);

        assert_eq!(store.get_tenant_segment_len(tenant_a), 9);
        assert_eq!(
            store.get_tenant_segment_len(tenant_b),
            1,
            "Tenant B (inactive) must not be starved by Tenant A"
        );
    }

    #[test]
    fn test_evict_lru_fair_respects_target_free_bytes() {
        let store = TenantIsolatedKvStore::new();

        let tenant_a = TenantId::try_new(1).unwrap();
        let tenant_b = TenantId::try_new(2).unwrap();

        for i in 1..=3 {
            store.insert_segment(tenant_a, KvSegment::new(tenant_a, i, vec![0x11; 512]));
            store.insert_segment(tenant_b, KvSegment::new(tenant_b, i + 10, vec![0x22; 512]));
        }

        assert_eq!(store.get_tenant_segment_len(tenant_a), 3);
        assert_eq!(store.get_tenant_segment_len(tenant_b), 3);

        let freed = store.evict_lru_fair(1000);
        assert!(
            freed >= 1000,
            "freed bytes ({freed}) must be >= target_free_bytes (1000)"
        );

        assert_eq!(store.get_tenant_segment_len(tenant_a), 2);
        assert_eq!(store.get_tenant_segment_len(tenant_b), 2);
    }

    #[test]
    fn test_evict_lru_fair_releases_lock_between_batches() {
        let store = Arc::new(TenantIsolatedKvStore::new());

        let tenant_a = TenantId::try_new(1).unwrap();
        let tenant_b = TenantId::try_new(2).unwrap();
        let tenant_c = TenantId::try_new(3).unwrap();
        let tenant_unaffected = TenantId::try_new(99).unwrap();

        for i in 1..=30 {
            store.insert_segment(tenant_a, KvSegment::new(tenant_a, i, vec![0x11; 256]));
            store.insert_segment(tenant_b, KvSegment::new(tenant_b, i + 100, vec![0x22; 256]));
        }
        for i in 1..=50 {
            store.insert_segment(tenant_c, KvSegment::new(tenant_c, i + 200, vec![0x33; 256]));
        }

        let notify_batch_released = Arc::new(tokio::sync::Notify::new());
        let notify_read_complete = Arc::new(tokio::sync::Notify::new());
        let reads_during_eviction = Arc::new(AtomicUsize::new(0));

        let store_clone = Arc::clone(&store);
        let notify_batch_released_clone = Arc::clone(&notify_batch_released);
        let notify_read_complete_clone = Arc::clone(&notify_read_complete);
        let reads_during_eviction_clone = Arc::clone(&reads_during_eviction);

        let rt = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap();

        let reader_handle = thread::spawn(move || {
            rt.block_on(async {
                loop {
                    notify_batch_released_clone.notified().await;
                    let len = store_clone.get_tenant_segment_len(tenant_c);
                    let segs = store_clone.get_segments(tenant_unaffected);
                    assert!(segs.is_empty(), "Unaffected tenant 99 must have 0 segments");
                    reads_during_eviction_clone.fetch_add(1, Ordering::SeqCst);
                    notify_read_complete_clone.notify_one();
                    if len <= 38 {
                        break;
                    }
                }
            });
        });

        let notify_batch_released_evict = Arc::clone(&notify_batch_released);
        let notify_read_complete_evict = Arc::clone(&notify_read_complete);

        let freed = store.evict_lru_fair_with_hook(9_216, || {
            notify_batch_released_evict.notify_one();
            tokio::runtime::Builder::new_current_thread()
                .build()
                .unwrap()
                .block_on(async {
                    notify_read_complete_evict.notified().await;
                });
        });

        notify_batch_released.notify_one();
        reader_handle.join().expect("reader thread panicked");

        assert!(freed >= 9_216, "must free at least 9,216 bytes");
        assert_eq!(store.get_tenant_segment_len(tenant_a), 18);
        assert_eq!(store.get_tenant_segment_len(tenant_b), 18);
        assert_eq!(store.get_tenant_segment_len(tenant_c), 38);

        let successful_reads = reads_during_eviction.load(Ordering::SeqCst);
        assert!(
            successful_reads > 0,
            "Reader thread must execute at least one successful read while eviction is in progress (got {successful_reads} reads)"
        );
    }

    #[test]
    fn test_evict_lru_global_visibility_or_deprecation() {
        let _store = TenantIsolatedKvStore::new();
        let tenant = TenantId::try_new(1).unwrap();
        let _seg = KvSegment::new(tenant, 1, vec![1, 2, 3]);
    }

    #[test]
    fn test_evict_lru_fair_rotation_prevents_low_id_bias() {
        let store = TenantIsolatedKvStore::new();

        for tenant_id in 1..=5 {
            let tenant = TenantId::try_new(tenant_id as u64).unwrap();
            for seg_id in 0..10 {
                let seg = KvSegment::new(tenant, seg_id, vec![1; 100]);
                store.insert_segment(tenant, seg);
            }
        }

        let mut evicted_by_tenant = std::collections::HashMap::new();

        for _call_num in 0..5 {
            store.evict_lru_fair(150);

            for tenant in [1u64, 2, 3, 4, 5].iter() {
                let tenant_id = TenantId::try_new(*tenant).unwrap();
                let remaining = store.get_tenant_segment_len(tenant_id);
                evicted_by_tenant.insert(*tenant, 10 - remaining);
            }
        }

        let eviction_counts: Vec<usize> = evicted_by_tenant.values().copied().collect();
        let min_evictions = *eviction_counts.iter().min().unwrap_or(&0);
        let max_evictions = *eviction_counts.iter().max().unwrap_or(&100);

        assert!(
            max_evictions - min_evictions <= 2,
            "Eviction bias detected: min_evictions={}, max_evictions={}; \
             tenants should be hit more evenly. Rotation may not be working.",
            min_evictions,
            max_evictions
        );
    }

    #[test]
    fn test_store_edge_cases() {
        let store = TenantIsolatedKvStore::default();
        let tenant = TenantId::try_new(10).unwrap();

        assert!(store.get_segment_bytes(tenant, 999).is_none());

        let freed_zero = store.evict_lru_fair(0);
        assert_eq!(freed_zero, 0);

        let freed_empty = store.evict_lru_fair(500);
        assert_eq!(freed_empty, 0);

        let seg1 = KvSegment::new(tenant, 1, vec![1, 2, 3]);
        let seg2 = KvSegment::new(tenant, 1, vec![4, 5, 6, 7]);
        store.insert_segment(tenant, seg1);
        store.insert_segment(tenant, seg2);

        assert_eq!(store.get_tenant_segment_len(tenant), 1);
        assert_eq!(store.get_segment_bytes(tenant, 1), Some(vec![4, 5, 6, 7]));
    }

    #[test]
    fn test_store_clear_all_and_global_lru() {
        let store = TenantIsolatedKvStore::new();
        let tenant_a = TenantId::try_new(1).unwrap();
        let tenant_b = TenantId::try_new(2).unwrap();

        store.insert_segment(tenant_a, KvSegment::new(tenant_a, 1, vec![10; 256]));
        store.insert_segment(tenant_b, KvSegment::new(tenant_b, 2, vec![20; 256]));

        assert_eq!(store.get_tenant_segment_len(tenant_a), 1);
        assert_eq!(store.get_tenant_segment_len(tenant_b), 1);

        let freed = store.evict_lru_global(200);
        assert!(freed >= 200);
        assert_eq!(
            store.get_tenant_segment_len(tenant_a) + store.get_tenant_segment_len(tenant_b),
            1
        );

        store.clear_all();
        assert_eq!(store.get_tenant_segment_len(tenant_a), 0);
        assert_eq!(store.get_tenant_segment_len(tenant_b), 0);
    }

    #[test]
    fn test_on_rollback_cleans_up_correctly() {
        let store = TenantIsolatedKvStore::new();
        let tenant = TenantId::try_new(42).unwrap();

        for id in [10u64, 20, 30] {
            store.insert_segment(tenant, KvSegment::new(tenant, id, vec![id as u8; 8]));
        }
        assert_eq!(store.get_tenant_segment_len(tenant), 3);

        store.on_rollback(tenant, &[10, 30]);
        assert_eq!(store.get_tenant_segment_len(tenant), 1);
        assert!(store.get_segment_bytes(tenant, 10).is_none());
        assert!(store.get_segment_bytes(tenant, 20).is_some());
        assert!(store.get_segment_bytes(tenant, 30).is_none());

        store.on_rollback(tenant, &[20]);
        assert_eq!(store.get_tenant_segment_len(tenant), 0);
        assert!(store.get_segments(tenant).is_empty());
    }

    #[test]
    fn test_remove_segments_for_rollback_cleans_up_correctly() {
        let store = TenantIsolatedKvStore::new();
        let tenant = TenantId::try_new(42).unwrap();

        for id in [1u64, 2, 3] {
            store.insert_segment(tenant, KvSegment::new(tenant, id, vec![id as u8; 8]));
        }
        assert_eq!(store.get_tenant_segment_len(tenant), 3);

        store.remove_segments_for_rollback(tenant, &[1, 3]);
        assert_eq!(store.get_tenant_segment_len(tenant), 1);
        assert!(
            store.get_segment_bytes(tenant, 1).is_none(),
            "Segment 1 muss entfernt sein"
        );
        assert!(
            store.get_segment_bytes(tenant, 2).is_some(),
            "Segment 2 muss erhalten bleiben"
        );
        assert!(
            store.get_segment_bytes(tenant, 3).is_none(),
            "Segment 3 muss entfernt sein"
        );

        store.remove_segments_for_rollback(tenant, &[2]);
        assert_eq!(store.get_tenant_segment_len(tenant), 0);
        assert!(store.get_segments(tenant).is_empty());
    }

    #[test]
    fn test_remove_segments_noop_on_empty_ids() {
        let store = TenantIsolatedKvStore::new();
        let tenant = TenantId::try_new(1).unwrap();
        store.insert_segment(tenant, KvSegment::new(tenant, 1, vec![0u8; 8]));
        store.remove_segments_for_rollback(tenant, &[]);
        assert_eq!(store.get_tenant_segment_len(tenant), 1);
    }

    #[test]
    fn test_store_prefix_radix_and_block_guard_protection() {
        let store = TenantIsolatedKvStore::new();
        let tenant = TenantId::try_new(10).unwrap();

        let seg = KvSegment::new(tenant, 501, vec![0xFF; 512]);
        store.insert_segment(tenant, seg);

        let tokens = vec![1, 2, 3, 4, 5, 6];
        store.insert_token_sequence(tenant, &tokens, 501).unwrap();

        let (pm, guard) = store
            .find_prefix_match(
                tenant,
                &[1, 2, 3, 4, 5, 6, 7],
                KvReusePolicy::CostBased { min_prefix_len: 4 },
                None,
            )
            .expect("Should match prefix [1..6]");

        assert_eq!(pm.matched_len, 6);
        assert_eq!(pm.block_id, 501);
        assert_eq!(guard.active_refs(), 1);

        let freed_while_guarded = store.evict_lru_fair(512);
        assert_eq!(
            freed_while_guarded, 0,
            "Active guard MUST protect segment from LRU eviction!"
        );
        assert_eq!(store.get_tenant_segment_len(tenant), 1);

        drop(guard);

        let freed_after_drop = store.evict_lru_fair(512);
        assert!(freed_after_drop >= 512);
        assert_eq!(store.get_tenant_segment_len(tenant), 0);
    }
}
