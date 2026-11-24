# AGENTS.md — `contextra-kvcache`

> Werkzeug/Crate-Kontext für Agenten. Der obere Teil ist generiert; manuelle Notizen unterhalb ergänzen.

<!-- BEGIN GENERATED: crate_inventory.py -->

## Inventar (generiert)

- **Pfad:** `crates/contextra-kvcache` · **Ring:** 1 · **Version:** 0.1.0
- **Beschreibung:** Ring 1 Prefix-Radix tree, KV-Block cache, tenant-isolated memory store and tiering for Contextra
- **src:** 21 Dateien, 5756 LOC · **unsafe:** 0 Vorkommen, Policy `forbid`
- **Interne Abhängigkeiten (normal, 4):** `contextra-crypto`, `contextra-env`, `contextra-ports`, `contextra-types`
- **Interne Dev/Build-Abhängigkeiten (3):** `contextra-crypto`, `contextra-ports`, `contextra-types`
- **Externe Abhängigkeiten (normal, 11):** `ahash`, `bincode`, `blake3`, `bytes`, `lru`, `parking_lot`, `serde`, `serde_json`, `thiserror`, `tracing`, `zeroize`
- **Reverse-Abhängigkeiten (0):** –
- **Kopplung an Hub-Crates:** ausgehend `contextra-ports`, `contextra-types`; eingehend –
- **Features (12):** `content-addressed-kv-cache`, `crypto-compat`, `default`, `exp-content-addressed`, `exp-kivi`, `kivi-quantization`, `kv-encryption`, `kvcache-kivi-quant`, `ports-compat`, `std`, `types-compat`, `workspace`

### Öffentliche API (grob, aus src/)
- struct (48): `BoundedScratchpadInvalidator`, `ContentAddressedKvStore`, `CryptoKeyAead`, `CryptoSegmentCipher`, `EmbeddingProviderSemanticEmbedder`, `EncryptedSegmentPayload`, `EvictionWorker`, `InMemoryKvSpillStorage`, `InvalidationBudget`, `KiviBlockMeta`, `KiviQuantizeConfig`, `KiviQuantizedBlock`, `KvBlockGuard`, `KvCacheConfig`, `KvCachePluginConfig` … (+33)
- enum (5): `CacheDirective`, `KvCacheError`, `KvLookupResult`, `KvReusePolicy`, `KvSegmentContent`
- trait (9): `AttentionScoreSource`, `Clock`, `KvCacheEngine`, `KvSpillStorage`, `Rng`, `ScratchpadInvalidator`, `SegmentAead`, `SegmentCipher`, `SemanticEmbedder`
- fn (127): `acquire_block_guard`, `acquire_guard`, `active_refs`, `allocate`, `allocated_blocks`, `allocated_bytes`, `as_bytes`, `as_u64`, `attention_exporter`, `attention_source`, `block_id`, `build`, `cipher`, `clear`, `clear_all` … (+112)
- type (4): `BoxFuture`, `Result`, `SegmentKey`, `SpillHandler`
- const (6): `CURRENT_KV_KEY_DERIVATION_VERSION`, `DEFAULT_BYTE_BUDGET_PER_TENANT`, `DEFAULT_SEGMENT_CAPACITY_PER_TENANT`, `DEFAULT_SHARD_COUNT`, `MAX_PINNED_BYTES_PER_TENANT`, `SYSTEM`
- mod (19): `adapters`, `attention_score`, `cipher`, `compat`, `contract`, `error`, `eviction_worker`, `invalidation`, `kv_encryption`, `prefix_store`, `quantize_kivi`, `radix`, `scanner`, `scratchpad_invalidation`, `segment` … (+4)
- pub use: `adapters::*`, `adapters::{ EmbeddingProviderSemanticEmbedder, KvCachePluginConfig, KvCachePluginManifest, StorageEngineKvBridge, }`, `attention_score::{ rank_for_eviction, rank_for_eviction_weighted, rank_for_eviction_weighted_for_tenant_nanos, AttentionScoreSource, NullAttentionScoreSource, }`, `cipher::{PlaintextSegmentCipher, SegmentCipher}`, `compat::scanner::{KvCachePruner, KvCacheScanner}`, `compat::{adapters, prefix_store, scratchpad_invalidation}`, `contextra_crypto::{ CryptoError, CryptoKey, EncryptedKvLayer, KvCipher, KvSegmentCipher, ModelFingerprint, }`, `contextra_ports::attention::AttentionExporter`, `contextra_ports::kv_bridge_port::KvBridgeStorage`, `crate::compat::scanner::{KvCachePruner, KvCacheScanner}` … (+17)

Regeneriert mit `python3 tools/crate_inventory.py agents`. Manuelle Notizen stehen außerhalb dieses Blocks.

<!-- END GENERATED: crate_inventory.py -->
