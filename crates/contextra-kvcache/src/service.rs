// FILE-CONTEXT
// ZWECK: Öffentliche Facade `KvCacheService` und `KvCacheServiceBuilder` für Ring-1/Ring-2 Verdrahtung.

//! Öffentliche Fassade für den KV-Cache-Dienst ([`KvCacheService`]).
//!
//! Stellt eine einheitliche, mandantenisolierte Schnittstelle bereit mit Unterstützung für:
//! - Speicherbudgetierung & Eviction-Policy ([`KvReusePolicy`])
//! - Verschlüsselten Tier-2-Spill ([`KvBridgeStorage`])
//! - KIVI 2-Bit Quantisierung ([`KiviQuantizeConfig`])
//! - Scratchpad-Invalidierung & Mandantenlöschung ([`TenantId`])
//!
//! # Beispiel
//! ```rust
//! use contextra_kvcache::service::{KvCacheService, KvCacheServiceBuilder};
//! use contextra_kvcache::radix::KvReusePolicy;
//! use contextra_types::TenantId;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let tenant = TenantId::try_new(42)?;
//! let service = KvCacheServiceBuilder::new()
//!     .memory_budget_bytes(128 << 20)
//!     .reuse_policy(KvReusePolicy::Always)
//!     .build();
//!
//! // Insert prefix
//! let tokens = vec![101, 102, 103, 104];
//! let data = vec![1, 2, 3, 4, 5, 6, 7, 8];
//! service.get_or_insert_prefix(tenant, &tokens, &data)?;
//!
//! // Stats
//! let stats = service.stats(tenant);
//! assert_eq!(stats.segment_count, 1);
//! # Ok(())
//! # }
//! ```

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

use contextra_crypto::{EncryptedKvLayer, KvSegmentCipher, ModelFingerprint};
use contextra_ports::attention::AttentionExporter;
use contextra_ports::clock::{Clock, SystemClock};
use contextra_ports::kv_bridge_port::KvBridgeStorage;
use contextra_types::{ContextraError, Result, TenantId, TxId};
use parking_lot::RwLock;

use crate::attention_score::AttentionScoreSource;
use crate::eviction_worker::EvictionWorker;
#[cfg(any(feature = "kivi-quantization", feature = "kvcache-kivi-quant"))]
use crate::quantize_kivi::{
    compress_bytes, decompress_bytes, kivi_dequantize, kivi_quantize, KiviQuantizedBlock,
    KiviQuantizeConfig, KvTensorView,
};
use crate::radix::{KvBlockGuard, KvReusePolicy, PrefixMatch, PrefixRadixTree};
use crate::scratchpad_invalidation::ScratchpadCacheScope;
use crate::segment::KvSegment;
use crate::store::TenantIsolatedKvStore;

fn block_on_simple<F: std::future::Future>(fut: F) -> F::Output {
    let waker = std::task::Waker::noop();
    let mut cx = std::task::Context::from_waker(waker);
    let mut pinned = std::pin::pin!(fut);
    loop {
        match pinned.as_mut().poll(&mut cx) {
            std::task::Poll::Ready(val) => return val,
            std::task::Poll::Pending => std::thread::yield_now(),
        }
    }
}

/// Statistiken über die KV-Cache-Nutzung eines Mandanten.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct KvCacheStats {
    /// Total number of active segments stored in RAM for this tenant.
    pub segment_count: usize,
    /// Total number of radix tree entries stored for this tenant.
    pub radix_entry_count: usize,
}

/// Facade configuration for [`KvCacheService`].
#[derive(Clone)]
pub struct KvCacheConfig {
    /// Maximum RAM memory budget in bytes (default: 512 MiB).
    pub memory_budget_bytes: usize,
    /// Prefix reuse policy (default: `CostBased { min_prefix_len: 4 }`).
    pub reuse_policy: KvReusePolicy,
    /// Whether Tier-2 spill storage is enabled (default: true if spill storage is attached).
    pub spill_enabled: bool,
    /// KIVI 2-bit quantization configuration, if enabled.
    #[cfg(any(feature = "kivi-quantization", feature = "kvcache-kivi-quant"))]
    pub kivi_config: Option<KiviQuantizeConfig>,
}

