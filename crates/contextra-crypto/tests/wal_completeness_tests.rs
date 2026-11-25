// FILE-CONTEXT
// ZWECK: Unit & proptest verification for WAL chain completeness checks.
// INVARIANTEN: Zero false negatives, zero false positives, constant-time verification.

#![forbid(unsafe_code)]

use contextra_crypto::{verify_wal_chain_completeness, CryptoError};
use proptest::prelude::*;

#[test]
fn test_verify_wal_chain_completeness_ok_when_identical() {
    let hmac = [0xA5u8; 32];
    let result = verify_wal_chain_completeness(&hmac, &hmac);
    assert!(result.is_ok());
}

#[test]
fn test_verify_wal_chain_completeness_err_when_truncated() {
    let hmac_a = [0x12u8; 32];
    for byte_idx in 0..32 {
        for bit_idx in 0..8 {
            let mut hmac_b = hmac_a;
            hmac_b[byte_idx] ^= 1 << bit_idx;
            let result = verify_wal_chain_completeness(&hmac_a, &hmac_b);
            assert!(
                matches!(result, Err(CryptoError::WalTruncationDetected)),
                "Bit flip at byte {byte_idx}, bit {bit_idx} MUST trigger WalTruncationDetected"
            );
        }
    }
}

proptest! {
    #[test]
    fn prop_wal_chain_completeness_equivalence(a: [u8; 32], b: [u8; 32]) {
        let res = verify_wal_chain_completeness(&a, &b);
        if a == b {
            prop_assert!(res.is_ok());
        } else {
            prop_assert!(matches!(res, Err(CryptoError::WalTruncationDetected)));
        }
    }
}
