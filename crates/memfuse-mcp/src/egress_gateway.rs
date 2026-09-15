// FILE-CONTEXT
// ZWECK:       Egress Security Gateway & Classifier Enforcement for Cloud MCP Queries
// INVARIANTEN: APM-EGRESS-BYPASS: Jede Anfrage MUSS EgressClassifier::classify() durchlaufen.
//              APM-PANIC-ON-MISSING-FIELD: Keine unwrap()/expect()-Aufrufe bei Initialization/Execution (Fail-Closed).
// SIEHE AUCH:  crates/memfuse-crypto/src/egress_vault.rs

pub use memfuse_crypto::egress_vault::{
    BlockReason, CompiledPattern, EgressClassification, EgressClassifier, EgressVault,
    EgressVaultError,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::protocol::McpError;

// AI-TAG[SMELL][MAJOR][RESOLVED] AGT-MCP-egress-gateway — Replaced local stub DefaultEgressClassifier with official EgressVault contract from memfuse-security (TS: 2026-09-14T10:00:00Z) (SESSION: HEAD)
pub type DefaultEgressClassifier = EgressVault;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudQueryRequest {
    pub query: String,
    pub collection: Option<String>,
    pub max_results: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudQueryResponse {
    pub status: String,
    pub query: String,
    pub abstracted: bool,
    pub results: Vec<Value>,
    pub abstraction_notice: Option<String>,
}

pub async fn handle_cloud_query(
    request: CloudQueryRequest,
    classifier: &dyn EgressClassifier,
) -> Result<CloudQueryResponse, McpError> {
    if request.query.trim().is_empty() {
        return Err(McpError::invalid_params("query cannot be empty"));
    }
    if request.query.len() > crate::MAX_SEARCH_QUERY_BYTES {
        return Err(McpError::invalid_params(format!(
            "query size exceeds limit: {} bytes > {} limit",
            request.query.len(),
            crate::MAX_SEARCH_QUERY_BYTES
        )));
    }

    // APM-EGRESS-BYPASS Guard: Jede Anfrage wird zuerst klassifiziert.
    let classification = classifier.classify(&request.query).await;

    match classification {
        EgressClassification::Block(BlockReason::SensitivePattern(rule_id)) => {
            Err(McpError::invalid_params(format!(
                "Egress policy violation: query blocked by rule {rule_id}"
            )))
        }
        EgressClassification::Block(BlockReason::PolicyDenied(_)) => Err(McpError::invalid_params(
            "Egress policy violation: query exceeds policy or size limits",
        )),
        EgressClassification::Block(BlockReason::ClassificationTimeout) => Err(
            McpError::invalid_params("Egress policy violation: classification evaluation timeout"),
        ),
        EgressClassification::Block(BlockReason::InternalError(_)) => Err(
            McpError::invalid_params("Egress policy violation: internal classification error"),
        ),
        EgressClassification::Allow => Ok(CloudQueryResponse {
            status: "success".to_string(),
            query: request.query,
            abstracted: false,
            results: vec![],
            abstraction_notice: None,
        }),
        _ => Err(McpError::invalid_params(
            "Egress policy violation: query blocked due to unknown classification",
        )),
    }
}
