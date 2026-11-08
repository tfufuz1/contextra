//! Standalone-Beispiel für `contextra-license` (M10).
//! Demonstriert die Initialisierung des Lizenz-Gates und Prüfung von Feature-Ringen.

use contextra_license::{FeatureRing, LicenseGate, OpenFastGate, SignedLicenseGate};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Unlizenziertes Default-Gate (OpenFastGate)
    let fast_gate = OpenFastGate;
    let fast_ok = fast_gate.check_ring(FeatureRing::Fast);
    let sovereign_denied = fast_gate.check_ring(FeatureRing::Sovereign);

    println!("OpenFastGate Ring Fast: {:?}", fast_ok);
    println!("OpenFastGate Ring Sovereign: {:?}", sovereign_denied);

    // 2. Initialisierung eines SignedLicenseGate ohne Aktivierung (Fail-Closed)
    let signed_gate = SignedLicenseGate::no_activation();
    let gate_fast = signed_gate.check_ring(FeatureRing::Fast);
    let gate_sovereign = signed_gate.check_ring(FeatureRing::Sovereign);

    println!("SignedLicenseGate Ring Fast: {:?}", gate_fast);
    println!(
        "SignedLicenseGate Ring Sovereign (unactivated): {:?}",
        gate_sovereign
    );

    Ok(())
}
