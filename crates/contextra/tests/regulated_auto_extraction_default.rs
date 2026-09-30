// FILE-CONTEXT
// ZWECK: Test default auto-extraction settings across deployment tiers (Spec B.1.8 / Spec D.6 / J.16 / B-08)

use contextra::collection_profile::{
    AutoExtractionMode, DeploymentTier, DEFAULT_AUTO_EXTRACTION_MODE,
    ENTERPRISE_REGULATED_AUTO_EXTRACTION_DEFAULT,
};

#[test]
fn test_regulated_auto_extraction_default() {
    assert_eq!(
        ENTERPRISE_REGULATED_AUTO_EXTRACTION_DEFAULT,
        AutoExtractionMode::Disabled,
        "ENTERPRISE_REGULATED_AUTO_EXTRACTION_DEFAULT must be Disabled"
    );

    assert_eq!(
        DEFAULT_AUTO_EXTRACTION_MODE,
        AutoExtractionMode::Enabled,
        "DEFAULT_AUTO_EXTRACTION_MODE must be Enabled"
    );

    assert_eq!(
        DeploymentTier::EnterpriseRegulated
            .resolve()
            .auto_extraction,
        AutoExtractionMode::Disabled,
        "EnterpriseRegulated tier preset must default auto_extraction to Disabled"
    );

    assert_eq!(
        DeploymentTier::EdgeMinimal.resolve().auto_extraction,
        AutoExtractionMode::Enabled,
        "EdgeMinimal tier preset must default auto_extraction to Enabled"
    );

    assert_eq!(
        DeploymentTier::PowerUserLocal.resolve().auto_extraction,
        AutoExtractionMode::Enabled,
        "PowerUserLocal tier preset must default auto_extraction to Enabled"
    );

    assert_eq!(
        DeploymentTier::EnterpriseShared.resolve().auto_extraction,
        AutoExtractionMode::Enabled,
        "EnterpriseShared tier preset must default auto_extraction to Enabled"
    );
}
