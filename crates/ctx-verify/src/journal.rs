//! Journal reduction hash chain verification per SPEC F.9.
//!
//! Chain rule: chain_n = SHA-256(chain_prev || len || ct || aead_tag)
//! First entry: chain_0 starts with initial seed or 0^32.

use sha2::{Digest, Sha256};

/// Journal reduction entry representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalChainEntry {
    pub prev_hash: [u8; 32],
    pub entry_payload: Vec<u8>,
    pub current_hash: [u8; 32],
}

impl JournalChainEntry {
    /// Computes the chain hash for the given payload and previous chain hash.
    pub fn compute_chain_step(prev_chain: &[u8; 32], payload_or_raw: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(prev_chain);
        hasher.update(payload_or_raw);
        let digest = hasher.finalize();
        let mut out = [0u8; 32];
        out.copy_from_slice(&digest);
        out
    }
}

/// Verifies a sequence of journal reduction hashes.
pub fn verify_journal_hash_chain(steps: &[([u8; 32], Vec<u8>, [u8; 32])]) -> bool {
    let mut expected_prev = match steps.first() {
        Some((prev, _, _)) => *prev,
        None => return true,
    };

    for (prev, data, curr) in steps {
        if prev != &expected_prev {
            return false;
        }
        let computed = JournalChainEntry::compute_chain_step(prev, data);
        if &computed != curr {
            return false;
        }
        expected_prev = *curr;
    }

    true
}
