#![forbid(unsafe_code)]
//! `memfuse-privacy` — Cloud Egress Security, DLP & Exfiltration Protection (Ring 3)

pub mod bulk_exfiltration_detector;
pub mod egress_gateway;
pub mod egress_guard;
pub mod egress_vault;
pub mod error;
pub mod guarded_payload;

pub use bulk_exfiltration_detector::{
    BulkExfiltrationDetector, BulkExfiltrationOutcome, SessionId,
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
};
pub use error::EgressError;
pub use guarded_payload::{GuardedPayload, Sanitized, Unsanitized};