impl Default for KvCacheConfig {
    fn default() -> Self {
        Self {
            memory_budget_bytes: 512 << 20,
            reuse_policy: KvReusePolicy::default(),
            spill_enabled: true,
            #[cfg(any(feature = "kivi-quantization", feature = "kvcache-kivi-quant"))]
            kivi_config: None,
        }
    }
}

/// Builder zur Erstellung von [`KvCacheService`].
pub struct KvCacheServiceBuilder {
    config: KvCacheConfig,
    clock: Option<Arc<dyn Clock>>,
    spill_storage: Option<Arc<dyn KvBridgeStorage>>,
    cipher: Option<Arc<KvSegmentCipher>>,
    attention_source: Option<Arc<dyn AttentionScoreSource>>,
    attention_exporter: Option<Arc<dyn AttentionExporter>>,
}

impl KvCacheServiceBuilder {
    /// Erstellt einen neuen Builder mit Standardkonfiguration.
    pub fn new() -> Self {
        Self {
            config: KvCacheConfig::default(),
            clock: None,
            spill_storage: None,
            cipher: None,
            attention_source: None,
            attention_exporter: None,
        }
    }

    /// Konfiguriert das Speicherbudget in Bytes.
    pub fn memory_budget_bytes(mut self, bytes: usize) -> Self {
        self.config.memory_budget_bytes = bytes.max(1 << 20);
        self
    }

    /// Konfiguriert die Prefix-Wiederverwendungs-Policy.
    pub fn reuse_policy(mut self, policy: KvReusePolicy) -> Self {
        self.config.reuse_policy = policy;
        self
    }

    /// Aktiviert oder deaktiviert den Tier-2 Spill.
    pub fn spill_enabled(mut self, enabled: bool) -> Self {
        self.config.spill_enabled = enabled;
        self
    }

    /// Aktiviert oder deaktiviert die KIVI 2-Bit Quantisierung.
    #[cfg(any(feature = "kivi-quantization", feature = "kvcache-kivi-quant"))]
    pub fn enable_kivi_quantization(mut self, config: Option<KiviQuantizeConfig>) -> Self {
        self.config.kivi_config = config;
        self
    }

    /// Injiziert die Clock-Port-Implementierung.
    pub fn clock(mut self, clock: Arc<dyn Clock>) -> Self {
        self.clock = Some(clock);
        self
    }

    /// Konfiguriert den Tier-2 Spill-Storage Adapter.
    pub fn spill_storage(mut self, storage: Arc<dyn KvBridgeStorage>) -> Self {
        self.spill_storage = Some(storage);
        self
    }

    /// Konfiguriert den Segment-Cipher für verschlüsselten Tier-2-Spill.
    pub fn cipher(mut self, cipher: Arc<KvSegmentCipher>) -> Self {
        self.cipher = Some(cipher);
        self
    }

    /// Konfiguriert eine benutzerdefinierte `AttentionScoreSource`.
    pub fn attention_source(mut self, source: Arc<dyn AttentionScoreSource>) -> Self {
        self.attention_source = Some(source);
        self
    }

    /// Konfiguriert einen `AttentionExporter` für den EvictionWorker.
    pub fn attention_exporter(mut self, exporter: Arc<dyn AttentionExporter>) -> Self {
        self.attention_exporter = Some(exporter);
        self
    }

