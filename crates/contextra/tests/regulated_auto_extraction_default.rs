// FILE-CONTEXT
// ZWECK: Verify DeploymentTier presets auto_extraction defaults (Spec D.6 / J.16/B-08)

use contextra::collection_profile::{
    AutoExtractionMode, DeploymentTier, ENTERPRISE_REGULATED_AUTO_EXTRACTION_DEFAULT,
};

#[test]
fn test_enterprise_regulated_auto_extraction_default_is_disabled() {
    assert_eq!(
        ENTERPRISE_REGULATED_AUTO_EXTRACTION_DEFAULT,
        AutoExtractionMode::Disabled,
        "ENTERPRISE_REGULATED_AUTO_EXTRACTION_DEFAULT constant must be Disabled"
    );

    let profile = DeploymentTier::EnterpriseRegulated.resolve();
    assert_eq!(
        profile.auto_extraction,
        AutoExtractionMode::Disabled,
        "EnterpriseRegulated preset must have auto_extraction set to Disabled by default"
    );
}

#[test]
fn test_other_deployment_tiers_auto_extraction_default_is_enabled() {
    let enabled_tiers = [
        DeploymentTier::EdgeMinimal,
        DeploymentTier::PowerUserLocal,
        DeploymentTier::EnterpriseShared,
    ];

    for tier in enabled_tiers {
        let profile = tier.resolve();
        assert_eq!(
            profile.auto_extraction,
            AutoExtractionMode::Enabled,
            "Preset {:?} must have auto_extraction set to Enabled by default",
            tier
        );
    }
}
