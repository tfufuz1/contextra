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
//! - **Ring 1 (E1):** Domänenbaustein. Kern-Closure: `ids`, `env`, `sys`, `durable-fs`, `crypto`.
//! - **Ausgehende Abhängigkeiten:** `contextra-ids`, `contextra-env`, `contextra-sys`,
//!   `contextra-durable-fs`, `contextra-crypto`, `contextra-version`. Keine E2-/E3-Abhängigkeiten (R1, R2).
//! - **Eingehende Abhängigkeiten:** E2-Schicht (`contextra-db`) über die öffentliche API.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
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

pub mod error;
pub mod traits;
pub use error::{StoreError, StoreResult};
pub mod budget;
pub mod snapshot_pins;
pub(crate) mod staging;
pub mod stats;

#[cfg(not(loom))]
pub mod compaction;
#[cfg(not(loom))]
pub mod fsync;
pub mod key_source;
#[cfg(not(loom))]
pub mod lsm;
#[cfg(not(loom))]
pub mod manifest;
#[cfg(not(loom))]
pub mod memtable;
#[cfg(not(loom))]
#[cfg(not(loom))]
pub mod sstable;
#[cfg(not(loom))]
pub mod system_pressure;
#[cfg(not(loom))]
pub mod tenant_codec;
pub mod util;
pub mod wal;

pub use snapshot_pins::SnapshotPins;
pub use util::{RecordAuthenticator, RecordSealer, SealedEnvelope};

#[cfg(not(loom))]
pub use compaction::{
    adaptive::{
        AdaptiveCompactionPlan, AdaptiveCompactionPlanner, CompactionStrategy,
        CostBasedAdaptivePlanner, WorkloadMetrics, WorkloadMetricsSnapshot,
    },
    CompactionConfig, CompactionEngine, MergeOperator, TtlMetadata,
};
#[cfg(not(loom))]
pub use key_source::{KeyMaterialSource, OsKeyMaterialSource, StaticKeyMaterialSource};
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
pub use io_seam::{RealFs, RealFsFile, StoreIo, Vfs, VfsFile, VfsOpenOptions};