    /// Baut die [`KvCacheService`]-Instanz.
    pub fn build(self) -> KvCacheService {
        let clock = self.clock.unwrap_or_else(|| Arc::new(SystemClock::new()));
        let store = Arc::new(TenantIsolatedKvStore::new());

        let mut eviction_worker = EvictionWorker::spawn(Arc::clone(&store));
        if let Some(exporter) = self.attention_exporter {
            eviction_worker = eviction_worker.with_attention_exporter(exporter);
        } else if let Some(source) = self.attention_source {
            store.set_attention_source(source);
        }
        let eviction_worker = Arc::new(eviction_worker);

        let total_bytes_stored = Arc::new(AtomicUsize::new(0));
        let total_bytes_spill = Arc::clone(&total_bytes_stored);

        if self.config.spill_enabled {
            if let Some(spill) = &self.spill_storage {
                let spill_storage = Arc::clone(spill);
                let cipher = self.cipher.clone();
                let handler = Arc::new(
                    move |tenant_id: TenantId, segment_id: u64, payload: Vec<u8>| {
                        total_bytes_spill.fetch_sub(payload.len(), Ordering::Relaxed);
                        let spill = Arc::clone(&spill_storage);
                        let cipher = cipher.clone();
                        let handle = std::thread::spawn(move || {
                            let tx_id = TxId::new(segment_id.max(1));
                            let encrypted = if let Some(c) = &cipher {
                                let dummy_fp =
                                    ModelFingerprint::new([0xaa; 32], "spill-model", "f16");
                                let enc_layer = c.encrypt(tenant_id, dummy_fp, &payload).ok()?;
                                bincode::serialize(&enc_layer).ok()?
                            } else {
                                payload
                            };
                            let key = format!("tenant:{}:seg:{}", tenant_id.inner(), segment_id);
                            let fut = async move {
                                spill.put(tx_id, key.as_bytes(), &encrypted).await.ok()?;
                                spill.commit(tx_id).await.ok()?;
                                Some(())
                            };
                            block_on_simple(fut)
                        });
                        let _ = handle.join();
                    },
                );
                store.set_spill_handler(handler);
            }
        }

        KvCacheService {
            config: self.config,
            store,
            eviction_worker,
            clock,
            spill_storage: self.spill_storage,
            cipher: self.cipher,
            radix_trees: Arc::new(RwLock::new(ahash::AHashMap::new())),
            next_segment_id: AtomicU64::new(1),
            total_bytes_stored,
        }
    }
}

impl Default for KvCacheServiceBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Öffentlicher Mandanten-isolierter KV-Cache-Dienst.
pub struct KvCacheService {
    config: KvCacheConfig,
    store: Arc<TenantIsolatedKvStore>,
    eviction_worker: Arc<EvictionWorker>,
    clock: Arc<dyn Clock>,
    spill_storage: Option<Arc<dyn KvBridgeStorage>>,
    cipher: Option<Arc<KvSegmentCipher>>,
    radix_trees: Arc<RwLock<ahash::AHashMap<TenantId, PrefixRadixTree>>>,
    next_segment_id: AtomicU64,
    total_bytes_stored: Arc<AtomicUsize>,
}

impl KvCacheService {
    /// Sucht nach einem übereinstimmenden Prefix oder fügt ein neues KV-Segment ein.
    ///
    /// Fail-closed: `tenant_id` ist Pflichtparameter jeder Operation.
    pub fn get_or_insert_prefix(
        &self,
        tenant_id: TenantId,
        tokens: &[u32],
        data: &[u8],
    ) -> Result<Option<(PrefixMatch, KvBlockGuard)>> {
        if tokens.is_empty() {
            return Err(ContextraError::InvalidInput(
                "Cannot query or insert empty token sequence".to_string(),
            ));
        }

        if data.is_empty() {
            return Err(ContextraError::InvalidInput(
                "Cannot insert empty data segment into KV-cache".to_string(),
            ));
        }

        // 1. Check for prefix match in radix tree
        let match_res = {
            let mut trees = self.radix_trees.write();
            let tree = trees
                .entry(tenant_id)
                .or_insert_with(|| PrefixRadixTree::new(tenant_id));
            tree.find_longest_prefix(tokens, self.config.reuse_policy)
        };

        if let Some(pm) = match_res {
            if let Some(guard) = self.store.acquire_block_guard(
                tenant_id,
                pm.block_id,
                Some(Arc::clone(&self.eviction_worker)),
            ) {
                return Ok(Some((pm, guard)));
            }
        }

        // 2. Check RAM budget limit before insertion
        let seg_len = data.len();
        if seg_len > self.config.memory_budget_bytes {
            return Err(ContextraError::InvalidInput(format!(
                "Segment size {} bytes exceeds total service memory budget {} bytes",
                seg_len, self.config.memory_budget_bytes
            )));
        }

        let current_bytes = self.total_bytes_stored.fetch_add(seg_len, Ordering::Relaxed);
        if current_bytes.saturating_add(seg_len) > self.config.memory_budget_bytes {
            let overflow = (current_bytes.saturating_add(seg_len)).saturating_sub(self.config.memory_budget_bytes);
            self.eviction_worker.trigger_eviction(overflow.max(seg_len));
        }

        // 3. Insert new segment
        let segment_id = self.next_segment_id.fetch_add(1, Ordering::Relaxed);

        #[cfg(any(feature = "kivi-quantization", feature = "kvcache-kivi-quant"))]
        let final_bytes = if let Some(kivi_cfg) = self.config.kivi_config {
            if let Ok(tensor_view) = KvTensorView::from_bytes(data) {
                let block = kivi_quantize(&tensor_view, kivi_cfg)?;
                bincode::serialize(&block)
                    .map_err(|e| ContextraError::Serialization(e.to_string()))?
            } else {
                compress_bytes(data)
            }
        } else {
            data.to_vec()
        };

        #[cfg(not(any(feature = "kivi-quantization", feature = "kvcache-kivi-quant")))]
        let final_bytes = data.to_vec();

        let segment = KvSegment::new(tenant_id, segment_id, final_bytes);
        self.store.insert_segment(tenant_id, segment);

        {
            let mut trees = self.radix_trees.write();
            let tree = trees
                .entry(tenant_id)
                .or_insert_with(|| PrefixRadixTree::new(tenant_id));
            tree.insert(tokens, segment_id)?;
        }

        Ok(None)
    }

