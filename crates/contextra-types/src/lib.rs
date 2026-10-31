#![deny(dead_code)]
//! `Contextra` Types — Canonical domain models, IDs, budgets, filters, and error types.
//!
//! # Architecture Role (Ring 0)
//!
//! This crate defines the foundational types and errors for the entire `Contextra` system.
//! In accordance with the Ring-0 architecture, this crate has NO asynchronous runtimes
//! and NO I/O dependencies.
//!
//! # Architektur-Rolle (Ring 0 / Kern)
//! Dieses Crate bildet das Fundament des gesamten Systems. Höhere Schichten binden
//! `contextra-types` ein, um gemeinsame Datenmodelle, Fehler und Identifikatoren zu nutzen.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

// ANCHOR[contextra-types:MOD-ERR]
/// Fehlerdefinitionen und kanonischer Fehlertyp `ContextraError`.
pub mod error;
// ANCHOR[contextra-types:MOD-DTO]
/// Serialisierbare Fehler-Transferobjekte (DTO) für IPC und FFI.
pub mod error_dto;
// ANCHOR[contextra-types:MOD-FINGERPRINT]
/// Modell- und Quantisierungs-Fingerabdrücke für Embeddings und LLM-Gewichte.
pub mod model_fingerprint;
// ANCHOR[contextra-types:MOD-STRATEGY]
/// Retrieval-Strategien zur semantischen Signalfusion.
pub mod retrieval_strategy;
// ANCHOR[contextra-types:MOD-SCHEMA]
/// Schema-Versionierung und Bit-Breiten für Manifeste und Indizes.
pub mod schema;
// ANCHOR[contextra-types:MOD-TENANTSCOPE]
/// Typ-sichere Mandanten-Isolation (`TenantScoped`).
pub mod tenant_scope;
// ANCHOR[contextra-types:MOD-TOMBSTONE]
/// Tombstone-Erkennung und Bitmasken-Semantik für gelöschte Datensätze.
pub mod tombstone;
// ANCHOR[contextra-types:MOD-TYPES]
/// Kernmodul für Domänenobjekte, Budgets, Filter und SAOS-Suchstrukturen.
pub mod types;
// ANCHOR[contextra-types:MOD-WITNESS]
/// Residuen-Messung, Request-Kontexte und Lösch-Saga-Zustände.
pub mod witness;

pub use error::{ContextraError, HnswDeletionError, Result};
pub use error_dto::ContextraErrorDto;
pub use model_fingerprint::ModelFingerprint;
pub use retrieval_strategy::RetrievalStrategy;
pub use schema::{DocIdWidth, ManifestSchemaVersion};
pub use tenant_scope::{TenantScopeViolation, TenantScoped};
pub use tombstone::{SeqBitTombstone, TombstoneSemanticsCheck};
pub use types::*;
pub use witness::{RequestContext, SagaRecord, ScanMethod};
