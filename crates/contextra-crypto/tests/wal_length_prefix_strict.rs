#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

// ZWECK: Strict length prefix tests, proptest non-collision proof, and golden checksum tests for WAL V3 HMAC verifier.

use contextra_crypto::wal_crypto::{len_prefix, IntegrityVerifier, WalEntrySnapshot, WalHmac};
use contextra_crypto::error::CryptoError;
use proptest::prelude::*;

#[test]
fn test_golden_v3_hmac_checksum() {
    let key = b"test-integrity-key-32-bytes----";

    // Compute checksum using WalHmac directly (matching V3 format specification)
    let mut mac = WalHmac::new(key).expect("WalHmac init failed");
    mac.update(&[0u8; 32]); // prev_hmac
    mac.update(&100u64.to_le_bytes()); // seq_no
    mac.update(&42u64.to_le_bytes()); // tx_id
    mac.update(&[0u8]); // Put op
    mac.update(&(9u32).to_le_bytes()); // key_len = 9
    mac.update(b"user:1001");
    mac.update(&(12u32).to_le_bytes()); // val_len = 12
    mac.update(b"data:payload");
    let checksum = mac.finalize();

    let hex_checksum: String = checksum.iter().map(|b| format!("{:02x}", b)).collect();
    assert_eq!(
        hex_checksum,
        "3a2dbea634884f08e030e908ac4cbb4223d2c0a163ac05fa2392baf2371974da",
        "Golden HMAC checksum for canonical V3 WAL entry MUST remain bit-exact"
    );

    let snapshot = WalEntrySnapshot {
        tx_id: 42,
        seq_no: 100,
        op_type: 0,
        key: b"user:1001".to_vec(),
        value: b"data:payload".to_vec(),
        checksum,
        prev_hmac: [0u8; 32],
    };

    let mut verifier = IntegrityVerifier::new(key);
    assert!(
        verifier.verify_and_update_v3(&snapshot, 0).is_ok(),
        "V3 verifier MUST accept valid snapshot matching golden checksum"
    );
}

proptest! {
    #[test]
    fn prop_v3_concatenation_collision_free(
        concat in proptest::collection::vec(any::<u8>(), 2..200),
        split1 in 0usize..200,
        split2 in 0usize..200,
    ) {
        let len = concat.len();
        let s1 = split1 % (len + 1);
        let s2 = split2 % (len + 1);

        if s1 != s2 {
            let k1 = concat[..s1].to_vec();
            let v1 = concat[s1..].to_vec();
            let k2 = concat[..s2].to_vec();
            let v2 = concat[s2..].to_vec();

            let key = b"integrity-key-32-bytes-proptest";

            // Entry 1
            let mut mac1 = WalHmac::new(key).unwrap();
            mac1.update(&[0u8; 32]);
            mac1.update(&1u64.to_le_bytes());
            mac1.update(&10u64.to_le_bytes());
            mac1.update(&[0u8]);
            mac1.update(&(u32::try_from(k1.len()).unwrap()).to_le_bytes());
            mac1.update(&k1);
            mac1.update(&(u32::try_from(v1.len()).unwrap()).to_le_bytes());
            mac1.update(&v1);
            let checksum1 = mac1.finalize();

            let entry1 = WalEntrySnapshot {
                tx_id: 10,
                seq_no: 1,
                op_type: 0,
                key: k1,
                value: v1,
                checksum: checksum1,
                prev_hmac: [0u8; 32],
            };

            // Entry 2
            let mut mac2 = WalHmac::new(key).unwrap();
            mac2.update(&[0u8; 32]);
            mac2.update(&1u64.to_le_bytes());
            mac2.update(&10u64.to_le_bytes());
            mac2.update(&[0u8]);
            mac2.update(&(u32::try_from(k2.len()).unwrap()).to_le_bytes());
            mac2.update(&k2);
            mac2.update(&(u32::try_from(v2.len()).unwrap()).to_le_bytes());
            mac2.update(&v2);
            let checksum2 = mac2.finalize();

            let entry2 = WalEntrySnapshot {
                tx_id: 10,
                seq_no: 1,
                op_type: 0,
                key: k2,
                value: v2,
                checksum: checksum2,
                prev_hmac: [0u8; 32],
            };

            // Verify both via IntegrityVerifier::verify_and_update_v3
            let mut verifier1 = IntegrityVerifier::new(key);
            prop_assert!(verifier1.verify_and_update_v3(&entry1, 0).is_ok());

            let mut verifier2 = IntegrityVerifier::new(key);
            prop_assert!(verifier2.verify_and_update_v3(&entry2, 0).is_ok());

            prop_assert_ne!(
                verifier1.last_hmac_snapshot(),
                verifier2.last_hmac_snapshot(),
                "Distinct key/value boundary splits MUST produce different HMAC chain states in V3"
            );
        }
    }
}

#[cfg(target_pointer_width = "64")]
#[test]
fn test_len_prefix_overflow_returns_error() {
    let overflow_len = (u32::MAX as usize) + 1;
    let res = len_prefix(overflow_len);
    assert!(
        matches!(res, Err(CryptoError::InvalidLengthPrefix(_))),
        "len_prefix MUST return CryptoError::InvalidLengthPrefix for lengths > u32::MAX"
    );
}

#[test]
#[allow(deprecated)]
fn test_v2_is_read_only_replay_path_documented() {
    let key = b"integrity-key-32-bytes-v2-test-";

    // Compute V2 HMAC checksum without tx_id and length prefixes
    let mut mac = WalHmac::new(key).unwrap();
    mac.update(&[0u8; 32]);
    mac.update(&1u64.to_le_bytes()); // seq_no
    mac.update(&[0u8]); // Put op
    mac.update(b"v2key");
    mac.update(b"v2val");
    let checksum = mac.finalize();

    let v2_snapshot = WalEntrySnapshot {
        tx_id: 0,
        seq_no: 1,
        op_type: 0,
        key: b"v2key".to_vec(),
        value: b"v2val".to_vec(),
        checksum,
        prev_hmac: [0u8; 32],
    };

    // 1. Dedicated legacy replay method verify_and_update_v2 MUST accept the V2 snapshot
    let mut verifier_v2 = IntegrityVerifier::new(key);
    assert!(
        verifier_v2.verify_and_update_v2(&v2_snapshot, 0).is_ok(),
        "Legacy replay method verify_and_update_v2 MUST accept valid V2 entry"
    );

    // 2. Default verifier method verify_and_update (which uses V3 with tx_id and length prefixes)
    // MUST reject the V2 snapshot due to HMAC mismatch
    let mut verifier_default = IntegrityVerifier::new(key);
    assert!(
        verifier_default.verify_and_update(&v2_snapshot, 0).is_err(),
        "Default verifier (V3) MUST reject legacy V2 snapshot format"
    );
}
