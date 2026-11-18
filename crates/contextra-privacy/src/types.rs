//! `types` — Re-exports der kanonischen Identitäts-, Zeit- und Isolationstypen.

pub use contextra_env::{Clock, SystemClock};
pub use contextra_ids::{TenantId, TenantScopeViolation, TenantScoped};

/// Entscheidung einer Egress-Richtlinienauswertung.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EgressDecision {
    /// Anfrage ohne Modifikation gestattet.
    Allow,
    /// Anfrage gestattet, aber Payload muss bereinigt / maskiert werden.
    Redact(String),
    /// Anfrage verweigert.
    Deny(String),
}

/// Crate-eigener dynamischer Trait für Egress-Richtlinien.
pub trait EgressPolicy: Send + Sync {
    /// Bewertet, ob der ausgehende Payload die Richtlinie erfüllt.
    fn evaluate(&self, payload: &str) -> EgressDecision;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct AllowAllPolicy;
    impl EgressPolicy for AllowAllPolicy {
        fn evaluate(&self, _payload: &str) -> EgressDecision {
            EgressDecision::Allow
        }
    }

    #[test]
    fn test_egress_policy_trait_contract() {
        let policy: Box<dyn EgressPolicy> = Box::new(AllowAllPolicy);
        assert_eq!(policy.evaluate("test-payload"), EgressDecision::Allow);
    }
}
