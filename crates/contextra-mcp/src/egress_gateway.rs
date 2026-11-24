// =========================================================================
// SECTION: egress_gateway
// =========================================================================
// ANCHOR[contextra-mcp:EGRESS-GAT-001]
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
            EgressError::SurrogateCollision { ref surrogate, .. } => {
                McpError::internal_error(format!("Surrogate collision for {surrogate}"))
            }
            other => McpError::internal_error(other.to_string()),
        }
    }
}

/// Adapter, der einen [`PromptInjectionGuard`] als [`contextra_privacy::InjectionDetector`] nutzbar macht.
///
/// Die Orphan-Regel verbietet die direkte Implementierung, weil weder Trait noch Typ
/// in diesem Crate definiert sind. Der Newtype ist lokal und löst das auf.
pub struct PromptGuardDetector<'a>(pub &'a PromptInjectionGuard);

impl contextra_privacy::InjectionDetector for PromptGuardDetector<'_> {
    fn detect(&self, text: &str) -> Option<String> {
        self.0.detect(text)
    }
}

// !SECTION: egress_gateway
