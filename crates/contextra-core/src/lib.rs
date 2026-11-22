//! `Contextra` Core — Types, traits, and error handling (Strangler Facade).
//!
//! # Architecture Role (Legacy Triebwerk — Layer 0 -> Ring 0-4 Migration)
//!
//! As part of the Ring-Modell Phase 1b migration, the functionality of `contextra-core`
//! has been decomposed into modular Ring-0 crates:
//! - [`contextra_types`]: Canonical domain models, IDs, budgets, filters, and error types.
//! - [`contextra_ports`]: Abstract `dyn`-compatible subsystem interfaces and traits.
//! - [`contextra_mvcc`]: Multi-version concurrency control, sequence log, and tx staging.
//! - [`contextra_wire`]: Zero-copy FlatBuffers IPC and wire message adapters.
//!
//! This crate acts as a backward-compatible Strangler Facade, re-exporting all types
//! with deprecation markers to avoid breaking existing downstream callers.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

// Re-export Ring-0 modular crates
#[deprecated(
    since = "0.1.0",
    note = "Moved to contextra_types as part of Ring-Modell Phase 1b"
)]
pub use contextra_types::error;
#[deprecated(
    since = "0.1.0",
    note = "Moved to contextra_types as part of Ring-Modell Phase 1b"
)]
pub use contextra_types::error_dto;
#[deprecated(
    since = "0.1.0",
    note = "Moved to contextra_types as part of Ring-Modell Phase 1b"
)]
pub use contextra_types::model_fingerprint;
#[deprecated(
    since = "0.1.0",
    note = "Moved to contextra_types as part of Ring-Modell Phase 1b"
)]
pub use contextra_types::schema;
#[deprecated(
    since = "0.1.0",
    note = "Moved to contextra_types as part of Ring-Modell Phase 1b"
)]
pub use contextra_types::tombstone;
#[deprecated(
    since = "0.1.0",
    note = "Moved to contextra_types as part of Ring-Modell Phase 1b"
)]
pub use contextra_types::types;

/// Traits submodule re-exporting `contextra_ports` for backward compatibility.
#[deprecated(
    since = "0.1.0",
    note = "Moved to contextra_ports as part of Ring-Modell Phase 1b"
)]
pub mod traits {
    pub use contextra_ports::*;
}

#[deprecated(
    since = "0.1.0",
    note = "Moved to contextra_mvcc as part of Ring-Modell Phase 1b"
)]
pub use contextra_mvcc::seq_log;
#[deprecated(
    since = "0.1.0",
    note = "Moved to contextra_mvcc as part of Ring-Modell Phase 1b"
)]
pub use contextra_mvcc::snapshot;
#[deprecated(
    since = "0.1.0",
    note = "Moved to contextra_mvcc as part of Ring-Modell Phase 1b"
)]
pub use contextra_mvcc::tx_buffer;

/// IPC module re-exporting `contextra_wire` for backward compatibility.
#[deprecated(
    since = "0.1.0",
    note = "Moved to contextra_wire as part of Ring-Modell Phase 1b"
)]
pub mod ipc {
    pub use contextra_wire::*;
}

#[deprecated(
    since = "0.1.0",
    note = "Moved to contextra_mvcc::seq_log as part of Ring-Modell Phase 1b"
)]
pub use contextra_mvcc::seq_log::{SeqLogChange, SeqLogEntry, SequenceLog};

#[deprecated(
    since = "0.1.0",
    note = "Moved to contextra_mvcc::snapshot as part of Ring-Modell Phase 1b"
)]
pub use contextra_mvcc::snapshot::{SnapshotGuard, SnapshotRegistry};

#[deprecated(
    since = "0.1.0",
    note = "Moved to contextra_mvcc::tx_buffer as part of Ring-Modell Phase 1b"
)]
pub use contextra_mvcc::tx_buffer::{IndexOp, TxBuffer};

#[deprecated(
    since = "0.1.0",
    note = "Moved to contextra_ports as part of Ring-Modell Phase 1b"
)]
pub use contextra_ports::*;

#[deprecated(
    since = "0.1.0",
    note = "Moved to contextra_types::error as part of Ring-Modell Phase 1b"
)]
pub use contextra_types::error::{ContextraError, Result};

#[deprecated(
    since = "0.1.0",
    note = "Moved to contextra_types::error_dto as part of Ring-Modell Phase 1b"
)]
pub use contextra_types::error_dto::ContextraErrorDto;

#[deprecated(
    since = "0.1.0",
    note = "Moved to contextra_types::model_fingerprint as part of Ring-Modell Phase 1b"
)]
pub use contextra_types::model_fingerprint::ModelFingerprint;

#[deprecated(
    since = "0.1.0",
    note = "Moved to contextra_types::schema as part of Ring-Modell Phase 1b"
)]
pub use contextra_types::schema::{DocIdWidth, ManifestSchemaVersion};

#[deprecated(
    since = "0.1.0",
    note = "Moved to contextra_types::tombstone as part of Ring-Modell Phase 1b"
)]
pub use contextra_types::tombstone::{SeqBitTombstone, TombstoneSemanticsCheck};
