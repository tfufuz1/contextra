//! # Integrationstest: ArmRegistry mit fünf Standard-Armen
//!
//! **Ring:** Ring 3 (Applikations- und Testschicht).
//! **Zweck:** Integrationstest: ArmRegistry mit fünf Standard-Armen.

// ANCHOR[contextra-router:IT-ARM5-001]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use contextra_router::{ArmRegistry, ArmRegistryError};
use contextra_types::RetrievalStrategy;

// ANCHOR[contextra-router:IT-ARM5-T01]
// Test `test_arm_registry_five_arms_bijection`: Prüft das erwartete Verhalten und Invarianten für integrationstest: armregistry mit fünf standard-armen.
#[test]
fn test_arm_registry_five_arms_bijection() {
    let registry = ArmRegistry::default();

    let expected_arms = [
        (0u32, RetrievalStrategy::Vector),
        (1u32, RetrievalStrategy::Text),
        (2u32, RetrievalStrategy::Graph),
        (3u32, RetrievalStrategy::Hybrid),
        (
            4u32,
            RetrievalStrategy::Global {
                max_community_nodes: None,
                min_community_size: None,
            },
        ),
    ];

    for (arm_index, expected_strategy) in expected_arms {
        let mapped_strategy = registry
            .strategy_for(arm_index)
            .expect("strategy_for returns valid strategy for slot 0..=4");
        assert_eq!(mapped_strategy, expected_strategy);

        let mapped_arm = registry.arm_for(expected_strategy);
        assert_eq!(mapped_arm, arm_index);
    }
}

// ANCHOR[contextra-router:IT-ARM5-T02]
// Test `test_arm_registry_five_arms_custom_global_variant`: Prüft das erwartete Verhalten und Invarianten für integrationstest: armregistry mit fünf standard-armen.
#[test]
fn test_arm_registry_five_arms_custom_global_variant() {
    let registry = ArmRegistry::default();

    let custom_global = RetrievalStrategy::Global {
        max_community_nodes: Some(100),
        min_community_size: Some(5),
    };

    let mapped_arm = registry.arm_for(custom_global);
    assert_eq!(mapped_arm, 4);
}

// ANCHOR[contextra-router:IT-ARM5-T03]
// Test `test_arm_registry_five_arms_out_of_range`: Prüft das erwartete Verhalten und Invarianten für integrationstest: armregistry mit fünf standard-armen.
#[test]
fn test_arm_registry_five_arms_out_of_range() {
    let registry = ArmRegistry::default();

    assert_eq!(
        registry.strategy_for(5),
        Err(ArmRegistryError::OutOfRange(5))
    );
    assert_eq!(
        registry.strategy_for(42),
        Err(ArmRegistryError::OutOfRange(42))
    );
}
