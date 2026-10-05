use contextra_license::{
    derive_local_installation_id_hash, FeatureRing, LicenseGate, SignedActivation,
    SignedLicenseGate,
};
use ed25519_dalek::{SigningKey, VerifyingKey};

#[test]
fn test_j26closure_symbols_direct_invocation() {
    let secret = [99u8; 32];
    let signing_key = SigningKey::from_bytes(&secret);
    let verifying_key = signing_key.verifying_key();
    let inst_id = derive_local_installation_id_hash(None);
    let expires = 2_100_000_000i64;

    // 1. Symbol: create_signed
    let activation = SignedActivation::create_signed(
        FeatureRing::Sovereign,
        inst_id,
        expires,
        &signing_key,
    );
    assert_eq!(activation.ring, FeatureRing::Sovereign);
    assert!(activation.verify_signature(&verifying_key));

    // 2. Symbol: from_activation
    let gate_from_act = SignedLicenseGate::from_activation(activation.clone(), verifying_key);
    assert_eq!(gate_from_act.activation(), Some(&activation));

    // 3. Symbol: no_activation
    let gate_no_act = SignedLicenseGate::no_activation();
    assert!(gate_no_act.check_ring(FeatureRing::Fast).is_ok());
    assert!(gate_no_act.check_ring(FeatureRing::Sovereign).is_err());
    assert!(gate_no_act.activation().is_none());

    // 4. Symbol: with_local_installation_id
    let gate_bound = gate_from_act.with_local_installation_id(inst_id);
    assert!(gate_bound.check_ring(FeatureRing::Sovereign).is_ok());

    // 5. Symbol: create_test_signed_payload
    let (payload_bytes, sig, vk_bytes) =
        SignedLicenseGate::create_test_signed_payload(vec![FeatureRing::Compliance], Some(expires));
    assert!(!payload_bytes.is_empty());
    assert_eq!(sig.len(), 64);
    assert_eq!(vk_bytes.len(), 32);

    let parsed_vk = VerifyingKey::from_bytes(&vk_bytes).unwrap();
    let gate_payload =
        SignedLicenseGate::from_signed_payload(&payload_bytes, &sig, parsed_vk).unwrap();
    assert!(gate_payload.check_ring(FeatureRing::Compliance).is_ok());
}
