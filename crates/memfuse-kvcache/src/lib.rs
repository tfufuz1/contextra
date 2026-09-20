#![cfg_attr(not(test), forbid(unsafe_code))]

//! MemFuse KV-Cache Crate (Ring 1).
//!
//! Stellt den In-Memory LRU-Cache, Eviction-Worker, Tenant-Isolation
//! und Prefix-Caching-Infrastruktur bereit.

pub mod eviction_worker;
pub mod segment;
pub mod store;

pub use eviction_worker::{emergency_wipe, EvictionWorker};
pub use segment::{KvSegment, CURRENT_KV_KEY_DERIVATION_VERSION};
pub use store::{SpillHandler, TenantIsolatedKvStore};
