// FILE-CONTEXT
// STAND:       2026-09-13
// ZWECK:       Egress Security Gateway & Classifier Enforcement for Cloud MCP Queries
// INVARIANTEN: APM-EGRESS-BYPASS: Jede Anfrage MUSS EgressClassifier::classify() durchlaufen.
//              APM-PANIC-ON-MISSING-FIELD: Keine unwrap()-Aufrufe auf Client-Eingaben.
// SIEHE AUCH:  crates/memfuse-crypto/src/egress_vault.rs

use memfuse_core::BoxFuture;
pub use memfuse_security::egress_vault::{BlockReason, EgressClassification, EgressClassifier};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::protocol::McpError;

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
        EgressClassification::Block(reason) => Err(McpError::invalid_params(format!(
            "Egress policy violation: query blocked: {reason:?}"
        ))),
        EgressClassification::Allow => Ok(CloudQueryResponse {
            status: "success".to_string(),
            query: request.query,
            abstracted: false,
            results: vec![],
            abstraction_notice: None,
        }),
        EgressClassification::RequiresAbstraction => Ok(CloudQueryResponse {
            status: "success".to_string(),
            query: "[REDACTED_SENSITIVE_QUERY]".to_string(),
            abstracted: true,
            results: vec![],
            abstraction_notice: Some(
                "Payload was abstracted before processing due to egress policy".to_string(),
            ),
        }),
        _ => Err(McpError::invalid_params(
            "Egress policy violation: query blocked due to unknown classification",
        )),
    }
}

pub struct DefaultEgressClassifier;

impl DefaultEgressClassifier {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DefaultEgressClassifier {
    fn default() -> Self {
        Self::new()
    }
}

impl EgressClassifier for DefaultEgressClassifier {
    fn classify<'a>(&'a self, payload: &'a str) -> BoxFuture<'a, EgressClassification> {
        Box::pin(async move {
            if payload.contains("sk-")
                || payload.contains("AKIA")
                || payload.contains("api_key")
                || payload.contains("password")
                || payload.contains('@')
            {
                EgressClassification::Block(BlockReason::SensitivePattern(
                    "Sensitive pattern detected in payload".to_string(),
                ))
            } else if payload.contains("abstract") || payload.contains("PII") {
                EgressClassification::RequiresAbstraction
            } else {
                EgressClassification::Allow
            }
        })
    }
}
