//! # Routing-Strategie-Enum (§4.14, §13.1)
//!
//! **Ring:** Ring 3 (Applikations- und Koordinationsschicht).
//! **Zweck:** Festlegung des Algorithmus zur Auswahl des optimalen Modellprofils.
//!
//! **Strategien:**
//! - [`RoutingStrategy::Cascade`]: Deterministische Filter- und Bewertungskaskade (Standard).
//! - [`RoutingStrategy::ContextualBandit`]: LinUCB (Linear Upper Confidence Bound) Bandit-Verfahren.
//! - [`RoutingStrategy::FlowCorrectedThompson`]: Thompson Sampling mit dynamischer Fluss-Korrektur.
//!
//! **Abhängigkeiten:**
//! - Upstream: `serde`.
//! - Downstream: [`crate::router::RouterEngine`].

use serde::{Deserialize, Serialize};

// ANCHOR[contextra-router:RTS-001]
/// Auswahl-Strategie für SLM-Profil-Routing.
///
/// Steuert, wie die `RouterEngine` unter den verfügbaren Profilen das am besten
/// geeignete Modell bestimmt.
///
/// # Invariante (§13.3)
/// `ContextualBandit` bleibt Default-off bis `test_bandit_vs_cascade_regret_comparison`
/// messbaren Regret-Vorteil zeigt (P7-Nachweis-Pflicht).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum RoutingStrategy {
    /// Deterministischer Kaskaden-Klassifikator (produktiv, kalibriert).
    /// Bewertet Token-Budgets, Domänen-Tags und empirische Konfidenz.
    #[default]
    Cascade,
    /// LinUCB Contextual Bandit (Feature `bandit-routing`, Default: off).
    /// Lernt kontinuierlich aus Feedback unter Verwendung von Sherman-Morrison-Matrixupdates.
    #[cfg(feature = "bandit-routing")]
    ContextualBandit,
    #[cfg(feature = "flow-corrected-thompson")]
    /// Flow-Corrected Thompson Sampling (§21.3, AK-18). Feature `flow-corrected-thompson`, Default: off.
    /// Balanciert Exploration und Exploitation anhand von Posterior-Stichproben.
    FlowCorrectedThompson,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ANCHOR[contextra-router:RTS-T01]
    // Prüft, dass der Standard-Routing-Modus immer Cascade ist (P7: konservativer Standard).
    #[test]
    fn test_default_is_cascade() {
        assert_eq!(RoutingStrategy::default(), RoutingStrategy::Cascade);
    }

    // ANCHOR[contextra-router:RTS-T02]
    // Prüft die Serialisierung des Cascade-Enums zu JSON.
    #[test]
    fn test_cascade_serde() {
        let s = serde_json::to_string(&RoutingStrategy::Cascade).unwrap();
        assert_eq!(s, "\"Cascade\"");
    }

    // ANCHOR[contextra-router:RTS-T03]
    // Prüft den JSON-Roundtrip für die FlowCorrectedThompson-Strategie unter dem Feature-Flag.
    #[test]
    #[cfg(feature = "flow-corrected-thompson")]
    fn test_flow_corrected_thompson_serde() {
        let s = serde_json::to_string(&RoutingStrategy::FlowCorrectedThompson).unwrap();
        assert_eq!(s, "\"FlowCorrectedThompson\"");
        let de: RoutingStrategy = serde_json::from_str(&s).unwrap();
        assert_eq!(de, RoutingStrategy::FlowCorrectedThompson);
    }
}
