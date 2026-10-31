#![forbid(unsafe_code)]
//! `contextra-privacy` — Cloud Egress Security, DLP & Exfiltration Protection (Ring 3)
//!
//! # Zweck und Architektur
//! Dieses Crate implementiert die Sicherheits- und Datenschutzschicht für ausgehende Datenströme
//! (Cloud Egress) in Contextra. Ziel ist es, Data Loss Prevention (DLP, Schutz vor Datenverlust),
//! Erkennung von Exfiltrationsversuchen, automatisierte Pseudonymisierung über Surrogat-Token
//! sowie unveränderbare Audit-Protokolle bereitzustellen.
//!
//! # Ring-Zugehörigkeit
//! Ring 3 (Domain / Application Services). Das Crate liegt oberhalb der Speicher- und Kryptokerne
//! (Ring 1/2) und unterhalb der API- und Netzwerk-Fassaden (Ring 4).
//!
//! # Einbindung und Abhängigkeiten
//! - **Eingehend:** Wird eingebunden von `contextra`, `contextra-mcp` und `contextra-router`.
//! - **Ausgehend:** Hängt ab von `contextra-types`, `contextra-ports`, `contextra-avv-generator`.
//!
//! # Wichtigste Module
//! - [`egress_vault`]: DLP-Prüfung, Regex-Klassifikation und `SurrogateVault`-Pseudonymisierung.
//! - [`entity_recognizer`]: Erkennung personenbezogener Daten (PII) wie IBAN, E-Mail, Kreditkarten.
//! - [`egress_guard`]: Semantische k-NN Vektorähnlichkeitsprüfung gegen Datenabfluss.
//! - [`egress_gateway`]: Zentrales Schutzgateway (Egress Shield) vor Weiterleitung in externe Cloud-Dienste.
//! - [`bulk_exfiltration_detector`]: Sitzungsbasiertes Rate-Limiting gegen Massenabflüsse.
//! - [`context_edit_audit`]: Kryptografisch verkettete Audit-Aufzeichnungen von Kontextbearbeitungen.
//! - [`processing_registry`]: Verzeichnis von Verarbeitungstätigkeiten (VVT) nach Art. 30 DSGVO.
//! - [`guarded_payload`]: Type-State-Muster zur Verhinderung ungesäuberter Datenabflüsse zur Compile-Zeit.

pub mod audit_trace;
pub mod avv_generator;
pub mod bulk_exfiltration_detector;
pub mod context_edit_audit;
pub mod egress_gateway;
pub mod egress_guard;
pub mod egress_vault;
pub mod entity_recognizer;
pub mod error;
pub mod guarded_payload;
pub mod processing_registry;

// ANCHOR[contextra-privacy:LIB-001]
// Re-Exports für bequemen Zugriff auf die Kernfunktionalitäten
pub use audit_trace::{compute_audit_trace, extract_rule_id, EgressClassifierTrace};
#[allow(deprecated)]
pub use avv_generator::{generate_avv_draft, AvvContext};
pub use bulk_exfiltration_detector::{
    BulkExfiltrationDetector, BulkExfiltrationOutcome, SessionId,
};
pub use context_edit_audit::{
    build_context_edit_audit_record, compute_record_hash, render_audit_line, verify_audit_chain,
    ContextEditAuditError, ContextEditAuditRecord, ContextEditKind,
};
#[allow(deprecated)]
pub use egress_gateway::{
    handle_cloud_query, handle_cloud_query_with_bulk_detector, handle_cloud_query_with_guard,
    process_cloud_response, CloudQueryRequest, CloudQueryResponse, CloudResponseRehydrator,
    DefaultEgressClassifier, EgressGateway, EgressGatewayBuilder, EgressGuardCheck,
    InjectionDetector, MAX_SEARCH_QUERY_BYTES,
};
pub use egress_guard::{
    EgressGuard, TextSearchEngine, TextSearchResult, DEFAULT_EGRESS_GUARD_MIN_BYTES,
    DEFAULT_EGRESS_GUARD_THRESHOLD, DEFAULT_EGRESS_GUARD_TIMEOUT,
};
#[allow(deprecated)]
pub use egress_vault::NoOpRecognizer;
pub use egress_vault::{
    BlockReason, BoxFuture, CompiledPattern, EgressClassification, EgressClassifier, EgressVault,
    EgressVaultError, EntityRecognizer, PolicyCategory, SurrogateVault,
    MAX_CLASSIFY_PAYLOAD_BYTES,
};
pub use entity_recognizer::{
    Gazetteer, PatternRecognizer, RuleBasedEntityRecognizer, DEFAULT_MAX_INPUT_LENGTH,
};
pub use error::EgressError;
pub use guarded_payload::{GuardedPayload, Sanitized, Unsanitized};
pub use processing_registry::{
    generate_registry, render_markdown, ProcessingActivityRecord, ProcessingRegistry,
    ProcessingRegistryExport, ProcessorRole,
};
