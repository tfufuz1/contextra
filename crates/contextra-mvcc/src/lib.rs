#![deny(dead_code)]
//! # contextra-mvcc — Multi-Version Concurrency Control & Transaktionsisolation
//!
//! **Ring:** Ring 0 (Transaktionsisolation & Speicher-Kern)
//! **Klasse:** Kern `[K]` (Normativ nach SPEC §6.1, §6.9 und `capabilities.toml`)
//! **Reifegrad:** `stable`
//! **`unsafe`-Vorkommen:** 0 (`#![forbid(unsafe_code)]`)
//!
//! ## Zweck & Architektur-Rolle
//!
//! `contextra-mvcc` stellt deterministische Concurrency-Control-Primitive für das Gesamtsystem
//! bereit. MVCC (Multi-Version Concurrency Control) ermöglicht gleichzeitige Lese- und
//! Schreibzugriffe ohne gegenseitiges Blockieren: Leser greifen auf unveränderliche, historische
//! Snapshots zu, während Schreiber Änderungen isoliert vorbereiten.
//!
//! ### Kernkomponenten
//! - [`SequenceLog`]: Chronologisches, monotones Sequenzlog für Dokumentänderungen (Einfügungen,
//!   Löschungen mit Bit-Masking für Tombstones) und Sichtbarkeitsfilterung historischer Suchen (`_at`-Familie).
//! - [`SnapshotRegistry`] / [`SnapshotGuard`]: RAII-basierte Snapshot-Verwaltung. Berechnet lock-frei
//!   die minimal aktive Sequenznummer (`min_active_seqno`), um voreilige Garbage Collection (GC) von
//!   Tombstones im Speicher und im LSM-Tree (`contextra-store`) zu verhindern.
//! - [`TxBuffer`]: Gesplitteter (sharded) Transaktionspuffer für zweiphasiges Staging von Index-Operationen,
//!   Absicherung gegen Speicherüberlastung (Byte- und Operations-Budgets) und automatisches Aufräumen verwaister Transaktionen (Orphan Reaping).
//! - [`ReadSet`] / [`SsiValidator`] / [`SequenceLogSsiValidator`]: Serializable Snapshot Isolation (SSI).
//!   Erfasst Lesezugriffe und Präfixe zur Erkennung von Write-Skew-Anomalien und Phantomen bei parallelen Transaktionen.
//!
//! ## Einbindung und Abhängigkeiten
//!
//! ### Abhängigkeiten (nach oben / konsumiert):
//! - `contextra-types`: Grundlegende Domain-Typen (`DocId`, `TxId`, `ContextraError`, `TOMBSTONE_BIT`).
//! - `contextra-ports`: Abstrakte Ports (insbesondere [`contextra_ports::Clock`] für deterministische Zeitmessung).
//! - Externe Bibliotheken: `parking_lot` (performante Mutexe & RwLocks), `ahash` (schnelle Hashing-Algorithmen),
//!   `bytes` (Zero-Copy Puffer), `serde` (Serialisierung der Konfigurationen), `thiserror` und `tracing`.
//!
//! ### Konsumenten (wird eingebunden von):
//! - `contextra-store`: Nutzt `SnapshotRegistry` für Tombstone-GC-Grenzen im LSM-Tree.
//! - `contextra-engine`: Koordiniert Transaktionsabläufe und zweiphasiges Commit-Staging über `TxBuffer` und SSI-Validierung.
//! - `contextra-checkpoint`: Verwendet Snapshot-Pins für konsistente Sicherungspunkte.
//! - `contextra-vector`: Nutzt `SequenceLog` zur Sichtbarkeitsprüfung in Vektor-Indizes.
//! - `contextra-core`: Re-exportiert Transaktionsprimitive.
//!
//! ## Nebenläufigkeit & Determinismus (Regel 3 aus CLAUDE.md)
//!
//! Sämtliche Zeit-, Zufalls- und ID-Generierungen erfolgen ausschließlich über Ports (`Clock`).
//! Es wird kein direkter Zugriff auf `std::time::Instant::now()` im Produktionscode verwendet,
//! wodurch alle Abläufe in Tests (z. B. via `contextra_testkit::ManualClock`) und Loom vollständig deterministisch sind.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

// ANCHOR[contextra-mvcc:LIB-001]
// Re-Export von Kerntypen aus contextra_types für nahtlose Integration in höhere Ringe.
pub use contextra_types::*;

/// Modul für das chronologische Sequenzlog und Sichtbarkeitsprüfung.
pub mod seq_log;
/// Modul für die Snapshot-Registry und RAII-Snapshot-Guards.
pub mod snapshot;
/// Modul für Serializable Snapshot Isolation (SSI), Read-Sets und Konflikt-Validierung.
pub mod ssi;
/// Modul für den nebenläufigen, geshardeten Transaktionspuffer.
pub mod tx_buffer;

// Re-Exports der wichtigsten Typen für die einfache Verwendung durch Konsumenten
pub use seq_log::{SeqLogChange, SeqLogEntry, SequenceLog};
pub use snapshot::{SnapshotGuard, SnapshotRegistry};
pub use ssi::{PruningBlockerInfo, ReadSet, SequenceLogSsiValidator, SsiValidator};
pub use tx_buffer::{IndexOp, TxBuffer};
