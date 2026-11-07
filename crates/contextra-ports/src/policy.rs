//! Policy enforcement port contracts (LicenseGate, EgressPolicy).

pub use crate::license::{
    AuthorizedRing, FeatureRing, LicenseError, LicenseGate, OpenFastGate, VectorDeleteMode,
};

/// Egress evaluation decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EgressDecision {
    /// Request is permitted to leave without modification.
    Allow,
    /// Request is permitted but payload must be redacted or sanitized.
    Redact(String),
    /// Request is denied.
    Deny(String),
}

/// Dynamic port trait for evaluating data egress policies.
pub trait EgressPolicy: Send + Sync {
    /// Evaluates if the outgoing content complies with data governance and egress rules.
    fn evaluate(&self, ctx: &contextra_types::RequestContext, payload: &str) -> EgressDecision;
}

/// Prelude for policy ports.
pub mod prelude {
    pub use super::{
        AuthorizedRing, EgressDecision, EgressPolicy, FeatureRing, LicenseGate, VectorDeleteMode,
    };
}
