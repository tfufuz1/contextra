// FILE-CONTEXT
// ZWECK:       Egress Security Gateway & Classifier Enforcement for Cloud MCP Queries (Re-export from contextra-privacy)

pub use contextra_privacy::egress_gateway::*;
pub use contextra_privacy::egress_vault::{EgressClassification, EgressClassifier};

use crate::prompt_injection::PromptInjectionGuard;
use crate::protocol::McpError;
use contextra_privacy::EgressError;

impl From<EgressError> for McpError {
    fn from(err: EgressError) -> Self {
        match err {
            EgressError::InvalidParams(msg) => McpError::invalid_params(msg),
            EgressError::PolicyViolation(msg) => McpError::invalid_params(msg),
            EgressError::InternalError(msg) => McpError::internal_error(msg),
        }
    }
}

impl contextra_privacy::InjectionDetector for PromptInjectionGuard {
    fn detect(&self, text: &str) -> Option<String> {
        self.detect(text)
    }
}
