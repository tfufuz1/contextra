use contextra_types::RetrievalStrategy;

/// Verlustfreie, deterministische Abbildung Arm-Index <-> RetrievalStrategy.
/// Lebt in contextra-router (wo Arme registriert werden), NICHT im BanditPolicy-Port.
#[derive(Debug, Clone, Copy)]
pub struct ArmRegistry {
    arms: [RetrievalStrategy; 4],
}

/// Fehlerklasse für ungültige Arm-Indizes — crate-lokal, P6-konform an Grenze konvertierbar.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum ArmRegistryError {
    /// Der angegebene Arm-Index liegt außerhalb des gültigen Bereichs (0..=3).
    #[error("arm index {0} out of range, expected 0..=3")]
    OutOfRange(u32),
}

impl Default for ArmRegistry {
    fn default() -> Self {
        // Feste, deterministische Reihenfolge (Determinismus-Invariante, §3.2 Punkt 3).
        Self {
            arms: [
                RetrievalStrategy::Vector,
                RetrievalStrategy::Text,
                RetrievalStrategy::Graph,
                RetrievalStrategy::Hybrid,
            ],
        }
    }
}

impl ArmRegistry {
    /// Liefert die `RetrievalStrategy` für den gegebenen Arm-Index (0..=3).
    pub fn strategy_for(&self, arm: u32) -> Result<RetrievalStrategy, ArmRegistryError> {
        self.arms
            .get(arm as usize)
            .copied()
            .ok_or(ArmRegistryError::OutOfRange(arm))
    }

    /// Liefert den Arm-Index (0..=3) für die gegebene `RetrievalStrategy`.
    ///
    /// Die Implementierung verwendet ein `match` über alle bekannten Varianten mit einem
    /// sicheren Fallback für das von Rust bei Crate-übergreifenden `#[non_exhaustive]` Enums
    /// erzwungene Wildcard-Pattern (`_`), um die Zero-Panic-Doktrin (P7) strikt einzuhalten.
    pub fn arm_for(&self, strategy: RetrievalStrategy) -> u32 {
        match strategy {
            RetrievalStrategy::Vector => 0,
            RetrievalStrategy::Text => 1,
            RetrievalStrategy::Graph => 2,
            RetrievalStrategy::Hybrid => 3,
            _ => 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arm_registry_bijection() {
        let registry = ArmRegistry::default();
        let strategies = [
            RetrievalStrategy::Vector,
            RetrievalStrategy::Text,
            RetrievalStrategy::Graph,
            RetrievalStrategy::Hybrid,
        ];

        for (expected_arm, &strategy) in strategies.iter().enumerate() {
            let arm = expected_arm as u32;
            let mapped_strategy = registry
                .strategy_for(arm)
                .unwrap_or(RetrievalStrategy::Vector);
            assert_eq!(mapped_strategy, strategy);

            let mapped_arm = registry.arm_for(strategy);
            assert_eq!(mapped_arm, arm);
        }
    }

    #[test]
    fn test_arm_registry_out_of_range() {
        let registry = ArmRegistry::default();
        assert_eq!(
            registry.strategy_for(4),
            Err(ArmRegistryError::OutOfRange(4))
        );
        assert_eq!(
            registry.strategy_for(100),
            Err(ArmRegistryError::OutOfRange(100))
        );
    }

    #[test]
    fn test_arm_registry_determinism() {
        let reg1 = ArmRegistry::default();
        let reg2 = ArmRegistry::default();

        for arm in 0..4 {
            assert_eq!(reg1.strategy_for(arm).ok(), reg2.strategy_for(arm).ok());
        }
    }
}
