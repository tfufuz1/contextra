//! Sequenzen, Snapshots und SSI-Validierung für Mehrversions-Nebenläufigkeit.
//!
//! Ring: 2 (Kernkomponente)
//! Erlaubte Abhängigkeiten laut SPEC §5.3: contextra-types, contextra-ports.
//! Siehe SPEC §8.7 für normative Anforderungen.

#![deny(dead_code)]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

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
