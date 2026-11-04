//! Öffentliche API und Verdrahtung. Enthält keine Logik.
//!
//! # Architektur-Rolle: Ring 4 (Fassade)
//!
//! Gemäß SPEC Abschnitt 5.1, 5.2, 7 und 8.14 (D-89, D-91, D-95):
//! - Das Crate `contextra` bildet die einzige öffentliche Oberfläche (D-66).
//! - Es enthält **keine eigene Logik** (AR-12, D-89).
//! - Erlaubte Module: `api`, `wiring`, `error`.
//! - Erlaubte Abhängigkeiten: `contextra-types`, `contextra-durable-fs`, `contextra-engine` (D-88).
//! - Pflichtparameter (Passphrase, Runtime-Handle, Backup-Retention, TenantContext) werden zur Compile-Zeit
//!   per Typsystem erzwungen (Type-State Builder).
//! - Strikte Trennung zwischen logischem `soft_delete` (Tombstone) und physischer Löschung `erase` (D-51).
//! - Alle öffentlichen Typen sind mit `#[non_exhaustive]` geschützt.

#![forbid(unsafe_code)]

pub mod api;
pub mod error;
pub mod wiring;

pub use api::{
    Collection, CollectionOptions, Contextra, DocHandle, DocKey, DocRef, Document, EngineBuilder,
    EraseMode, EraseReceipt, EraseRequest, EraseTarget, ErasureId, ErasureStatus, Filter,
    HealthReport, Hit, Missing, RestrictedAccess, RestrictedPurpose, RetentionAttestation,
    RetentionCode, ScanRange, SearchPage, Secret, Set, SignedReport, SnapshotId, Supersede, Tenant,
    TenantContext, TenantRef, Timing, UpdateOptions, UpdateReceipt,
};
pub use error::{ConfigDetails, ConflictDetails, Error, ErrorCode, LimitDetails, Result};
pub use wiring::{system_defaults, OsRng, SystemDefaults};
