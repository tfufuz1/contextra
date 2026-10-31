#![deny(dead_code)]
// ANCHOR[contextra-mvcc:LIB-ROOT]
//! # `Contextra` MVCC — Multi-Version Concurrency Control, Sequence Log, Transaction Staging und SSI-Validierung.
//!
//! Multi-Version Concurrency Control (MVCC, Mehrversions-Nebenläufigkeitssteuerung) ermöglicht es
//! Lese- und Schreiboperationen gleichzeitig ohne gegenseitiges Blockieren auszuführen.
//! Leser sehen konsistente Snapshots (Schnappschüsse) zu einem logischen Zeitpunkt,
//! während Schreiber isoliert neue Versionen im Staging-Puffer vorbereiten.
//!
//! ## Architektur-Rolle (Ring 0 — Kernkomponente)
//!
//! `contextra-mvcc` gehört zur innersten Systemschicht (Ring 0, Basis/Kern). Es bietet fundamentale
//! Isolationsgarantien für relationale und vektorielle Indexstrukturen und besitzt keinerlei
//! Kenntnisse über Festplatten-Layouts oder Netzwerkprotokolle.
//!
//! ### Wer nutzt diese Crate? (Eingehende Abhängigkeiten)
//! - `contextra-store`: Nutzt MVCC-Sequenzen für WAL-Synchronisation und Compaction-GC-Pruning.
//! - `contextra-engine`: Koordiniert Transaktionen, ReadSets und SSI-Validierung vor Commits.
//! - `contextra-vector`: Führt snapshot-isolierte Vektorsuchen (`_at`) durch.
//! - `contextra-checkpoint`: Schützt Snapshot-Zustände bei Backup-Erstellung.
//! - `contextra-core`: Exportiert Kern-MVCC-Fähigkeiten für übergeordnete Abfrage-Pipelines.
//!
//! ### Wovon hängt diese Crate ab? (Ausgehende Abhängigkeiten)
//! - `contextra-types`: Domänentypen wie `TxId`, `DocId`, `ContextraError`.
//! - `contextra-ports`: Abstrakte Schnittstellen wie `Clock` (Zeitport zur deterministischen Testbarkeit).
//! - Bibliotheken dritter Parteien: `parking_lot` (effiziente Mutexe/RwLocks), `ahash` (schnelle Hashing-Algorithmen).
//!
//! ### Kernkomponenten
//! - [`SequenceLog`]: Monotone Versionsverfolgung von Dokument-Änderungen für Snapshot-Suchen.
//! - [`SnapshotRegistry`] / [`SnapshotGuard`]: RAII-geschützte Schnappschüsse, die minimale aktive Sequenzen garantieren.
//! - [`TxBuffer`]: Nach Transaktions-ID partitionierter Staging-Puffer für atomare 2-Phase-Commit-Vorbereitungen.
//! - [`ReadSet`] / [`SsiValidator`] / [`SequenceLogSsiValidator`]: Serializable Snapshot Isolation (SSI)
//!   zur Erkennung von Write-Skew-Anomalien und Phantom-Reads bei Transaktionen.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub use contextra_types::*;

pub mod seq_log;
pub mod snapshot;
pub mod ssi;
pub mod tx_buffer;

pub use seq_log::{SeqLogChange, SeqLogEntry, SequenceLog};
pub use snapshot::{SnapshotGuard, SnapshotRegistry};
pub use ssi::{PruningBlockerInfo, ReadSet, SequenceLogSsiValidator, SsiValidator};
pub use tx_buffer::{IndexOp, TxBuffer};
