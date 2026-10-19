#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::collections::BTreeMap;
use std::sync::Arc;

use contextra_license::{
    derive_local_installation_id_hash, FeatureRing, LicensePayload, SignedActivation,
    SignedLicenseGate,
};
use contextra_ports::clock::Clock;
use contextra_types::{DeterministicTimestamp, TenantId};
use ed25519_dalek::{Signer, SigningKey};
use proptest::prelude::*;
use rand::rngs::OsRng;
use serde::ser::{Serialize, Serializer};

struct TestClock {
    now_unix_secs: u64,
}

impl Clock for TestClock {
    fn now(&self) -> DeterministicTimestamp {
        DeterministicTimestamp::new(self.now_unix_secs * 1000, 0)
    }

    fn monotonic_ticks(&self) -> u64 {
        0
    }
}

fn generate_keypair() -> (SigningKey, ed25519_dalek::VerifyingKey) {
    let mut csprng = OsRng;
    let signing_key = SigningKey::generate(&mut csprng);
    let verifying_key = signing_key.verifying_key();
    (signing_key, verifying_key)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]
    #[test]
    fn prop_flipping_any_bit_in_payload_or_signature_yields_error(
        byte_idx in 0usize..100,
        bit_idx in 0u8..8u8,
    ) {
        let (signing_key, verifying_key) = generate_keypair();
        let local_id = [0x55u8; 32];
        let clock = Arc::new(TestClock { now_unix_secs: 1000 });

        let payload = LicensePayload {
            tenant_id: TenantId::SYSTEM,
            allowed_rings: vec![FeatureRing::Sovereign],
            expires_at: Some(2000),
            feature_flags: BTreeMap::new(),
            installation_id_hash: Some(local_id),
            key_id: None,
        };

        let payload_bytes = bincode::serialize(&payload).expect("serialization succeeds");
        let signature = signing_key.sign(&payload_bytes).to_bytes();

        // 1. Bit flip in payload
        if byte_idx < payload_bytes.len() {
            let mut corrupted_payload = payload_bytes.clone();
            corrupted_payload[byte_idx] ^= 1 << bit_idx;

            let res = SignedLicenseGate::from_signed_payload_with_clock(
                &corrupted_payload,
                &signature,
                verifying_key,
                clock.clone(),
            );
            prop_assert!(res.is_err(), "Bit flip at payload byte {} bit {} MUST fail signature verification", byte_idx, bit_idx);
        }

        // 2. Bit flip in signature
        if byte_idx < signature.len() {
            let mut corrupted_sig = signature;
            corrupted_sig[byte_idx] ^= 1 << bit_idx;

            let res = SignedLicenseGate::from_signed_payload_with_clock(
                &payload_bytes,
                &corrupted_sig,
                verifying_key,
                clock.clone(),
            );
            prop_assert!(res.is_err(), "Bit flip at signature byte {} bit {} MUST fail signature verification", byte_idx, bit_idx);
        }
    }
}

#[test]
fn test_signed_activation_bit_flip_verification() {
    let seed = [123u8; 32];
    let signing_key = SigningKey::from_bytes(&seed);
    let verifying_key = signing_key.verifying_key();
    let local_id = [0x42u8; 32];

    let act = SignedActivation::create_signed(FeatureRing::Sovereign, local_id, 2000, &signing_key)
        .expect("create_signed succeeds");

    assert!(act.verify_signature(&verifying_key));

    // Flip bits in signature
    for i in 0..act.signature.len() {
        for bit in 0..8 {
            let mut tampered = act.clone();
            tampered.signature[i] ^= 1 << bit;
            assert!(
                !tampered.verify_signature(&verifying_key),
                "Flipping bit {} in signature byte {} must fail verification",
                bit,
                i
            );
        }
    }
}

struct FailingPayload;

impl Serialize for FailingPayload {
    fn serialize<S>(&self, _serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Err(serde::ser::Error::custom("forced serialization error"))
    }
}

#[test]
fn test_serialization_failure_returns_err() {
    let failing = FailingPayload;
    let res = bincode::serialize(&failing);
    assert!(
        res.is_err(),
        "Failing payload serialization MUST return Err"
    );
}

#[test]
fn test_installation_id_persistence_failure_returns_err() {
    let invalid_path = std::path::Path::new("/proc/nonexistent_contextra_dir");
    let res = derive_local_installation_id_hash(Some(invalid_path));
    assert!(
        res.is_err(),
        "derive_local_installation_id_hash MUST propagate IO persistence error on write failure"
    );
}
