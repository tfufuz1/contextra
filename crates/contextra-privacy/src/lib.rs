#![forbid(unsafe_code)]
//! `contextra-privacy` — Cloud Egress Security, DLP & Exfiltration Protection (Ring 3)

pub mod audit_trace;
pub mod bulk_exfiltration_detector;
pub mod egress_gateway;
pub mod egress_guard;
pub mod egress_vault;
pub mod error;
pub mod entity_recognizer;
pub mod guarded_payload;

pub use audit_trace::{compute_audit_trace, extract_rule_id, EgressClassifierTrace};
pub use bulk_exfiltration_detector::{
    BulkExfiltrationDetector, BulkExfiltrationOutcome, SessionId,
};
pub use egress_gateway::{
    handle_cloud_query, handle_cloud_query_with_bulk_detector, handle_cloud_query_with_guard,
    pii_vault_forces_crypto_shred, process_cloud_response, resolve_effective_kv_delete_mode,
    CloudQueryRequest, CloudQueryResponse, CloudResponseRehydrator, DefaultEgressClassifier,
    EgressGuardCheck, InjectionDetector, MAX_SEARCH_QUERY_BYTES,
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
