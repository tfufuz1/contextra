//! # Erweiterte Fusions-Schnittstellen (`fusion/ext`)
//!
//! **Zweck:** Definiert Abstraktionen für experimentelle Fusions- und Bonusmechanismen
//! (wie z. B. Resonanz-Kohärenz-Bonus), ohne Kopplung an externe Workspace-Abhängigkeiten.

use crate::fusion::SearchResult;

/// Trait für Signal-Resonanz und Kohärenz-Modulation.
pub trait ResonanceModifier {
    /// Wendet Resonanz- oder Kohärenzmodifikationen auf die Ergebnisliste an.
    fn modify_results(&self, results: Vec<SearchResult>, valid_signal_count: usize) -> Vec<SearchResult>;
}

#[cfg(not(feature = "exp-resonance"))]
/// Stub-Konfiguration für Resonanz-Bonus, wenn `exp-resonance` nicht aktiviert ist.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, Default)]
pub struct ResonanceConfig {
    /// Dummy field
    pub enabled: bool,
}

#[cfg(not(feature = "exp-resonance"))]
impl ResonanceModifier for ResonanceConfig {
    fn modify_results(&self, results: Vec<SearchResult>, _valid_signal_count: usize) -> Vec<SearchResult> {
        results
    }
}

#[cfg(not(feature = "exp-resonance"))]
/// No-Op Fallback für `apply_resonance_bonus`.
pub fn apply_resonance_bonus(
    results: Vec<SearchResult>,
    _valid_signal_count: usize,
    _config: &ResonanceConfig,
) -> Vec<SearchResult> {
    results
}
