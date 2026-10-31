// FILE-CONTEXT
// ZWECK: Test deployment tier presets validation (Spec B.1.8 / INV-COLLECTION-PROFILE-2)

//! # Integrationstest: `deployment_tier_presets_valid`
//!
//! Überprüft die Invariante INV-COLLECTION-PROFILE-2: Alle vordefinierten Betriebsstufen
//! (`DeploymentTier::EdgeMinimal`, `PowerUserLocal`, `EnterpriseShared`, `EnterpriseRegulated`)
//! müssen nach der Auflösung via `.resolve()` eine intern valide Konfiguration ergeben (`validate().is_ok()`).


use contextra::collection_profile::DeploymentTier;

// ANCHOR[contextra:TEST-DTP-001]
// Stellt sicher, dass alle 4 DeploymentTier-Presets nach dem Auflösen fehlerfrei durch die Profilvalidierung laufen.
#[test]
fn test_all_deployment_tier_presets_valid() {
    let tiers = [
        DeploymentTier::EdgeMinimal,
        DeploymentTier::PowerUserLocal,
        DeploymentTier::EnterpriseShared,
        DeploymentTier::EnterpriseRegulated,
    ];

    for tier in tiers {
        let profile = tier.resolve();
        assert!(
            profile.validate().is_ok(),
            "Preset {:?} failed validation: {:?}",
            tier,
            profile.validate()
        );
    }
}
