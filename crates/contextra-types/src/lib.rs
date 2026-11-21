#![deny(dead_code)]
//! `Contextra` Types — Kanonische Domänenmodelle, Identifikatoren, Budgets, Filter und Fehlertypen.
//!
//! # Architektur-Rolle: Gruppe contracts (Ebene 2, Typen-Fundament)
//!
//! Dieses Crate definiert das fundamentale Typfundament für das gesamte Contextra-System.
//! Gemäß der Baukasten-Architektur (reiner Rechenkern):
//! - Keine asynchronen Laufzeiten (kein `tokio`).
//! - Keine direkten I/O- oder Dateisystemabhängigkeiten.
//! - Kein `unsafe` (`#![forbid(unsafe_code)]`).
//! - Vollständig deterministische Datenstrukturen und fehlerresistente DTO-Konvertierungen.
//!
//! # Einbindung im Workspace
//!
//! `contextra-types` ist das zentrale Vertrags- und Typenpaket.
//!
//! **Abhängigkeiten (ausgehend):**
//! - Reines Rust mit Basis-Bibliotheken: `serde`, `serde_json`, `bincode`, `ahash`, `blake3`, `sha2`, `bytes`, `thiserror`.
//! - Interne Abhängigkeiten: `contextra-ids`.
//!
//! **Einbindung im Workspace (eingehend):**
//! - Siehe `capabilities.toml` (Gruppe `contracts`, Ebene 2); dient als typisiertes Fundament
//!   für Speicher-, Abfrage- und Integrationsbausteine im Gesamtsystem.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used))]

/// Fehlertypen und Fehler-Hierarchie für Contextra.
pub mod error;
/// Serialisierbare Data-Transfer-Objects (DTO) für fehlerresistente Schnittstellenübertragung.
pub mod error_dto;
/// Kryptografische Modell-Fingerabdrücke zur Validierung von Embedding- und Inferenz-Konfigurationen.
pub mod model_fingerprint;
/// Retrieval-Strategien und Routing-Optionen für die hybride Suche.
pub mod retrieval_strategy;
/// Manifest-Schemaversionen und Dokument-ID-Breitenkonfigurationen.
pub mod schema;
/// Mandantenspezifische Wrapper-Typen zur Durchsetzung strikter Mandantenisolation.
pub mod tenant_scope;
/// Bitmaskenbasierte Tombstones (Löschmarker) für Multi-Version-Concurrency-Control (MVCC).
pub mod tombstone;
/// Zentrale Domänentypen, Filter, Budgets, Scoring-Funktionen und SAOS-Suchstrukturen.
pub mod types;
/// Audit-Trails, Zeugnisse (Witnesses) und Saga-Transaktionsaufzeichnungen.
pub mod erasure;
pub mod ids;
pub mod refs;
pub mod witness;

#[allow(deprecated)]
pub use erasure::*;
#[allow(deprecated)]
pub use ids::{ErasureId, FileId, GateId, Seq, StoreId, Timestamp};
pub use refs::{CollectionRef, DocKey, OperatorRef, SubjectRef, TenantRef, MAX_REF_LEN};

#[allow(deprecated)]
pub use error::{ContextraError, HnswDeletionError, Result};
pub use error_dto::ContextraErrorDto;
pub use model_fingerprint::ModelFingerprint;
pub use retrieval_strategy::RetrievalStrategy;
pub use schema::{DocIdWidth, ManifestSchemaVersion};
pub use tenant_scope::{TenantScopeViolation, TenantScoped};
pub use tombstone::{SeqBitTombstone, TombstoneSemanticsCheck};
pub use types::*;
#[allow(deprecated)]
pub use witness::{RequestContext, SagaRecord, ScanMethod};
