#![forbid(unsafe_code)]

//! Contextra KV-Cache Crate (Ring 1).
//!
//! Stellt den In-Memory LRU-Cache, Eviction-Worker, Tenant-Isolation,
//! Prefix-Radix-Baum, Tiering und Crypto-Shredding Infrastruktur bereit.

#[cfg(feature = "kvcache-attention-eviction")]
pub mod attention_score;
pub mod eviction_worker;
pub mod prefix_store;
#[cfg(feature = "kvcache-kivi-quant")]
pub mod quantize_kivi;
pub mod radix;
pub mod segment;
pub mod store;

#[cfg(feature = "kvcache-attention-eviction")]
pub use attention_score::{
    rank_for_eviction, rank_for_eviction_weighted, AttentionScoreSource, NullAttentionScoreSource,
};
pub use eviction_worker::{emergency_wipe, EvictionWorker};
pub use prefix_store::TenantPrefixKvStore;
#[cfg(feature = "kvcache-kivi-quant")]
pub use quantize_kivi::{pack_kivi_block, unpack_kivi_block, KiviBlockMeta, KiviQuantizedBlock};
pub use radix::{KvBlockGuard, KvReusePolicy, PrefixMatch, PrefixRadixTree};
#[cfg(feature = "content-addressed-kv-cache")]
pub use radix::{
    ContentAddressedKvStore, KvLookupResult, KvSegmentRef, SemanticCacheConfig, SemanticEmbedder,
};
pub use segment::{
    KvSegment, ShreddableSegmentKey, Tier2EncryptedSegment, CURRENT_KV_KEY_DERIVATION_VERSION,
};
pub use store::{SpillHandler, TenantIsolatedKvStore};
