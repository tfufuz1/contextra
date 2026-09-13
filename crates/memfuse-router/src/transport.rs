//! MCP-Transport-Abstraktion (§4.14).
//! Feature `cloud-egress-guard` schaltet `HttpCloud`-Variante frei.

use serde::{Deserialize, Serialize};

/// Transport-Kanal für SLM-Routing-Entscheidungen.
///
/// # Invariante
/// `HttpCloud` darf nur mit aktivem Feature `cloud-egress-guard` und
/// einem `GuardedPayload<Sanitized>` verwendet werden — Compile-Zeit-Schutz
/// via `dispatch_to_cloud()` Signatur.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[non_exhaustive]
pub enum Transport {
    /// Standard: stdio JSON-RPC 2.0 zu lokalem SLM-Prozess (ADR-010).
    StdioMcp,
    /// Cloud-HTTP: Nur mit Feature `cloud-egress-guard` + `GuardedPayload<Sanitized>`.
    #[cfg(feature = "cloud-egress-guard")]
    HttpCloud { url: String },
}

impl Default for Transport {
    fn default() -> Self {
        Self::StdioMcp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_default_is_stdio() {
        assert_eq!(Transport::default(), Transport::StdioMcp);
    }

    #[test]
    fn test_transport_serde_roundtrip() {
        let t = Transport::StdioMcp;
        let json = serde_json::to_string(&t).unwrap();
        let t2: Transport = serde_json::from_str(&json).unwrap();
        assert_eq!(t, t2);
    }
}
