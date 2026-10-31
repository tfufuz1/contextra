//! `Contextra` Core — Typen, Traits und Fehlerbehandlung (Strangler Facade).
//!
//! # Architektur-Rolle (Ring 0 / Legacy-Schicht 0 -> Ring 0-4 Migration)
//!
//! Im Zuge der Migration auf das Ring-Modell (Phase 1b / SPEC §5.2) wurde die
//! monolithische Funktionalität der früheren `contextra-core`-Crate in eigenständige,
//! modulare Ring-0-Crates zerlegt:
//! - [`contextra_types`]: Kanonische Domänenmodelle, Bezeichner (`DocId`, `TxId`),
//!   Ressourcenbudgets, Filter und das zentrale Fehlersystem (`ContextraError`).
//! - [`contextra_ports`]: Abstrakte, `dyn`-kompatible Subsystem-Schnittstellen (Ports)
//!   gemäß der Hexagonalen Architektur (z. B. `StorageEngine`, `Clock`, `Rng`).
//! - [`contextra_mvcc`]: Multi-Version Concurrency Control (Mehrversions-Nebenläufigkeitssteuerung),
//!   Sequenz-Log (`SequenceLog`) und Transaktionspuffer (`TxBuffer`).
//! - [`contextra_wire`]: Zero-Copy-IPC (Inter-Process Communication) via FlatBuffers
//!   und Wire-Nachrichtenadapter.
//!
//! Diese Crate fungiert als abwärtskompatible **Strangler Facade** (Strangler-Muster nach Martin Fowler).
//! Sie leitet alle historischen Typen und Module per Re-Export (`pub use`) weiter, damit
//! bestehende Aufrufer schrittweise ohne Breaking Changes migriert werden können.
//!
//! ## Status und Ring-Zugehörigkeit
//! - **Ring:** Ring 0 (Basis-Typen und Abstraktionen ohne zyklische Abhängigkeiten).
//! - **Status:** Veraltet (`deprecated`). Die Crate soll nach vollständiger Migration aller
//!   Konsumenten auf die spezialisierten Crates entfernt werden (SPEC §5.2 / WP 2.8).
//!
//! ## Konsumenten (Wer nutzt diese Crate?)
//! - [`contextra`]: Haupt-Fassadencrate.
//! - [`contextra-engine`]: Core-Engine zur Ausführung von Speicher-, Such- und Löschoperationen.
//!
//! ## Nebenläufigkeits- und Sperrkonzept
//! In `contextra-core` selbst existieren keine internen Mutexe oder Sperren (`#![forbid(unsafe_code)]`).
//! Die nebenläufige Synchronisation erfolgt vollständig in den re-exportierten Modulen:
//! - `TxBuffer` nutzt partitioniertes/sharded Lock-Striping (über `DashMap`) für sperrenfreie oder
//!   feingranulare Nebenläufigkeit beim Staging von Index-Operationen.
//! - `SnapshotRegistry` verwaltet atomare Sequenznummern (`AtomicU64`) für lock-free Leseschnappschüsse.
//!
//! ## Persistenz und I/O
//! Re-exportiert Schnittstellen (`traits`) zur Interaktion mit WAL (Write-Ahead-Log) und VFS (Virtual File System).
//! E/A-Operationen und `fsync`-Garantien werden von den implementierenden Speichercrates (wie `contextra-store`)
//! bereitgestellt. Fehler werden strikt über den kanonischen Fehlertyp [`ContextraError`] nach dem Fail-Closed-Prinzip
//! propagiert.
//!
//! ## Determinismus und Ports (Regel 3 aus CLAUDE.md)
//! Zeit (`Clock`), Zufall (`Rng`/`EntropyRng`) und ID-Generierung (`IdGenerator`) dürfen in produktiven
//! Pfaden niemals über undeterministische Systemaufrufe (`SystemTime::now`, `thread_rng`) erfolgen,
//! sondern müssen über die in `traits` re-exportierten Ports injiziert werden.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

// ANCHOR[contextra-core:MOD-TYPES-001]
// Re-Export der modularen Ring-0-Crate `contextra-types` (Fehler, Schemata, Domänentypen).
// NOTE: Alle Re-Exports sind mit `#[deprecated]` annotiert, um Downstream-Code zur direkten Nutzung von `contextra_types` zu bewegen.

/// Fehlerbehandlung und Fehlertypen aus `contextra_types`.
///
/// Enthält das zentrale [`contextra_types::error::ContextraError`]-Enum und Hilfstypen.
#[deprecated(
    since = "0.1.0",
    note = "Moved to contextra_types as part of Ring-Modell Phase 1b"
)]
pub use contextra_types::error;