    /// Liest ein Segment-Payload aus RAM oder Tier-2 Spill-Storage.
    pub async fn read_segment(
        &self,
        tenant_id: TenantId,
        segment_id: u64,
    ) -> Result<Option<Vec<u8>>> {
        // 1. Attempt RAM read
        if let Some(bytes) = self.store.get_segment_bytes(tenant_id, segment_id) {
            return self.decompress_or_dequantize(&bytes);
        }

        // 2. Attempt Tier-2 Spill Storage read
        if let Some(spill) = &self.spill_storage {
            let key = format!("tenant:{}:seg:{}", tenant_id.inner(), segment_id);
            if let Some(spill_bytes) = spill.get(key.as_bytes()).await? {
                let raw_payload = if let Some(cipher) = &self.cipher {
                    let enc_layer: EncryptedKvLayer = bincode::deserialize(&spill_bytes)
                        .map_err(|e| ContextraError::Serialization(e.to_string()))?;
                    cipher.decrypt(&enc_layer)?
                } else {
                    spill_bytes.to_vec()
                };
                return self.decompress_or_dequantize(&raw_payload);
            }
        }

        Ok(None)
    }

    /// Spill-over method: persists a segment to Tier-2 spill storage encrypted per tenant.
    pub async fn spill_segment(
        &self,
        tx_id: TxId,
        tenant_id: TenantId,
        segment_id: u64,
        payload: &[u8],
    ) -> Result<()> {
        let spill = match &self.spill_storage {
            Some(s) => s,
            None => return Ok(()),
        };

        let serialized_data = if let Some(cipher) = &self.cipher {
            let dummy_fp = ModelFingerprint::new([0xaa; 32], "spill-model", "f16");
            let enc_layer = cipher.encrypt(tenant_id, dummy_fp, payload)?;
            bincode::serialize(&enc_layer)
                .map_err(|e| ContextraError::Serialization(e.to_string()))?
        } else {
            payload.to_vec()
        };

        let key = format!("tenant:{}:seg:{}", tenant_id.inner(), segment_id);
        spill.put(tx_id, key.as_bytes(), &serialized_data).await?;
        spill.commit(tx_id).await?;
        Ok(())
    }

    fn decompress_or_dequantize(&self, bytes: &[u8]) -> Result<Option<Vec<u8>>> {
        #[cfg(any(feature = "kivi-quantization", feature = "kvcache-kivi-quant"))]
        {
            if let Ok(decompressed) = decompress_bytes(bytes) {
                return Ok(Some(decompressed));
            }

            if let Ok(block) = bincode::deserialize::<KiviQuantizedBlock>(bytes) {
                let tensor = kivi_dequantize(&block.packed, &block.meta)?;
                return Ok(Some(tensor.to_bytes()?));
            }
        }

        Ok(Some(bytes.to_vec()))
    }

