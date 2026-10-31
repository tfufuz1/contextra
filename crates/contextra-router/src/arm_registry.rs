//! # ArmRegistry: Abbildung zwischen Multi-Armed-Bandit-Armen und Retrieval-Strategien
//!
//! **Ring:** Ring 3 (Applikations- und Koordinationsschicht).
//! **Zweck:** Deterministische, bijektive Abbildung zwischen 0-basierten Bandit-Arm-Indizes (0..=4)
//! und den konkreten Informationsabruf-Strategien ([`RetrievalStrategy`]).
//!
//! **Hintergrund:**
//! Multi-Armed-Bandit-Algorithmen (wie LinUCB oder Thompson Sampling) operieren mathematisch auf
//! diskreten Arm-Indizes (0, 1, 2, ...). Die `ArmRegistry` übersetzt diese Zahlen verlustfrei in
//! Domänenkonzepte des Contextra-Systems (Vektor-, Text-, Graph-, Hybrid- oder Global-Suche).
//!
//! **Abhängigkeiten:**
//! - Upstream: `contextra_types::RetrievalStrategy`.
//! - Downstream: Bandit-Routing-Komponenten in `crates/contextra-router/src/router/`.

use contextra_types::RetrievalStrategy;

// ANCHOR[contextra-router:ARM-001]
/// Verlustfreie, deterministische Abbildung Arm-Index <-> RetrievalStrategy.
///
/// Lebt in `contextra-router` (wo Arme registriert und ausgewertet werden),
/// NICHT im generischen `BanditPolicy`-Port in Ring 0.
///
/// # Invariante
/// Hält genau 5 registrierte Arme in fester Reihenfolge (Indizes 0 bis 4).
#[derive(Debug, Clone, Copy)]
pub struct ArmRegistry {
    arms: [RetrievalStrategy; 5],
}

// ANCHOR[contextra-router:ARM-002]
/// Fehlerklasse für ungültige Arm-Indizes — crate-lokal, P6-konform an Systemgrenzen konvertierbar.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum ArmRegistryError {
    /// Der angegebene Arm-Index liegt außerhalb des gültigen Bereichs (0..=4).
    #[error("arm index {0} out of range, expected 0..=4")]
    OutOfRange(u32),
}

impl Default for ArmRegistry {
    // ANCHOR[contextra-router:ARM-003]
    /// Initialisiert die Standard-Registrierung mit den 5 Standard-Retrieval-Strategien:
    /// - Index 0: `RetrievalStrategy::Vector`
    /// - Index 1: `RetrievalStrategy::Text`
    /// - Index 2: `RetrievalStrategy::Graph`
    /// - Index 3: `RetrievalStrategy::Hybrid`
    /// - Index 4: `RetrievalStrategy::Global`
    fn default() -> Self {
        Self {
            arms: [
                RetrievalStrategy::Vector,
                RetrievalStrategy::Text,
                RetrievalStrategy::Graph,
                RetrievalStrategy::Hybrid,
                RetrievalStrategy::Global {
                    max_community_nodes: None,
                    min_community_size: None,
                },
            ],
        }
    }
}

impl ArmRegistry {
    // ANCHOR[contextra-router:ARM-004]
    /// Liefert die [`RetrievalStrategy`] für den gegebenen Arm-Index (0..=4).
    ///
    /// # Ablauf
    /// 1. Konvertierung des `u32`-Index nach `usize`.
    /// 2. Abruf des Eintrags aus dem internen Array `arms`.
    /// 3. Rückgabe der Strategie oder `ArmRegistryError::OutOfRange`.
    ///
    /// # Vorbedingungen
    /// `arm` sollte im Bereich `0..=4` liegen.
    ///
    /// # Nachbedingungen
    /// Gibt bei Erfolg die zugehörige `RetrievalStrategy` zurück.
    ///
    /// # Fehler
    /// Liefert [`ArmRegistryError::OutOfRange`], wenn `arm >= 5`.
    ///
    /// # Nebenwirkungen
    /// Keine (reine Array-Indizierung im Speicher).
    pub fn strategy_for(&self, arm: u32) -> Result<RetrievalStrategy, ArmRegistryError> {
        self.arms
            .get(arm as usize)
            .copied()
            .ok_or(ArmRegistryError::OutOfRange(arm))
    }

    // ANCHOR[contextra-router:ARM-005]
    /// Liefert den numerischen Arm-Index (0..=4) für die gegebene [`RetrievalStrategy`].
    ///
    /// # Ablauf
    /// 1. Pattern-Matching auf die Strategie-Variante.
    /// 2. Zuweisung des Index:
    ///    - Vector => 0
    ///    - Text => 1
    ///    - Graph => 2
    ///    - Hybrid => 3
    ///    - Global => 4
    /// 3. Fallback auf 0 für unerkannte/zukünftige Varianten.
    ///
    /// # Vorbedingungen
    /// Keine.
    ///
    /// # Nachbedingungen
    /// Liefert stets einen Index im Bereich `0..=4`.
    ///
    /// # Nebenwirkungen
    /// Keine.
    // NOTE: Fallback auf 0 (Vector) verhindert Panics bei Varianten-Erweiterungen.
    pub fn arm_for(&self, strategy: RetrievalStrategy) -> u32 {
        match strategy {
            RetrievalStrategy::Vector => 0,
            RetrievalStrategy::Text => 1,
            RetrievalStrategy::Graph => 2,
            RetrievalStrategy::Hybrid => 3,
            RetrievalStrategy::Global { .. } => 4,
            _ => 0,
        }
    }
}

#[cfg(test)]
#[allow(clippy::indexing_slicing, clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    // ANCHOR[contextra-router:ARM-T01]
    // Prüft die Bijektion zwischen allen fünf Retrieval-Strategien und den Arm-Indizes 0 bis 4.
    #[test]
    fn test_arm_registry_bijection() {
        let registry = ArmRegistry::default();
        let strategies = [
            RetrievalStrategy::Vector,
            RetrievalStrategy::Text,
            RetrievalStrategy::Graph,
            RetrievalStrategy::Hybrid,
            RetrievalStrategy::Global {
                max_community_nodes: None,
                min_community_size: None,
            },
        ];

        for (expected_arm, &strategy) in strategies.iter().enumerate() {
            let arm = u32::try_from(expected_arm).unwrap_or(0);
            let mapped_strategy = registry
                .strategy_for(arm)
                .unwrap_or(RetrievalStrategy::Vector);
            assert_eq!(mapped_strategy, strategy);

            let mapped_arm = registry.arm_for(strategy);
            assert_eq!(mapped_arm, arm);
        }
    }

    // ANCHOR[contextra-router:ARM-T02]
    // Prüft, dass ungültige Arm-Indizes (>= 5) zuverlässig mit OutOfRange abgewiesen werden.
    #[test]
    fn test_arm_registry_out_of_range() {
        let registry = ArmRegistry::default();
        assert_eq!(
            registry.strategy_for(5),
            Err(ArmRegistryError::OutOfRange(5))
        );
        assert_eq!(
            registry.strategy_for(100),
            Err(ArmRegistryError::OutOfRange(100))
        );
    }

    // ANCHOR[contextra-router:ARM-T03]
    // Prüft die Identität und den Determinismus zweier unabhängig initialisierter Registrierungs-Instanzen.
    #[test]
    fn test_arm_registry_determinism() {
        let reg1 = ArmRegistry::default();
        let reg2 = ArmRegistry::default();

        for arm in 0..5 {
            assert_eq!(reg1.strategy_for(arm).ok(), reg2.strategy_for(arm).ok());
        }
    }
}
