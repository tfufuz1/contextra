//! # MCP-Transport-Abstraktion (§4.14)
//!
//! **Ring:** Ring 3 (Applikations- und Koordinationsschicht).
//! **Zweck:** Definition des Übertragungskanals (Transport) für Interaktionen mit Sprachmodellen.
//! Unterstützt standardmäßig lokale Stdio-Verbindungen (JSON-RPC) und optional geschützte Cloud-HTTP-Verbindungen.
//!
//! **Sicherheits-Invariante:**
//! Cloud-Egress (Versand nach außen) erfordert zwingend das Feature `cloud-egress-guard` und
//! darf ausschließlich mit typgeprüften, sanitisierten Datenpaketen (`GuardedPayload<Sanitized>`) erfolgen.
//!
//! **Abhängigkeiten:**
//! - Upstream: `serde`.
//! - Downstream: [`crate::dispatch`].

use serde::{Deserialize, Serialize};

// ANCHOR[contextra-router:TRP-001]
/// Transport-Kanal für SLM-Routing-Entscheidungen.
///
/// Definiert, über welches Medium das Sprachmodell (SLM = Small Language Model)
/// angesprochen wird.
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

impl Transport {
    // ANCHOR[contextra-router:TRP-002]
    /// Prüft, ob dieser Transport einen externen Cloud-Endpunkt repräsentiert.
    ///
    /// # Ablauf
    /// 1. Pattern-Matching auf die Transport-Variante.
    /// 2. `false` für `StdioMcp`, `true` für `HttpCloud`.
    ///
    /// # Vorbedingungen
    /// Keine.
    ///
    /// # Nachbedingungen
    /// Gibt einen booleschen Indikator zurück.
    ///
    /// # Nebenwirkungen
    /// Keine.
    pub fn is_cloud(&self) -> bool {
        match self {
            Self::StdioMcp => false,
            #[cfg(feature = "cloud-egress-guard")]
            Self::HttpCloud { .. } => true,
        }
    }
}

impl Default for Transport {
    // ANCHOR[contextra-router:TRP-003]
    /// Liefert den Standard-Transport ([`Transport::StdioMcp`]).
    // NOTE: Fail-closed / Local-First: Der Standardweg verlässt niemals die lokale Maschine.
    fn default() -> Self {
        Self::StdioMcp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ANCHOR[contextra-router:TRP-T01]
    // Prüft, dass der Default-Transport immer der lokale StdioMcp ist.
    #[test]
    fn test_transport_default_is_stdio() {
        assert_eq!(Transport::default(), Transport::StdioMcp);
    }

    // ANCHOR[contextra-router:TRP-T02]
    // Prüft die verlustfreie Serialisierung und Deserialisierung des Transport-Enums via JSON.
    #[test]
    fn test_transport_serde_roundtrip() {
        let t = Transport::StdioMcp;
        let json = serde_json::to_string(&t).unwrap();
        let t2: Transport = serde_json::from_str(&json).unwrap();
        assert_eq!(t, t2);
    }
}
