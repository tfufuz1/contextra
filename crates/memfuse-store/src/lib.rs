//! `MemFuse` Store — LSM-Tree based storage engine.
//!
//! Provides persistent key-value storage with WAL, `MemTable`,
//! `SSTable`, and background compaction.
//!
//! # Checkpoint-Architektur
//! `memfuse-store` enthält ein lokales, crate-internes Checkpointing (`pub(crate) mod checkpoint`).
//! Dieses dient ausschließlich als internes MVCC-Snapshot-Pinning (gekoppelt an `SnapshotRegistry`)
//! und darf niemals von außerhalb dieses Crates verwendet werden.
//! Die öffentliche, benannte Checkpoint-API gemäß ADR-011 ("Consolidated Checkpoint Subsystem Architecture")
//! befindet sich im Crate `memfuse-checkpoint`.

// INVARIANT: LSM-Tree Storage Engine (Triebwerk — Layer 1).
// DATEN-PFAD: Client → TxBuffer → WAL → MemTable → SSTable → Compaction
// INVARIANTE: tokio::fs für Metadaten/Lifecycle, std::fs::File ausschließlich innerhalb spawn_blocking für Block-Level Random-Access.
// ANCHOR[INTEGRATION:STO-001] STATUS:DONE (TS:2026-09-16T16:28:29Z)
// REVIEW-PASS[1/2] Systematischer Tiefen-Audit von memfuse-store (PRÜFER-KONTEXT: FRESH) (TS: 2026-09-13T01:33:57Z) (SESSION: 60ca322c)
// REVIEW-PASS[2/2] Unabhängiges SDLC Phase 3 Review von memfuse-store (PRÜFER-KONTEXT: FRESH) (TS: 2026-09-16T16:28:29Z) (SESSION: 1cd824d8)
// MODUL-HIERARCHIE: lsm.rs orchestriert, memtable/wal/sstable sind Bausteine.

// INTENT: deny unsafe_code except Windows Win32 API calls for file permissions
// BEGRÜNDUNG: Sovereign Core Doctrine mandates zero unsafe outside `memfuse-index`,
// except Windows ACL security programming (`SetNamedSecurityInfoW`, etc.).
#![deny(unsafe_code)]
#![allow(unexpected_cfgs)]

#[cfg(not(loom))]
pub(crate) mod checkpoint;
#[cfg(not(loom))]
pub mod compaction;
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
// Tracking-Issue: [ISSUE-NUMMER]

#[cfg(not(loom))]
pub use compaction::{CompactionConfig, CompactionEngine};
#[cfg(not(loom))]
pub use lsm::{LsmConfig, LsmStorage};
#[cfg(not(loom))]
pub use manifest::{Manifest, ManifestEntry};
#[cfg(not(loom))]
pub use system_pressure::{PressureLevel, SystemPressure, SystemPressureMonitor};
#[cfg(not(loom))]
pub use tenant_codec::TenantKeyCodec;
