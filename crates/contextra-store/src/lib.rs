#![deny(dead_code)]
//! `Contextra` Store — LSM-Tree based storage engine.
//!
//! Provides persistent key-value storage with WAL, `MemTable`,
//! `SSTable`, and background compaction.

// INVARIANT: LSM-Tree Storage Engine (Triebwerk — Layer 1).
// DATEN-PFAD: Client → TxBuffer → WAL → MemTable → SSTable → Compaction
// INVARIANTE: tokio::fs für Metadaten/Lifecycle, std::fs::File ausschließlich innerhalb spawn_blocking für Block-Level Random-Access.
// ANCHOR[INTEGRATION:STO-001] STATUS:RESOLVED (TS:2026-08-24T00:00:00Z)
// REVIEW-PASS[1/2] Systematischer Tiefen-Audit von contextra-store (PRÜFER-KONTEXT: FRESH) (TS: 2026-09-13T01:33:57Z) (SESSION: 60ca322c)
// MODUL-HIERARCHIE: lsm.rs orchestriert, memtable/wal/sstable sind Bausteine.

#![forbid(unsafe_code)]
#![allow(unexpected_cfgs)]

#[cfg(not(loom))]
pub mod compaction;
pub mod kv;
#[cfg(not(loom))]
pub mod lsm;
#[cfg(not(loom))]
pub mod manifest;
#[cfg(not(loom))]
pub mod memtable;
#[cfg(not(loom))]
pub mod sstable;
#[cfg(not(loom))]
pub mod system_pressure;
#[cfg(not(loom))]
pub mod tenant_codec;
pub(crate) mod util;
pub mod wal;

// WP-4.1 (UNIMPLEMENTED): mmap-basierter SSTable-Zugriff für Out-of-Core-Daten.
// Aktuell: reguläres tokio::fs/std::fs File-I/O in sstable.rs.
// Tracking-Issue: TODO(owner): Issue anlegen
// INVARIANTE: LSM-Kern bleibt blind gegenüber Indexstrukturen.

#[cfg(not(loom))]
pub use compaction::{
    adaptive::{
        AdaptiveCompactionPlan, AdaptiveCompactionPlanner, CompactionStrategy,
        CostBasedAdaptivePlanner, WorkloadMetrics, WorkloadMetricsSnapshot,
    },
    CompactionConfig, CompactionEngine, MergeOperator, TtlMetadata,
};
#[cfg(not(loom))]
pub use lsm::{
    CommittedBatch, LsmConfig, LsmStorage, ObserverRegistry, WalEntryRef, WalObserver, WriteOrigin,
    DEFAULT_MAX_OBSERVER_LATENCY,
};
#[cfg(not(loom))]
pub use manifest::{Manifest, ManifestEntry};
#[cfg(not(loom))]
pub use system_pressure::{PressureLevel, SystemPressure, SystemPressureMonitor};
#[cfg(not(loom))]
pub use tenant_codec::{TenantKeyCodec, TenantScopedStorage};
