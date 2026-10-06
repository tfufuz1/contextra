use contextra_license::{
    derive_local_installation_id_hash, FeatureRing, LicenseGate, SignedLicenseGate,
};
use ed25519_dalek::SigningKey;

#[test]
fn test_activation_params_production_path() {
    let signing_key = SigningKey::from_bytes(&[7u8; 32]);
    let verifying_key = signing_key.verifying_key();
    let inst_id = derive_local_installation_id_hash(None);
    let expires_at_unix = 2_000_000_000;

    // Call enclosing production constructor from_activation_params
    let gate = SignedLicenseGate::from_activation_params(
        FeatureRing::Sovereign,
        inst_id,
        expires_at_unix,
        &signing_key,
        verifying_key,
    )
    .with_auto_installation_id(None);

    // Verify the gate operates as expected
    assert!(gate.check_ring(FeatureRing::Fast).is_ok());
    assert!(gate.check_ring(FeatureRing::Sovereign).is_ok());
    assert!(gate.activation().is_some());
    assert_eq!(gate.activation().unwrap().ring, FeatureRing::Sovereign);
}

#[test]
fn test_unactivated_local_and_default_production_path() {
    // Test Default trait
    let default_gate = SignedLicenseGate::default();
    assert!(default_gate.check_ring(FeatureRing::Fast).is_ok());
    assert!(default_gate.check_ring(FeatureRing::Compliance).is_err());

    // Test unactivated_local
    let unactivated_gate = SignedLicenseGate::unactivated_local(None);
    assert!(unactivated_gate.check_ring(FeatureRing::Fast).is_ok());
    assert!(unactivated_gate.check_ring(FeatureRing::Sovereign).is_err());
}

#[test]
fn test_for_testing_production_path() {
    let gate = SignedLicenseGate::for_testing(vec![FeatureRing::Compliance], None);

    assert!(gate.check_ring(FeatureRing::Fast).is_ok());
    assert!(gate.check_ring(FeatureRing::Compliance).is_ok());
    assert!(gate.check_ring(FeatureRing::Sovereign).is_err());
    assert!(gate.license_payload().is_some());
}
