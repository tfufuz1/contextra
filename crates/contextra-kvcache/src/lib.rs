#![forbid(unsafe_code)]

//! # Contextra KV-Cache (Ring 1)
//!
//! ANCHOR[contextra-kvcache:LIB-001]
//!
//! Dieses Modul bildet die zentrale Schnittstelle des `contextra-kvcache`-Crates.
//!
//! ## Was ist ein KV-Cache?
//! In LLMs (Large Language Models = großen Sprachmodellen) bezeichnet der **KV-Cache**
//! (Key-Value-Cache) zwischengespeicherte Zwischenzustände von Aufmerksamkeitsmatrizen
//! (Attention-Keys und Values) vorheriger Token. Wenn mehrere Anfragen denselben Prompt-Präfix
//! teilen (z. B. System-Prompts oder Dokumentenkontexte), kann durch Wiederverwendung des Caches
//! massive Rechenzeit eingespart werden (Prompt Caching).
//!
//! ## Architektur & Ring-Zugehörigkeit
//! - **Ring 1 (State & Storage)**:
//!   Die Crate verwaltet kurzlebige und transiente Speicherstrukturen im RAM sowie verschlüsselte
//!   Segmente im Rahmen des Tiering (Auslagerung auf sekundäre Speicher).
//! - **Erzwungene Sicherheit**: Es gilt `#![forbid(unsafe_code)]` – kein `unsafe` im gesamten Crate.
//! - **Abhängigkeiten (eingehend)**: Keine Workspace-Crate bindet `contextra-kvcache` aktuell
//!   direkt als Abhängigkeit ein (wird jedoch im Workspace mitgebaut und getestet).
//! - **Abhängigkeiten (ausgehend)**:
//!   - `contextra-types`: Domänentypen wie `TenantId` (Mandanten-Identifikator).
//!   - `contextra-ports`: Abstrakte Schnittstellen (z. B. `KvPrefixStore`, `Clock`, `StorageEngine`).
//!   - `contextra-crypto`: Kryptographische Primitiven für AEAD-Verschlüsselung und Crypto-Shredding.
//!
//! ## Kernkomponenten
//! 1. [`radix`]: Ein Prefix-Radix-Baum zur schnellen Identifikation geteilter Token-Sequenzen.
//! 2. [`store`]: [`TenantIsolatedKvStore`] als mandantengetrennter In-Memory-LRU-Cache.
//! 3. [`segment`]: [`KvSegment`] und [`ShreddableSegmentKey`] für kryptographisch geschützte Segmente.
//! 4. [`eviction_worker`]: Asynchroner Hintergrund-Worker zur Speicherbereinigung und Verdrängung.
//! 5. [`attention_score`]: Gewichtete Verdrängungsstrategien basierend auf Attention-Scores.
//! 6. [`scratchpad_invalidation`]: Gezielte Invalidierung von Scratchpads bei Agenteninteraktionen.
//! 7. `quantize_kivi`: Optionale KIVI 2-Bit Quantisierung für maximale Speichereffizienz (feature-gated).
//! 8. [`adapters`]: Brückenadapter zu Embedding-Providern und Storage-Engines.
//! 9. [`service`]: Hochrangige Fassade [`KvCacheService`] zur einfachen Verwendung.

pub mod adapters;
pub mod attention_score;
pub mod eviction_worker;
pub mod prefix_store;
#[cfg(any(feature = "kivi-quantization", feature = "kvcache-kivi-quant"))]
pub mod quantize_kivi;
pub mod radix;
pub mod scratchpad_invalidation;
pub mod segment;
pub mod service;
pub mod store;

pub use adapters::{
    EmbeddingProviderSemanticEmbedder, KvCachePluginConfig, KvCachePluginManifest,
    StorageEngineKvBridge,
};
pub use attention_score::{
    rank_for_eviction, rank_for_eviction_weighted, AttentionScoreSource, NullAttentionScoreSource,
};
pub use eviction_worker::{emergency_wipe, EvictionWorker};
pub use prefix_store::TenantPrefixKvStore;
#[cfg(any(feature = "kivi-quantization", feature = "kvcache-kivi-quant"))]
pub use quantize_kivi::{
    compress_bytes, decompress_bytes, kivi_dequantize, kivi_quantize, pack_kivi_block,
    unpack_kivi_block, KiviBlockMeta, KiviQuantizeConfig, KiviQuantizedBlock, KvTensorView,
};
#[cfg(feature = "content-addressed-kv-cache")]
pub use radix::{
    ContentAddressedKvStore, KvLookupResult, KvSegmentRef, SemanticCacheConfig, SemanticEmbedder,
};
pub use radix::{KvBlockGuard, KvReusePolicy, PrefixMatch, PrefixRadixTree};
pub use scratchpad_invalidation::{
    PrefixStoreScratchpadInvalidator, ScratchpadCacheScope, ScratchpadInvalidator,
};
pub use segment::{
    KvSegment, KvSegmentContent, ShreddableSegmentKey, Tier2EncryptedSegment,
    CURRENT_KV_KEY_DERIVATION_VERSION,
};
pub use service::{KvCacheConfig, KvCacheService, KvCacheServiceBuilder, KvCacheStats};
pub use store::{SpillHandler, TenantIsolatedKvStore};