/// Data Transfer Objects (DTOs) für Fehlermeldungen aus `contextra_types`.
///
/// Dient der verlustfreien Serialisierung und IPC-Übertragung von Fehlern.
#[deprecated(
    since = "0.1.0",
    note = "Moved to contextra_types as part of Ring-Modell Phase 1b"
)]
pub use contextra_types::error_dto;

/// Fingerprinting von Modellen und Konfigurationen aus `contextra_types`.
///
/// Berechnet deterministische Prüfsummen für ML-Modelle und Schemata zur Vermeidung von Drift.
#[deprecated(
    since = "0.1.0",
    note = "Moved to contextra_types as part of Ring-Modell Phase 1b"
)]
pub use contextra_types::model_fingerprint;

/// Schemadefinitionen und Versionierung aus `contextra_types`.
///
/// Verwaltet Dokument-ID-Breiten (`DocIdWidth`) und Manifest-Schemaversionen.
#[deprecated(
    since = "0.1.0",
    note = "Moved to contextra_types as part of Ring-Modell Phase 1b"
)]
pub use contextra_types::schema;

/// Tombstone-Prüfungen und Semantik aus `contextra_types`.
///
/// Definiert Bit-Tombstones für logische und physische Löschvorgänge im LSM-Tree/Index.
#[deprecated(
    since = "0.1.0",
    note = "Moved to contextra_types as part of Ring-Modell Phase 1b"
)]
pub use contextra_types::tombstone;

/// Grundlegende Domänentypen aus `contextra_types` (z. B. `DocId`, `TxId`, `DistanceMetric`).
#[deprecated(
    since = "0.1.0",
    note = "Moved to contextra_types as part of Ring-Modell Phase 1b"
)]
pub use contextra_types::types;

// ANCHOR[contextra-core:MOD-PORTS-001]
/// Submodul `traits`, das alle Port-Interfaces aus `contextra_ports` abwärtskompatibel bereitstellt.
///
/// # Hexagonale Architektur
/// Definiert die abstrakten Verträge für Speicher (`StorageEngine`), Vektorindizes (`VectorIndex`),
/// Textsuche (`TextIndex`), Uhren (`Clock`) und Zufallsgeneratoren (`Rng`).
///
/// # Invarianten
/// - Alle Schnittstellen sind `dyn`-kompatibel ausgelegt.
/// - I/O- und Zeitabhängigkeiten werden abstrahiert.
#[deprecated(
    since = "0.1.0",
    note = "Moved to contextra_ports as part of Ring-Modell Phase 1b"
)]
pub mod traits {
    pub use contextra_ports::*;
}

// ANCHOR[contextra-core:MOD-MVCC-001]
// Re-Export der MVCC-Submodule (Multi-Version Concurrency Control) aus `contextra_mvcc`.

/// Sequenzprotokollierung für MVCC-Operationen aus `contextra_mvcc`.
///
/// Zeichnet Änderungen mit monoton steigenden Sequenznummern auf.
#[deprecated(
    since = "0.1.0",
    note = "Moved to contextra_mvcc as part of Ring-Modell Phase 1b"
)]
pub use contextra_mvcc::seq_log;

/// Snapshot-Verwaltung für konsistente Lesezugriffe aus `contextra_mvcc`.
///
/// Ermöglicht Point-in-Time-Reads ohne Blockieren von Schreibtransaktionen.
#[deprecated(
    since = "0.1.0",
    note = "Moved to contextra_mvcc as part of Ring-Modell Phase 1b"
)]
pub use contextra_mvcc::snapshot;

/// Transaktions-Staging-Puffer aus `contextra_mvcc`.
///
/// Puffert uncommitted Schreiboperationen isoliert pro Transaktion.
#[deprecated(
    since = "0.1.0",
    note = "Moved to contextra_mvcc as part of Ring-Modell Phase 1b"
)]
pub use contextra_mvcc::tx_buffer;

// ANCHOR[contextra-core:MOD-WIRE-001]
/// IPC-Modul (Inter-Process Communication) via `contextra_wire` für Rückwärtskompatibilität.
///
/// Ermöglicht binäre Zero-Copy-Kommunikation und Serialisierung über FlatBuffers-Protokolle.
#[deprecated(
    since = "0.1.0",
    note = "Moved to contextra_wire as part of Ring-Modell Phase 1b"
)]
pub mod ipc {
    pub use contextra_wire::*;
}

// ANCHOR[contextra-core:REEXPORT-ITEMS-001]
// Direkte Re-Exports von Kernstrukturen für bestehende Aufrufer

