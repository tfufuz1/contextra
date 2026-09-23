#![forbid(unsafe_code)]

//! Contextra KV-Cache Crate (Ring 1).
//!
//! Stellt den In-Memory LRU-Cache, Eviction-Worker, Tenant-Isolation,
//! Prefix-Radix-Baum, Tiering und Crypto-Shredding Infrastruktur bereit.

pub mod eviction_worker;
pub mod radix;
pub mod segment;
pub mod store;

pub use eviction_worker::{emergency_wipe, EvictionWorker};
pub use radix::{KvBlockGuard, KvReusePolicy, PrefixMatch, PrefixRadixTree};
pub use segment::{
    KvSegment, ShreddableSegmentKey, Tier2EncryptedSegment, CURRENT_KV_KEY_DERIVATION_VERSION,
};
pub use store::{SpillHandler, TenantIsolatedKvStore};
