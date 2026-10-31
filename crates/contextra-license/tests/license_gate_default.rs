use contextra_license::{FeatureRing, LicenseError, LicenseGate, OpenFastGate};

#[test]
fn test_open_fast_gate_permits_fast_ring() {
    let gate = OpenFastGate;
    assert_eq!(gate.check_ring(FeatureRing::Fast), Ok(()));
}

#[test]
fn test_open_fast_gate_denies_sovereign_ring() {
    let gate = OpenFastGate;
    assert_eq!(
        gate.check_ring(FeatureRing::Sovereign),
        Err(LicenseError::NotActivated(FeatureRing::Sovereign))
    );
}

#[test]
fn test_open_fast_gate_denies_compliance_ring() {
    let gate = OpenFastGate;
    assert_eq!(
        gate.check_ring(FeatureRing::Compliance),
        Err(LicenseError::NotActivated(FeatureRing::Compliance))
    );
}
