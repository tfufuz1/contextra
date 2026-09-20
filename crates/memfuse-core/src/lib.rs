//! `MemFuse` Core — Types, traits, and error handling (Strangler Facade).
//!
//! # Architecture Role (Legacy Triebwerk — Layer 0 -> Ring 0-4 Migration)
//!
//! As part of the Ring-0-4 migration (Phase 1b), the functionality of `memfuse-core`
//! has been decomposed into modular Ring-0 crates:
//! - [`memfuse_types`]: Canonical domain models, IDs, budgets, filters, and error types.
//! - [`memfuse_ports`]: Abstract `dyn`-compatible subsystem interfaces and traits.
//! - [`memfuse_mvcc`]: Multi-version concurrency control, sequence log, and tx staging.
//!
//! This crate acts as a backward-compatible Strangler Facade, re-exporting all types
//! with deprecation markers to avoid breaking existing downstream callers.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

// Re-export Ring-0 modular crates
#[deprecated(since = "0.1.0", note = "Moved to memfuse_types as part of Ring-Modell Phase 1b")]
pub use memfuse_types::error;
#[deprecated(since = "0.1.0", note = "Moved to memfuse_types as part of Ring-Modell Phase 1b")]
pub use memfuse_types::error_dto;
#[deprecated(since = "0.1.0", note = "Moved to memfuse_types as part of Ring-Modell Phase 1b")]
pub use memfuse_types::model_fingerprint;
#[deprecated(since = "0.1.0", note = "Moved to memfuse_types as part of Ring-Modell Phase 1b")]
pub use memfuse_types::schema;
#[deprecated(since = "0.1.0", note = "Moved to memfuse_types as part of Ring-Modell Phase 1b")]
pub use memfuse_types::tombstone;
#[deprecated(since = "0.1.0", note = "Moved to memfuse_types as part of Ring-Modell Phase 1b")]
pub use memfuse_types::types;

/// Traits submodule re-exporting `memfuse_ports` for backward compatibility.
#[deprecated(since = "0.1.0", note = "Moved to memfuse_ports as part of Ring-Modell Phase 1b")]
pub mod traits {
    pub use memfuse_ports::*;
}

#[deprecated(since = "0.1.0", note = "Moved to memfuse_mvcc as part of Ring-Modell Phase 1b")]
pub use memfuse_mvcc::seq_log;
#[deprecated(since = "0.1.0", note = "Moved to memfuse_mvcc as part of Ring-Modell Phase 1b")]
pub use memfuse_mvcc::snapshot;
#[deprecated(since = "0.1.0", note = "Moved to memfuse_mvcc as part of Ring-Modell Phase 1b")]
pub use memfuse_mvcc::tx_buffer;

pub mod ipc;

pub use memfuse_types::error::{MemFuseError, Result};
pub use memfuse_types::error_dto::MemFuseErrorDto;
pub use memfuse_types::model_fingerprint::ModelFingerprint;
pub use memfuse_types::schema::{DocIdWidth, ManifestSchemaVersion};
pub use memfuse_mvcc::seq_log::{SeqLogChange, SeqLogEntry, SequenceLog};
pub use memfuse_mvcc::snapshot::{SnapshotGuard, SnapshotRegistry};
pub use memfuse_mvcc::tx_buffer::{IndexOp, TxBuffer};
pub use memfuse_types::tombstone::{SeqBitTombstone, TombstoneSemanticsCheck};
pub use memfuse_ports::*;

