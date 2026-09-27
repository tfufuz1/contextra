// FILE-CONTEXT
// ZWECK: Test deployment tier presets validation (Spec B.1.8 / INV-COLLECTION-PROFILE-2)

use contextra::collection_profile::DeploymentTier;

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
