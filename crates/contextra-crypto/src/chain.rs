//! Appendlog hash chains and MAC chains (F.4, F.9, 10.4).
//!
//! Provides cryptographic building blocks for WAL and ledger integrity:
//! - Frame-level authentication via HMAC-SHA256 chains (`wal_mac`).
//! - Cumulative append-log hash chaining (`chain_next`).
//! - Ledger integrity commitment (`ledger_mac`).
//! - Constant-time chain completeness verification (`verify_chain_complete`).
//!
//! ## Reference Example for Consumers (`contextra-store`, `contextra-vector`)
//!
//! Demonstrates WAL chaining, verification, truncation detection, and tampering detection:
//!
//! ```rust
//! use contextra_crypto::{chain_next, wal_mac, verify_chain_complete, MasterKey, KeyHierarchy, CryptoError};
//!
//! let hierarchy = KeyHierarchy::from_master(MasterKey::generate_random());
//! let (mut prev, ct1, ct2, tag) = ([0u8; 32], b"batch_1", b"batch_2", [0xAA; 16]);
//!
//! // 1. Chain frames
//! let mac1 = wal_mac(hierarchy.wal(), &prev, ct1.len() as u32, 1, ct1, &tag);
//! let mac2 = wal_mac(hierarchy.wal(), &mac1, ct2.len() as u32, 2, ct2, &tag);
//!
//! // 2. Completeness verification against expected tail
//! assert!(verify_chain_complete(&mac2, &mac2).is_ok());
//!
//! // 3. Truncation detection (chain ended prematurely at mac1)
//! assert_eq!(verify_chain_complete(&mac2, &mac1), Err(CryptoError::Corrupt));
//!
//! // 4. Tampering detection (modified payload or swapped ordering)
//! let bad_mac = wal_mac(hierarchy.wal(), &mac1, ct2.len() as u32, 2, b"tampered", &tag);
//! assert_eq!(verify_chain_complete(&mac2, &bad_mac), Err(CryptoError::Corrupt));
//! ```

use crate::error::{CryptoError, Result};
use crate::hierarchy::{LedgerKey, WalKey};
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

/// Advances a SHA-256 hash chain: SHA-256(prev || entry).
///
/// # Inputs
/// * `prev`: 32-byte previous hash state.
/// * `entry`: Serialized entry bytes to append to the chain.
///
/// # Properties
/// Deterministic and collision-resistant. Returns a new 32-byte hash state.
pub fn chain_next(prev: &[u8; 32], entry: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(prev);
    hasher.update(entry);
    hasher.finalize().into()
}

/// Computes the WAL frame MAC: HMAC-SHA256(K_wal, prev_mac || len || seq || ct || aead_tag).
///
/// # Inputs
/// * `k`: Strongly-typed `WalKey` derived from the master key.
/// * `prev`: MAC of the previous frame (or initial segment `prev_mac`).
/// * `len`: Length of ciphertext in bytes.
/// * `seq`: Monotonic sequence number of the WAL frame.
/// * `ct`: Ciphertext slice of length `len`.
/// * `tag`: 16-byte AEAD authentication tag.
///
/// # Properties & Constant-Time
/// Performs HMAC-SHA256 over canonical frame fields. Execution time is independent of key
/// secret material. Guarantees non-panicking execution and returns a 32-byte HMAC tag.
pub fn wal_mac(
    k: &WalKey,
    prev: &[u8; 32],
    len: u32,
    seq: u64,
    ct: &[u8],
    tag: &[u8; 16],
) -> [u8; 32] {
    let mut mac = match Hmac::<Sha256>::new_from_slice(k.0.as_bytes()) {
        Ok(m) => m,
        Err(_) => return [0u8; 32],
    };
    mac.update(prev);
    mac.update(&len.to_be_bytes());
    mac.update(&seq.to_be_bytes());
    mac.update(ct);
    mac.update(tag);
    mac.finalize().into_bytes().into()
}

/// Computes Ledger MAC: HMAC-SHA256(K_ledger, chain).
///
/// # Inputs
/// * `k`: Strongly-typed `LedgerKey`.
/// * `chain`: 32-byte cumulative hash chain state.
///
/// # Properties & Constant-Time
/// Computes HMAC-SHA256 over the chain digest with execution time independent of key secrets.
pub fn ledger_mac(k: &LedgerKey, chain: &[u8; 32]) -> [u8; 32] {
    let mut mac = match Hmac::<Sha256>::new_from_slice(k.0.as_bytes()) {
        Ok(m) => m,
        Err(_) => return [0u8; 32],
    };
    mac.update(chain);
    mac.finalize().into_bytes().into()
}

/// Verifies completeness of a WAL hash/MAC chain against an expected high-water mark or tail.
///
/// Evaluates equivalence using constant-time comparison (`subtle::ConstantTimeEq`) to prevent
/// timing side-channel attacks.
///
/// # Inputs
/// * `expected_tail`: Expected high-water mark or checkpoint MAC digest (32 bytes).
/// * `actual_tail`: Replayed or candidate chain tail MAC digest (32 bytes).
///
/// # Errors
/// Returns `Err(CryptoError::Corrupt)` if the tail does not match the expected state (truncation,
/// omitted frames, reordering, or tampering).
pub fn verify_chain_complete(expected_tail: &[u8; 32], actual_tail: &[u8; 32]) -> Result<()> {
    if expected_tail.ct_eq(actual_tail).unwrap_u8() == 1 {
        Ok(())
    } else {
        Err(CryptoError::Corrupt)
    }
}

/// Re-export alias for storage recovery verification per SPEC 10.4.
#[inline]
pub fn verify_wal_chain_completeness(expected: &[u8; 32], actual: &[u8; 32]) -> Result<()> {
    verify_chain_complete(expected, actual)
}
