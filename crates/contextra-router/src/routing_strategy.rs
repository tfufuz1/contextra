//! Routing-Strategie-Enum (§4.14, §13.1).
//! Feature `bandit-routing` schaltet `ContextualBandit`-Variante frei.

use serde::{Deserialize, Serialize};

/// Auswahl-Strategie für SLM-Profil-Routing.
///
/// # Invariante (§13.3)
/// `ContextualBandit` bleibt Default-off bis `test_bandit_vs_cascade_regret_comparison`
/// messbaren Regret-Vorteil zeigt (P7-Nachweis-Pflicht).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum RoutingStrategy {
    /// Deterministischer Kaskaden-Klassifikator (produktiv, kalibriert).
    #[default]
    Cascade,
    /// LinUCB Contextual Bandit (Feature `bandit-routing`, Default: off).
    #[cfg(feature = "bandit-routing")]
    ContextualBandit,
    #[cfg(feature = "flow-corrected-thompson")]
    /// Flow-Corrected Thompson Sampling (§21.3, AK-18). Feature `flow-corrected-thompson`, Default: off.
    FlowCorrectedThompson,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_cascade() {
        assert_eq!(RoutingStrategy::default(), RoutingStrategy::Cascade);
    }

    #[test]
    fn test_cascade_serde() {
        let s = serde_json::to_string(&RoutingStrategy::Cascade).unwrap();
        assert_eq!(s, "\"Cascade\"");
    }

    #[test]
    #[cfg(feature = "flow-corrected-thompson")]
    fn test_flow_corrected_thompson_serde() {
        let s = serde_json::to_string(&RoutingStrategy::FlowCorrectedThompson).unwrap();
        assert_eq!(s, "\"FlowCorrectedThompson\"");
        let de: RoutingStrategy = serde_json::from_str(&s).unwrap();
        assert_eq!(de, RoutingStrategy::FlowCorrectedThompson);
    }
}
