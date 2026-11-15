#![deny(dead_code)]
//! # `Acta` MVCC — Multi-Version Concurrency Control, Sequence Log, Transaction Staging und SSI-Validierung.
//!
//! Multi-Version Concurrency Control (MVCC, Mehrversions-Nebenläufigkeitssteuerung) ermöglicht es
//! Lese- und Schreiboperationen gleichzeitig ohne gegenseitiges Blockieren auszuführen.
//! Leser sehen konsistente Snapshots (Schnappschüsse) zu einem logischen Zeitpunkt,
//! während Schreiber isoliert neue Versionen im Staging-Puffer vorbereiten.
//!
//! ## Architektur-Rolle (Ring 0 / Ebene E1a — Kernkomponente)
//!
//! `acta-history` gehört zur innersten Systemschicht (Ring 0 / Ebene E1a). Es bietet fundamentale
//! Isolationsgarantien für relationale und vektorielle Indexstrukturen und besitzt keinerlei
//! Kenntnisse über Festplatten-Layouts oder Netzwerkprotokolle.
//!
//! ### Wer nutzt diese Crate? (Eingehende Abhängigkeiten)
//! - `acta-seg`: Nutzt MVCC-Sequenzen für WAL-Synchronisation und Compaction-GC-Pruning.
//! - `acta-engine`: Koordiniert Transaktionen, ReadSets und SSI-Validierung vor Commits.
//! - `acta-vec`: Führt snapshot-isolierte Vektorsuchen (`_at`) durch.
//! - `acta-history`: Schützt Snapshot-Zustände bei Backup-Erstellung.
//!
//! ### Wovon hängt diese Crate ab? (Ausgehende Abhängigkeiten)
//! - `acta-base`: Kanonische ID-Typen wie `TxId`, `DocId`, `TenantId`, `TOMBSTONE_BIT`.
//! - `acta-base`: Abstrakte Schnittstellen wie `Clock`, `SystemClock`, `MetricsSink`.
//! - Bibliotheken dritter Parteien: `parking_lot` (effiziente Mutexe/RwLocks), `ahash` (schnelle Hashing-Algorithmen).
//!
//! ### Kernkomponenten
//! - [`SequenceLog`]: Monotone Versionsverfolgung von Dokument-Änderungen für Snapshot-Suchen.
//! - [`SnapshotRegistry`] / [`SnapshotGuard`]: RAII-geschützte Schnappschüsse, die minimale aktive Sequenzen garantieren.
//! - [`TxBuffer`]: Nach Transaktions-ID partitionierter Staging-Puffer für atomare 2-Phase-Commit-Vorbereitungen.
//! - [`ReadSet`] / [`SsiValidator`] / [`SequenceLogSsiValidator`]: Serializable Snapshot Isolation (SSI)
//!   zur Erkennung von Write-Skew-Anomalien und Phantom-Reads bei Transaktionen.
//!
//! ## Sperrenordnung & Invarianten
//!
//! Um Verklemmungen (Deadlocks) bei nebenläufigem Zugriff deterministisch auszuschließen,
//! gilt für `acta-history` und die darüberliegende Engine eine strikte, totale Sperrenhierarchie:
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
//! **Regel:** Kein Sperr-Guard wird über `.await`-Punkte hinweg gehalten
//! (`acta-history` ist vollkommen synchron und frei von Async-Runtimes wie Tokio).
//!
//! ## Anwendungsbeispiel (Doctest)
//!
//! ```rust
//! use std::sync::Arc;
//! use acta_history::{SnapshotRegistry, TxBuffer, ReadSet, SequenceLogSsiValidator, SsiValidator, IndexOp};
//! use acta_history::types::{TxId, DocId};
//!
//! let registry = Arc::new(SnapshotRegistry::new());
//! let ssi = SequenceLogSsiValidator::new();
//! let buffer = TxBuffer::<String>::new();
//!
//! let tx = TxId::new(1);
//! let guard = registry.register(100);
//!
//! let mut read_set = ReadSet::new();
//! read_set.record_read(b"account:42", 100);
//!
//! buffer.begin(tx);
//! buffer.stage(tx, IndexOp::Insert { doc_id: DocId::from(1u64), data: "value".into() }).unwrap();
//!
//! // Validierung vor Commit gegen Anti-Dependencies
//! assert!(ssi.validate(tx, &read_set).is_ok());
//! drop(guard);
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

// ANCHOR[id=acta-history-exports]
pub use crate::mvcc_error::{MvccError, Result as MvccResult};
pub use acta_base::{DocId, TenantId, TxId, TOMBSTONE_BIT};

/// Kanonische Typen und Identifikatoren für MVCC aus `acta-base`.
pub mod types {
    pub use acta_base::{DocId, TenantId, TxId, TOMBSTONE_BIT};
}

// SECTION Workspace Compatibility Bridge
#[cfg(feature = "workspace")]
pub mod compat;

#[cfg(feature = "workspace")]
pub use crate::compat::{workspace_error as error, ActaError, Result};
#[cfg(not(feature = "workspace"))]
pub use crate::mvcc_error::Result;
// !SECTION Workspace Compatibility Bridge

// SECTION Core MVCC Modules
pub mod mvcc_error;
pub mod seq_log;
pub mod snapshot;
pub mod ssi;
pub mod tombstone;
pub mod tx_buffer;

pub use seq_log::{SeqLogChange, SeqLogEntry, SequenceLog};
pub use snapshot::{SnapshotGuard, SnapshotRegistry};
pub use ssi::{PruningBlockerInfo, ReadSet, SequenceLogSsiValidator, SsiValidator};
pub use tombstone::{SeqBitTombstone, TombstoneSemanticsCheck};
pub use tx_buffer::{IndexOp, TxBuffer};
// !SECTION Core MVCC Modules
