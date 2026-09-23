//! `Contextra` MVCC — Multi-Version Concurrency Control, Sequence Log, and Transaction Staging.
//!
//! # Architecture Role (Ring 0)
//!
//! This crate provides the concurrency and isolation layer:
//! - [`SequenceLog`]: Monotonic sequence tracking and change notifications.
//! - [`SnapshotRegistry`] / [`SnapshotGuard`]: MVCC read isolation guards.
//! - [`TxBuffer`]: Sharded transaction staging with orphan reaping.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub use contextra_types::*;

pub mod seq_log;
pub mod snapshot;
pub mod tx_buffer;

pub use seq_log::{SeqLogChange, SeqLogEntry, SequenceLog};
pub use snapshot::{SnapshotGuard, SnapshotRegistry};
pub use tx_buffer::{IndexOp, TxBuffer};
