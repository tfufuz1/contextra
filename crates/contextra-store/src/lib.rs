//! # contextra-store (Ring 1)
//!
//! Die Crate `contextra-store` implementiert das persistente LSM-Tree-Speichertriebwerk
//! (Log-Structured Merge-tree) für die Contextra-Plattform. Sie stellt die primäre
//! Datenspeicherebene bereit und bietet atomare Schreibvorgänge, Konsistenz
//! über Snapshots sowie Ausfallsicherheit durch Write-Ahead-Logging (WAL).
//!
//! ## Architektur & Datenpfad
//! - **Client / Transaktion:** Sendet atomare Schreiboperationen (Put, Delete) über Transaktionspuffer.
//! - **WAL (Write-Ahead-Log):** Persistiert alle Mutationen sequentiell auf Festplatte inklusive
//!   CRC32C- und HMAC-SHA256-Integritätsprüfung, bevor sie im RAM bestätigt werden.
//! - **MemTable:** Haltebereich im Arbeitsspeicher (geschardete BTreeMap), der schnelle
//!   Suchen und Einfügungen bei geordneten Sequenznummern ermöglicht.
//! - **Flush & SSTable:** Bei Erreichen von Größenschwellen wird die aktive MemTable eingefroren
//!   und asynchron als unveränderliche SSTable (Sorted String Table, Format V4 Standard) auf Platte geschrieben.
//! - **Compaction:** Hintergrundprozess, der SSTables hierarchisch (Leveled / Adaptive) zusammenführt,
//!   Tombstones bereinigt und Lese- sowie Speicherverstärkung minimiert.
//!
//! ## Sicherheitsinvariante
//! Die gesamte Crate erzwingt `#![forbid(unsafe_code)]` in Produktion. Es existieren keinerlei
//! `unsafe`-Blöcke im Produktionscode.
//!
//! ## Ring-Hierarchie & Abhängigkeiten
//! - **Ring 1 / Klasse B:** Kernkomponente des Storage-Bundles (`types`, `ports`, `mvcc`, `crypto`, `sys`, `store`).
//! - **Eingehende Abhängigkeiten:** Wird genutzt von `contextra-engine`, `contextra-db`,
//!   `contextra-checkpoint`, `contextra-agent` und der übergeordneten Fassade `contextra`.
//! - **Ausgehende Abhängigkeiten:** Nutzt `contextra-types`, `contextra-ports`, `contextra-crypto`,
//!   `contextra-sys`, `contextra-mvcc`.

#![forbid(unsafe_code)]
#![deny(dead_code)]
#![allow(unexpected_cfgs)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::todo,
    clippy::unimplemented,
    clippy::unreachable,
    clippy::indexing_slicing
)]
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing
    )
)]

#[cfg(not(loom))]
pub mod compaction;
pub mod engine;
pub mod fsync;
pub mod key_source;
#[cfg(not(loom))]
pub mod lsm;
#[cfg(not(loom))]
pub mod manifest;
#[cfg(not(loom))]
pub mod memtable;
#[cfg(not(loom))]
pub mod scanner;
#[cfg(not(loom))]
pub mod sstable;
#[cfg(not(loom))]
pub mod system_pressure;
#[cfg(not(loom))]
pub mod tenant_codec;
pub(crate) mod util;
pub mod wal;

#[cfg(not(loom))]
pub use key_source::{KeyMaterialSource, OsKeyMaterialSource, StaticKeyMaterialSource};
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

pub mod io_seam;
pub use io_seam::{RealFs, RealFsFile, StoreIo};
