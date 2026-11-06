//! Prüfung des Default-Verhaltens und der Null-Aktivierung des LicenseGate (OpenFastGate).
//! Verifiziert, dass ohne Lizenz ausschließlich der Fast-Ring zugänglich ist.

use contextra_license::{FeatureRing, LicenseError, LicenseGate, OpenFastGate};

// ANCHOR[contextra-license:TEST-DEFAULT-PERMITS-FAST]
// Prüft, dass OpenFastGate den FeatureRing::Fast genehmigt.
#[test]
fn test_open_fast_gate_permits_fast_ring() {
    let gate = OpenFastGate;
    assert_eq!(gate.check_ring(FeatureRing::Fast), Ok(()));
}

// ANCHOR[contextra-license:TEST-DEFAULT-DENIES-SOVEREIGN]
// Prüft, dass OpenFastGate den FeatureRing::Sovereign verweigert.
#[test]
fn test_open_fast_gate_denies_sovereign_ring() {
    let gate = OpenFastGate;
    assert_eq!(
        gate.check_ring(FeatureRing::Sovereign),
        Err(LicenseError::NotActivated(FeatureRing::Sovereign))
    );
}

// ANCHOR[contextra-license:TEST-DEFAULT-DENIES-COMPLIANCE]
// Prüft, dass OpenFastGate den FeatureRing::Compliance verweigert.
#[test]
fn test_open_fast_gate_denies_compliance_ring() {
    let gate = OpenFastGate;
    assert_eq!(
        gate.check_ring(FeatureRing::Compliance),
        Err(LicenseError::NotActivated(FeatureRing::Compliance))
    );
}
