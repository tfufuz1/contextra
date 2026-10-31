#![deny(dead_code)]
//! `Contextra` Types — Kanonische Domänenmodelle, Identifikatoren, Budgets, Filter und Fehlertypen.
//!
//! # Architektur-Rolle: Ring 0 (Typen-Kern)
//!
//! Dieses Crate definiert das fundamentale Typfundament für das gesamte Contextra-System.
//! Gemäß der Ring-0-Architektur (reiner Rechenkern):
//! - Keine asynchronen Laufzeiten (kein `tokio`).
//! - Keine direkten I/O- oder Dateisystemabhängigkeiten.
//! - Kein `unsafe` (`#![forbid(unsafe_code)]`).
//! - Vollständig deterministische Datenstrukturen und fehlerresistente DTO-Konvertierungen.
//!
//! # Einbindung im Workspace
//!
//! `contextra-types` ist ein fundamentales Ring-0-Workspace-Mitglied.
//!
//! **Abhängigkeiten (ausgehend):**
//! - Reines Rust mit Basis-Bibliotheken: `serde`, `serde_json`, `bincode`, `ahash`, `blake3`, `sha2`, `bytes`, `thiserror`.
//! - Keine Abhängigkeiten zu anderen Contextra-Crates.
//!
//! **Wird eingebunden von (eingehend, 30 Crates):**
//! - `contextra`, `contextra-adapt`, `contextra-agent`, `contextra-audit-export`, `contextra-avv-generator`,
//!   `contextra-checkpoint`, `contextra-cognition`, `contextra-core`, `contextra-crypto`, `contextra-db`,
//!   `contextra-engine`, `contextra-graph`, `contextra-infer-candle`, `contextra-infer-ollama`,
//!   `contextra-infer-onnx`, `contextra-ingest`, `contextra-kvcache`, `contextra-license`, `contextra-mcp`,
//!   `contextra-mvcc`, `contextra-ports`, `contextra-privacy`, `contextra-py`, `contextra-rank`,
//!   `contextra-router`, `contextra-simd`, `contextra-store`, `contextra-testkit`, `contextra-text`,
//!   `contextra-vector`.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

// ANCHOR[contextra-types:MOD-ERR]
/// Fehlertypen und Fehler-Hierarchie für Contextra.
pub mod error;
// ANCHOR[contextra-types:MOD-DTO]
/// Serialisierbare Data-Transfer-Objects (DTO) für fehlerresistente Schnittstellenübertragung.
pub mod error_dto;
// ANCHOR[contextra-types:MOD-FP]
/// Kryptografische Modell-Fingerabdrücke zur Validierung von Embedding- und Inferenz-Konfigurationen.
pub mod model_fingerprint;
// ANCHOR[contextra-types:MOD-RET]
/// Retrieval-Strategien und Routing-Optionen für die hybride Suche.
pub mod retrieval_strategy;
// ANCHOR[contextra-types:MOD-SCH]
/// Manifest-Schemaversionen und Dokument-ID-Breitenkonfigurationen.
pub mod schema;
// ANCHOR[contextra-types:MOD-TS]
/// Mandantenspezifische Wrapper-Typen zur Durchsetzung strikter Mandantenisolation.
pub mod tenant_scope;
// ANCHOR[contextra-types:MOD-TOMB]
/// Bitmaskenbasierte Tombstones (Löschmarker) für Multi-Version-Concurrency-Control (MVCC).
pub mod tombstone;
// ANCHOR[contextra-types:MOD-TYP]
/// Zentrale Domänentypen, Filter, Budgets, Scoring-Funktionen und SAOS-Suchstrukturen.
pub mod types;
// ANCHOR[contextra-types:MOD-WIT]
/// Audit-Trails, Zeugnisse (Witnesses) und Saga-Transaktionsaufzeichnungen.
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
