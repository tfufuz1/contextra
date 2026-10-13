use contextra_license::wiring::{LicenseWiring, LicenseWiringDeps};
use contextra_license::LicenseGateMode;
use contextra_ports::license::{FeatureRing, LicenseError};

#[test]
fn test_license_gate_mode_and_wiring() {
    let mode_enforced = LicenseGateMode::Enforced;

    #[cfg(any(test, debug_assertions, feature = "dev-open-gate"))]
    let mode_dev = LicenseGateMode::DevelopmentOpen;

    let deps_enforced = LicenseWiringDeps {
        mode: mode_enforced,
        payload: None,
        activation: None,
        keyring: None,
        clock: None,
        storage_path: None,
    };

    let wiring = LicenseWiring::build(deps_enforced).expect("LicenseWiring::build succeeded");
    let gate = wiring.into_gate();

    // Default enforced without activation permits Fast ring (INV-LICENSE-2) but rejects Sovereign/Compliance
    assert_eq!(gate.check_ring(FeatureRing::Fast), Ok(()));
    assert_eq!(
        gate.check_ring(FeatureRing::Sovereign),
        Err(LicenseError::NotActivated(FeatureRing::Sovereign))
    );

    #[cfg(any(test, debug_assertions, feature = "dev-open-gate"))]
    {
        let deps_dev = LicenseWiringDeps {
            mode: mode_dev,
            payload: None,
            activation: None,
            keyring: None,
            clock: None,
            storage_path: None,
        };

        let wiring_dev = LicenseWiring::build(deps_dev).expect("LicenseWiring::build succeeded");
        let gate_dev = wiring_dev.into_gate();
        assert_eq!(gate_dev.check_ring(FeatureRing::Fast), Ok(()));
    }
}
