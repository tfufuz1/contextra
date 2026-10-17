// FILE-CONTEXT
// STAND: 2026-10-11T00:00:00Z (SESSION: 0ccaf2ff)
// ZWECK: Crate-Einstiegspunkt, Modul-Hierarchie und Re-Exports der contextra-store Storage Engine
// INVARIANTEN: Strikte Kapselung LSM-interner Details (lsm::guard ist pub(crate)); #![forbid(unsafe_code)]
// NICHT-OFFENSICHTLICH: Snapshot-Pinning nur über contextra-checkpoint Guard; Store-Module für Benches/Tests freigegeben
// SIEHE AUCH: AGENTS.md, docs/audits/AUDIT_contextra-store_2026-10-04.md

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
pub mod blob;
#[cfg(not(loom))]
pub mod compaction;
#[cfg(not(loom))]
pub mod copy_registry;
pub mod engine;
#[cfg(not(loom))]
pub mod erasure_ledger;
#[cfg(not(loom))]
pub mod format;
pub mod kv;
#[cfg(not(loom))]
pub mod lsm;
#[cfg(not(loom))]
pub mod manifest;
#[cfg(not(loom))]
pub mod memtable;
pub mod restore_guard;
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

// TODO(contextra-plan): LSM/WAL Persistenz-Engine Architekturentwurf & Risikobetrachtung [ref:eigenbau-lsm-wal]
// 1. Disk-Physik (fsync-Reihenfolge): Strikte Durchsetzung des atomaren Sequenzmusters `.tmp` Schreiben -> Datei `fsync` -> Atomares `rename` -> Parent Directory `fsync`. Verhindert TOCTOU-Races und unvollständige SSTable/Manifest-Dateien nach abrupten Crashs.
// 2. Group-Commit Scope Reduction: `commit_mutex` wird nur während der Batch-Vorbereitung und Sequenzzuordnung gehalten, vor physischen Disk-I/O-Operationen (`wal.append_batch`) freigegeben und erst zur MemTable-Aktualisierung/Sichtbarkeits-Erweiterung erneut akquiriert. Reduziert Contention und Latenzspitzen unter hoher Schreiblast.
// 3. HMAC-Chains: Fortlaufende V3 HMAC-Kettenverifizierung über WAL-Segmente hinweg in Kombination mit der Manifest High-Water-Mark. Bietet Schutz vor unbeabsichtigter WAL-Kürzung (INV-WAL-TRUNCATION-1) und Manipulationen.
// 4. rollback_to_tx: Rollback-Mechanismus für uncommitted WAL- und MemTable-Einträge bei Abbruch oder Transaktionsfehlern, inklusive korrekter Speicherfreigabe im MemTable-Budget und Wiederherstellung des konsistenten LsmState.

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
pub use restore_guard::StreamingRestoreGuard;
#[cfg(not(loom))]
pub use system_pressure::{PressureLevel, SystemPressure, SystemPressureMonitor};
#[cfg(not(loom))]
pub use tenant_codec::{TenantKeyCodec, TenantScopedStorage};
