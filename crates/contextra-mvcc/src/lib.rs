//! `Contextra` MVCC — Multi-Version Concurrency Control, Sequence Log, Transaction Staging, and SSI Validation.
//!
//! # Architecture Role (Ring 0)
//!
//! This crate provides the concurrency and isolation layer:
//! - [`SequenceLog`]: Monotonic sequence tracking and change notifications.
//! - [`SnapshotRegistry`] / [`SnapshotGuard`]: MVCC read isolation guards.
//! - [`TxBuffer`]: Sharded transaction staging with orphan reaping.
//! - [`ReadSet`] / [`SsiValidator`] / [`SequenceLogSsiValidator`]: Serializable Snapshot Isolation (SSI) read tracking and conflict detection.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub use contextra_types::*;

pub mod seq_log;
pub mod snapshot;
pub mod ssi;
pub mod tx_buffer;

pub use seq_log::{SeqLogChange, SeqLogEntry, SequenceLog};
pub use snapshot::{SnapshotGuard, SnapshotRegistry};
pub use ssi::{ReadSet, SequenceLogSsiValidator, SsiValidator};
pub use tx_buffer::{IndexOp, TxBuffer};