/// Re-Export von Typen für das Sequenz-Log aus `contextra_mvcc::seq_log`.
///
/// - [`SequenceLog`]: Monoton wachsendes Protokoll zur Vergabe und Nachverfolgung logischer Zeitstempel.
/// - [`SeqLogEntry`]: Ein einzelner Eintrag im Sequenz-Log.
/// - [`SeqLogChange`]: Spezifische Zustandsänderung innerhalb eines Eintrags.
#[deprecated(
    since = "0.1.0",
    note = "Moved to contextra_mvcc::seq_log as part of Ring-Modell Phase 1b"
)]
pub use contextra_mvcc::seq_log::{SeqLogChange, SeqLogEntry, SequenceLog};

/// Re-Export von Snapshot-Strukturen aus `contextra_mvcc::snapshot`.
///
/// - [`SnapshotGuard`]: RAII-Guard (Resource Acquisition Is Initialization), der einen Snapshot aktiv hält.
///   Beim Verwerfen (`drop`) wird der Snapshot deregistriert und die minimale Sequenznummer freigegeben.
/// - [`SnapshotRegistry`]: Thread-sichere Registry zur Erfassung aller aktiven Snapshot-Sequenznummern
///   für die Garbage Collection (GC).
#[deprecated(
    since = "0.1.0",
    note = "Moved to contextra_mvcc::snapshot as part of Ring-Modell Phase 1b"
)]
pub use contextra_mvcc::snapshot::{SnapshotGuard, SnapshotRegistry};

/// Re-Export des Transaktions-Puffers aus `contextra_mvcc::tx_buffer`.
///
/// - [`TxBuffer`]: Sharded In-Memory-Puffer für nicht committete Transaktionen mit automatischem
///   Timeout-Reaping verwaister Transaktionen.
/// - [`IndexOp`]: Aufzählung von Index-Modifikationen (z. B. `Insert`, `Delete`).
#[deprecated(
    since = "0.1.0",
    note = "Moved to contextra_mvcc::tx_buffer as part of Ring-Modell Phase 1b"
)]
pub use contextra_mvcc::tx_buffer::{IndexOp, TxBuffer};

/// Globale Re-Exports aller Ports und Traits aus `contextra_ports`.
///
/// Stellt zentrale Traits wie `StorageEngine`, `VectorIndex`, `TextIndex`, `Clock` direkt im Root bereit.
#[deprecated(
    since = "0.1.0",
    note = "Moved to contextra_ports as part of Ring-Modell Phase 1b"
)]
pub use contextra_ports::*;

/// Re-Export des kanonischen Fehlertyps [`ContextraError`] und des zugehörigen [`Result`]-Alias.
///
/// Jeder Fehler im Gesamtsystem wird in dieses zentrale Enum überführt (Fail-Closed).
#[deprecated(
    since = "0.1.0",
    note = "Moved to contextra_types::error as part of Ring-Modell Phase 1b"
)]
pub use contextra_types::error::{ContextraError, Result};

/// Re-Export des serialisierbaren Fehler-DTOs [`ContextraErrorDto`].
///
/// Dient der Weiterleitung strukturierter Fehler über Prozessgrenzen hinweg.
#[deprecated(
    since = "0.1.0",
    note = "Moved to contextra_types::error_dto as part of Ring-Modell Phase 1b"
)]
pub use contextra_types::error_dto::ContextraErrorDto;

/// Re-Export von [`ModelFingerprint`] zur eindeutigen Kennzeichnung von Einbettungs- und Sprachmodellen.
#[deprecated(
    since = "0.1.0",
    note = "Moved to contextra_types::model_fingerprint as part of Ring-Modell Phase 1b"
)]
pub use contextra_types::model_fingerprint::ModelFingerprint;

/// Re-Export von Schematypen aus `contextra_types::schema`.
///
/// - [`DocIdWidth`]: Bitbreite von Dokument-IDs (z. B. 64-Bit oder 128-Bit).
/// - [`ManifestSchemaVersion`]: Versionierung von Datenbank- und Index-Manifesten.
#[deprecated(
    since = "0.1.0",
    note = "Moved to contextra_types::schema as part of Ring-Modell Phase 1b"
)]
pub use contextra_types::schema::{DocIdWidth, ManifestSchemaVersion};

/// Re-Export von Tombstone-Typen zur Löschmarkierung aus `contextra_types::tombstone`.
///
/// - [`SeqBitTombstone`]: Bit-codierte Löschmarkierung verknüpft mit einer Sequenznummer.
/// - [`TombstoneSemanticsCheck`]: Prüflogik zur Validierung der Tombstone-Integrität.
#[deprecated(
    since = "0.1.0",
    note = "Moved to contextra_types::tombstone as part of Ring-Modell Phase 1b"
)]
pub use contextra_types::tombstone::{SeqBitTombstone, TombstoneSemanticsCheck};
