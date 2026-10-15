use contextra_license::signed_gate::{
    LicenseKeyEntry, LicenseKeyring, SignedActivation, SignedLicenseGate,
};
use contextra_ports::clock::Clock;
use contextra_ports::license::{FeatureRing, LicenseError, LicenseGate};
use ed25519_dalek::SigningKey;
use std::sync::Arc;

struct MockClock {
    now: i64,
}

impl Clock for MockClock {
    fn now_unix_nanos(&self) -> u64 {
        (self.now as u64) * 1_000_000_000
    }

    fn monotonic_nanos(&self) -> u64 {
        (self.now as u64) * 1_000_000_000
    }
}

#[test]
fn test_key_rotation_paths() {
    let clock = Arc::new(MockClock { now: 1_000_000 });

    let sk1 = SigningKey::from_bytes(&[1u8; 32]);
    let vk1 = sk1.verifying_key();

    let sk2 = SigningKey::from_bytes(&[2u8; 32]);
    let vk2 = sk2.verifying_key();

    let mut keyring = LicenseKeyring::new();
    keyring.add_key(LicenseKeyEntry {
        key_id: "key-1".to_string(),
        verifying_key: vk1,
        valid_from_unix: Some(500_000),
        valid_until_unix: Some(2_000_000),
        is_revoked: false,
    });
    keyring.add_key(LicenseKeyEntry {
        key_id: "key-2".to_string(),
        verifying_key: vk2,
        valid_from_unix: Some(900_000),
        valid_until_unix: Some(3_000_000),
        is_revoked: false,
    });

    let inst_id = [7u8; 32];

    // 1. Valid signature with key-1
    let mut act1 =
        SignedActivation::create_signed(FeatureRing::Sovereign, inst_id, 2_000_000, &sk1);
    act1.key_id = Some("key-1".to_string());

    let gate1 = SignedLicenseGate::from_activation_with_clock(act1.clone(), vk1, clock.clone())
        .with_keyring(keyring.clone())
        .with_local_installation_id(inst_id);

    assert_eq!(gate1.check_ring(FeatureRing::Sovereign), Ok(()));

    // 2. Rotated key with key-2
    let mut act2 =
        SignedActivation::create_signed(FeatureRing::Sovereign, inst_id, 2_000_000, &sk2);
    act2.key_id = Some("key-2".to_string());

    let gate2 = SignedLicenseGate::from_activation_with_clock(act2.clone(), vk2, clock.clone())
        .with_keyring(keyring.clone())
        .with_local_installation_id(inst_id);

    assert_eq!(gate2.check_ring(FeatureRing::Sovereign), Ok(()));

    // 3. Unknown key ID
    let mut act_unknown = act1.clone();
    act_unknown.key_id = Some("unknown-key-99".to_string());

    let gate_unknown =
        SignedLicenseGate::from_activation_with_clock(act_unknown, vk1, clock.clone())
            .with_keyring(keyring.clone())
            .with_local_installation_id(inst_id);

    assert_eq!(
        gate_unknown.check_ring(FeatureRing::Sovereign),
        Err(LicenseError::InvalidSignature)
    );

    // 4. Revoked key
    let mut keyring_revoked = keyring.clone();
    keyring_revoked.revoke_key("key-1");

    let gate_revoked =
        SignedLicenseGate::from_activation_with_clock(act1.clone(), vk1, clock.clone())
            .with_keyring(keyring_revoked)
            .with_local_installation_id(inst_id);

    assert_eq!(
        gate_revoked.check_ring(FeatureRing::Sovereign),
        Err(LicenseError::InvalidSignature)
    );

    // 5. Expired key window
    let mut keyring_expired_window = LicenseKeyring::new();
    keyring_expired_window.add_key(LicenseKeyEntry {
        key_id: "key-1".to_string(),
        verifying_key: vk1,
        valid_from_unix: Some(100_000),
        valid_until_unix: Some(800_000), // now is 1_000_000 -> expired key
        is_revoked: false,
    });

    let gate_key_expired = SignedLicenseGate::from_activation_with_clock(act1, vk1, clock)
        .with_keyring(keyring_expired_window)
        .with_local_installation_id(inst_id);

    assert_eq!(
        gate_key_expired.check_ring(FeatureRing::Sovereign),
        Err(LicenseError::InvalidSignature)
    );
}
