use contextra_types::RetrievalStrategy;

/// Verlustfreie, deterministische Abbildung Arm-Index <-> RetrievalStrategy.
/// Lebt in contextra-router (wo Arme registriert werden), NICHT im BanditPolicy-Port.
#[derive(Debug, Clone, Copy)]
pub struct ArmRegistry {
    arms: [RetrievalStrategy; 5],
}

/// Fehlerklasse für ungültige Arm-Indizes — crate-lokal, P6-konform an Grenze konvertierbar.
#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum ArmRegistryError {
    /// Der angegebene Arm-Index liegt außerhalb des gültigen Bereichs (0..=4).
    #[error("arm index {0} out of range, expected 0..=4")]
    OutOfRange(u32),
}

impl Default for ArmRegistry {
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
    /// Liefert die `RetrievalStrategy` für den gegebenen Arm-Index (0..=4).
    pub fn strategy_for(&self, arm: u32) -> Result<RetrievalStrategy, ArmRegistryError> {
        self.arms
            .get(arm as usize)
            .copied()
            .ok_or(ArmRegistryError::OutOfRange(arm))
    }

    /// Liefert den Arm-Index (0..=4) für die gegebene `RetrievalStrategy`.
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

    #[test]
    fn test_arm_registry_determinism() {
        let reg1 = ArmRegistry::default();
        let reg2 = ArmRegistry::default();

        for arm in 0..5 {
            assert_eq!(reg1.strategy_for(arm).ok(), reg2.strategy_for(arm).ok());
        }
    }
}
