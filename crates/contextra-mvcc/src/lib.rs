//! Sequenzen, Snapshots und SSI-Validierung für Mehrversions-Nebenläufigkeit.
//!
//! Ring: 2 (Kernkomponente)
//! Erlaubte Abhängigkeiten laut SPEC §5.3: contextra-types, contextra-ports.
//! Siehe SPEC §8.7 für normative Anforderungen.
//!
//! ## Lock-Ordnung (SPEC §5.4, §8.7, D-11, D-127)
//! Zur Vermeidung von Deadlocks zwischen nebenläufigen Schreib-, Commit- und Purge-Pfaden
//! gilt die strikte Lock-Hierarchie:
//! `Dokumentlock -> Commit-Barriere -> Purge-Barriere -> Journal`
//!
//! - **Dokumentlock**: Exklusiv pro Dokument während In-Memory-Modifikationen (keine Verschachtelung).
//! - **Commit-Barriere**: Shared (`read`) bei regulären Transaktions-Commits; Exclusive (`write`) bei Barrieren-Flush.
//! - **Purge-Barriere**: Shared (`read`) während regulärer Lese- und Schreiboperationen; Exclusive (`write`) während Purge-/Lösch-Sagas.
//! - **Journal**: Serialisiertes WAL/Manifest-Schreiben.
//!
//! Shard-Locks werden niemals verschachtelt und keine Sperre wird über `.await` gehalten (L-7, SPEC §5.4).

#![deny(dead_code)]
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![deny(clippy::indexing_slicing)]

pub use contextra_types::{ContextraError, DocId, Result, TxId, TOMBSTONE_BIT};

pub mod error;
pub mod seq_log;
pub mod snapshot;
pub mod ssi;
pub mod tx_buffer;

// Re-Exporte laut Spec §3 einzeln (seq, snapshot, ssi, tx, error)
pub use error::MvccError;
pub use seq_log::{SeqLogChange, SeqLogEntry, SequenceLog};
pub use snapshot::{SnapshotGuard, SnapshotRegistry};
pub use ssi::{PruningBlockerInfo, ReadSet, SequenceLogSsiValidator, SsiValidator};
pub use tx_buffer::{IndexOp, TxBuffer};
