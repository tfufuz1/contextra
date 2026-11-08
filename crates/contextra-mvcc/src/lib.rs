#![deny(dead_code)]
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
//!
//! ## Sperrenordnung & Invarianten (INV-LOCK-ORDER-1)
//!
//! Um Verklemmungen (Deadlocks) bei nebenläufigem Zugriff deterministisch auszuschließen,
//! gilt für `contextra-mvcc` und die darüberliegende Engine eine strikte, totale Sperrenhierarchie:
//!
//! 1. **Document / Key Lock**: Externe Zeilen- bzw. Dokumentsperre (z. B. striped Shard-Locks).
//! 2. **TxBuffer Shards**: Partitionierte Staging-Shards (`shards[0..N-1]`). Es wird **niemals**
//!    mehr als ein Shard-Lock gleichzeitig erworben. Operationen, die alle Shards durchlaufen
//!    (z. B. `reap_orphans`, `min_read_snapshot`), sperren streng sequentiell Shard für Shard
//!    in aufsteigender Index-Reihenfolge (0 bis $N-1$) unter kurzem Lock und verschachteln niemals Locks.
//! 3. **SnapshotRegistry**: Zentraler Mutex für die aktive Snapshot-Map. Sperre wird nur für
//!    minimale O(1)-Operationen gehalten (Eintrag einfügen/entfernen, atomares Minimum aktualisieren)
//!    und ruft keinen externen Code auf.
//! 4. **SequenceLog / SSI (CommittedWrites)**: Feingranularer `parking_lot::RwLock` zur Verfolgung
//!    bestätigter Schreibvorgänge und Phantom-Präfixe.
//!
//! **Regel (INV-LOCK-ORDER-1):** Kein Sperr-Guard wird über `.await`-Punkte hinweg gehalten
//! (`contextra-mvcc` ist vollkommen synchron und frei von Async-Runtimes wie Tokio).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub use contextra_types::{
    error::{self, ContextraError, Result},
    tombstone::{SeqBitTombstone, TombstoneSemanticsCheck},
    types::{self, DocId, TenantId, TxId, TOMBSTONE_BIT},
};

pub mod seq_log;
pub mod snapshot;
pub mod ssi;
pub mod tx_buffer;

pub use seq_log::{SeqLogChange, SeqLogEntry, SequenceLog};
pub use snapshot::{SnapshotGuard, SnapshotRegistry};
pub use ssi::{PruningBlockerInfo, ReadSet, SequenceLogSsiValidator, SsiValidator};
pub use tx_buffer::{IndexOp, TxBuffer};