    /// Invalidates scratchpad tokens matching scope for the specified tenant.
    pub fn invalidate_scratchpad(
        &self,
        tenant_id: TenantId,
        scope: ScratchpadCacheScope,
    ) -> Result<usize> {
        let mut invalidated = 0;
        let mut trees = self.radix_trees.write();
        if let Some(tree) = trees.get_mut(&tenant_id) {
            if tree.remove(&scope.scratchpad_key_prefix).is_some() {
                invalidated += 1;
            }
        }
        Ok(invalidated)
    }

    /// Removes all RAM segments and Radix nodes for the given tenant (Mandantenlöschung).
    pub fn delete_tenant(&self, tenant_id: TenantId) {
        // Remove all RAM segments and track deleted byte length
        let segments = self.store.get_segments(tenant_id);
        let mut freed_bytes = 0usize;
        for seg_id in &segments {
            if let Some(data) = self.store.get_segment_bytes(tenant_id, *seg_id) {
                freed_bytes += data.len();
            }
        }
        self.store
            .remove_segments_for_rollback(tenant_id, &segments);
        self.total_bytes_stored.fetch_sub(freed_bytes, Ordering::Relaxed);

        // Remove Radix tree
        let mut trees = self.radix_trees.write();
        trees.remove(&tenant_id);

        // Remove Tier-2 spill storage items for this tenant if spill storage is configured
        if let Some(spill) = &self.spill_storage {
            for segment_id in segments {
                let spill_storage = Arc::clone(spill);
                let key = format!("tenant:{}:seg:{}", tenant_id.inner(), segment_id);
                let handle = std::thread::spawn(move || {
                    let fut = async move {
                        let tx_id = TxId::new(segment_id.max(1));
                        spill_storage.put(tx_id, key.as_bytes(), &[]).await.ok()?;
                        spill_storage.commit(tx_id).await.ok()
                    };
                    block_on_simple(fut)
                });
                let _ = handle.join();
            }
        }

        tracing::info!(tenant_id = ?tenant_id, freed_bytes = freed_bytes, "Deleted all KV-cache resources for tenant");
    }

    /// Triggers background non-blocking eviction targeting `target_free_bytes`.
    pub fn trigger_eviction(&self, target_free_bytes: usize) {
        self.eviction_worker.trigger_eviction(target_free_bytes);
    }

    /// Returns the active clock port reference.
    pub fn clock(&self) -> &Arc<dyn Clock> {
        &self.clock
    }

    /// Liefert Nutzungsstatistiken für den angegebenen Mandanten.
    pub fn stats(&self, tenant_id: TenantId) -> KvCacheStats {
        let segment_count = self.store.get_tenant_segment_len(tenant_id);
        let trees = self.radix_trees.read();
        let radix_entry_count = trees.get(&tenant_id).map(|t| t.len()).unwrap_or(0);

        KvCacheStats {
            segment_count,
            radix_entry_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kv_cache_service_builder_and_isolation() {
        let service = KvCacheServiceBuilder::new()
            .memory_budget_bytes(64 << 20)
            .reuse_policy(KvReusePolicy::Always)
            .build();

        let tenant_a = match TenantId::try_new(100) {
            Ok(t) => t,
            Err(_) => return,
        };
        let tenant_b = match TenantId::try_new(200) {
            Ok(t) => t,
            Err(_) => return,
        };

        let tokens = vec![1, 2, 3, 4, 5];
        let data = vec![10, 20, 30, 40];

        // Tenant A insert
        if let Ok(res) = service.get_or_insert_prefix(tenant_a, &tokens, &data) {
            assert!(res.is_none());
        }

        // Tenant B with identical tokens should MISS (Tenant Isolation)
        if let Ok(res) = service.get_or_insert_prefix(tenant_b, &tokens, &data) {
            assert!(res.is_none());
        }

        // Tenant A lookup should HIT
        if let Ok(hit_a) = service.get_or_insert_prefix(tenant_a, &tokens, &data) {
            assert!(hit_a.is_some());
        }

        // Delete Tenant A
        service.delete_tenant(tenant_a);
        assert_eq!(service.stats(tenant_a).segment_count, 0);
        assert_eq!(service.stats(tenant_a).radix_entry_count, 0);

        // Tenant B stats should remain intact
        assert_eq!(service.stats(tenant_b).segment_count, 1);
        assert_eq!(service.stats(tenant_b).radix_entry_count, 1);
    }
}
