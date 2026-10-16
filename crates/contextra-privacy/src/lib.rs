#![forbid(unsafe_code)]
//! `contextra-privacy` — Cloud Egress Security, DLP & Exfiltration Protection (Ring 3)

pub mod audit_trace;
pub mod avv_generator;
pub mod bulk_exfiltration_detector;
pub mod context_edit_audit;
pub mod egress_gateway;
pub mod egress_guard;
pub mod egress_vault;
pub mod error;
pub mod guarded_payload;
pub mod processing_registry;

pub use audit_trace::{compute_audit_trace, extract_rule_id, EgressClassifierTrace};
pub use avv_generator::{generate_avv_draft, AvvContext};
pub use bulk_exfiltration_detector::{
    BulkExfiltrationDetector, BulkExfiltrationOutcome, SessionId,
};
pub use context_edit_audit::{
    build_context_edit_audit_record, compute_record_hash, render_audit_line, verify_audit_chain,
    ContextEditAuditError, ContextEditAuditRecord, ContextEditKind,
};
pub use egress_gateway::{
    handle_cloud_query, handle_cloud_query_with_bulk_detector, handle_cloud_query_with_guard,
    process_cloud_response, CloudQueryRequest, CloudQueryResponse, CloudResponseRehydrator,
    DefaultEgressClassifier, EgressGuardCheck, InjectionDetector, MAX_SEARCH_QUERY_BYTES,
};
pub use egress_guard::{
    EgressGuard, TextSearchEngine, TextSearchResult, DEFAULT_EGRESS_GUARD_MIN_BYTES,
    DEFAULT_EGRESS_GUARD_THRESHOLD, DEFAULT_EGRESS_GUARD_TIMEOUT,
};
pub use egress_vault::{
    BlockReason, BoxFuture, CompiledPattern, EgressClassification, EgressClassifier, EgressVault,
    EgressVaultError, EntityRecognizer, NoOpRecognizer, PolicyCategory, SurrogateVault,
    MAX_CLASSIFY_PAYLOAD_BYTES,
};
pub use error::EgressError;
pub use guarded_payload::{GuardedPayload, Sanitized, Unsanitized};
pub use processing_registry::{
    generate_registry, render_markdown, ProcessingActivityRecord, ProcessingRegistryExport,
    ProcessorRole, TenantId,
};
