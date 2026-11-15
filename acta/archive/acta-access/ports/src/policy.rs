//!
//! LINK crates/acta-ports/src/lib.rs
//! LINK crates/acta-ports/src/license.rs
//!
//! // SECTION policy-egress-contracts
//! // NOTE: Autorisierungs- und Egress-Governance-Schnittstellen.

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
    ///
    /// # Fehler
    /// Keine; fehlerfrei über `EgressDecision` abgebildet.
    /// # Crash
    /// Reiner Rechenvorgang im Arbeitsspeicher; keine persistente Mutation.
    fn evaluate(&self, ctx: &acta_types::RequestContext, payload: &str) -> EgressDecision;
}
// !SECTION policy-egress-contracts

/// Prelude for policy ports.
pub mod prelude {
    pub use super::{
        AuthorizedRing, EgressDecision, EgressPolicy, FeatureRing, LicenseGate, VectorDeleteMode,
    };
}
