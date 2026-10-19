use contextra_license::{LicenseGateMode, SignedLicenseGate};

#[test]
fn test_license_gate_mode_construction_and_getter() {
    let mode_enforced = LicenseGateMode::enforced();
    assert_eq!(mode_enforced, LicenseGateMode::Enforced);

    #[cfg(any(test, debug_assertions, feature = "dev-open-gate"))]
    {
        let mode_dev = LicenseGateMode::development_open();
        assert_eq!(mode_dev, LicenseGateMode::DevelopmentOpen);
    }

    let gate = SignedLicenseGate::no_activation().with_mode(LicenseGateMode::Enforced);
    assert_eq!(gate.mode(), LicenseGateMode::Enforced);
}
