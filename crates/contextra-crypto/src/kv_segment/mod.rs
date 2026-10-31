//! # KV-Segment-Speicher und Sharding (`kv_segment/`)
//!
//! Dieses Subsystem implementiert einen hochgradig nebenläufigen, geschützten In-Memory-Segment-Cache.
//!
//! ## Komponenten
//! - `segment.rs`: Datensegment mit Zeroize-on-Drop.
//! - `store.rs`: Geshardeter Segment-Speicher mit LRU-Eviction.
//! - `eviction_worker.rs`: Asynchroner Hintergrund-Worker zur Speicherbereinigung.

// FILE-CONTEXT
// ZWECK: KV-Cache-Bridge Sicherheitsschicht (KvSegment, Tenant-Isolation, Eviction-Worker, Segment-Verschlüsselung).
// STAND: TS:2026-09-08T00:00:00Z (SESSION: a413a598)

pub mod eviction_worker;
pub mod segment;
pub mod store;

pub use eviction_worker::{emergency_wipe, EvictionWorker};
pub use segment::{KvSegment, CURRENT_KV_KEY_DERIVATION_VERSION};
pub use store::TenantIsolatedKvStore;
