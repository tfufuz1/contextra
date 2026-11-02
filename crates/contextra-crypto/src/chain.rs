//! Appendlog hash chains and MAC chains (F.4, F.9, 10.4).

use crate::hierarchy::{LedgerKey, WalKey};
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};

/// Advances a SHA-256 hash chain: SHA-256(prev || entry).
pub fn chain_next(prev: &[u8; 32], entry: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(prev);
    hasher.update(entry);
    hasher.finalize().into()
}

/// Computes the WAL frame MAC: HMAC-SHA256(K_wal, prev_mac || len || seq || ct || aead_tag).
pub fn wal_mac(
    k: &WalKey,
    prev: &[u8; 32],
    len: u32,
    seq: u64,
    ct: &[u8],
    tag: &[u8; 16],
) -> [u8; 32] {
    let mut mac = Hmac::<Sha256>::new_from_slice(k.0.as_bytes()).expect("hmac key");
    mac.update(prev);
    mac.update(&len.to_be_bytes());
    mac.update(&seq.to_be_bytes());
    mac.update(ct);
    mac.update(tag);
    mac.finalize().into_bytes().into()
}

/// Computes Ledger MAC: HMAC-SHA256(K_ledger, chain).
pub fn ledger_mac(k: &LedgerKey, chain: &[u8; 32]) -> [u8; 32] {
    let mut mac = Hmac::<Sha256>::new_from_slice(k.0.as_bytes()).expect("hmac key");
    mac.update(chain);
    mac.finalize().into_bytes().into()
}
