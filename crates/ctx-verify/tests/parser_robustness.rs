//! Parser robustness and fuzzing verification per SPEC T-FUZZ-1, T-FMT-1, HR-2.
//!
//! Verifies that parsers for canonical JSON, report payloads, journal hash chains,
//! and residual risk catalogs never panic when processing arbitrary, malformed,
//! truncated, or corrupted input byte streams.

use ctx_verify::canonical_json::verify_canonical_json;
use ctx_verify::journal::{verify_journal_hash_chain, JournalChainEntry};
use ctx_verify::report::verify_report_bytes;
use ctx_verify::residual_risk::verify_residual_risks;
use ed25519_dalek::SigningKey;
use rand::rngs::StdRng;
use rand::{RngCore, SeedableRng};

/// SPEC: T-FUZZ-1, HR-2 (Zero panic on arbitrary byte inputs for JSON canonical parser)
#[test]
fn t_fuzz_1_canonical_json_random_bytes_no_panic() {
    let mut rng = StdRng::seed_from_u64(0xdeadbeef12345678);
    let iterations = 1_000;

    for len in [0, 1, 2, 7, 16, 64, 256, 1024, 4096] {
        for _ in 0..iterations {
            let mut buf = vec![0u8; len];
            rng.fill_bytes(&mut buf);
            let _ = verify_canonical_json(&buf);
        }
    }
}

/// SPEC: T-FUZZ-1, HR-2 (Zero panic on arbitrary byte inputs for report verifier)
#[test]
fn t_fuzz_1_report_bytes_random_bytes_no_panic() {
    let mut rng = StdRng::seed_from_u64(0xfeedface87654321);
    let mut key_bytes = [0u8; 32];
    rng.fill_bytes(&mut key_bytes);
    let signing_key = SigningKey::from_bytes(&key_bytes);
    let verifying_key = signing_key.verifying_key();

    let iterations = 500;
    for len in [0, 1, 5, 32, 128, 512, 2048] {
        for _ in 0..iterations {
            let mut buf = vec![0u8; len];
            rng.fill_bytes(&mut buf);
            let _ = verify_report_bytes(&buf, &verifying_key, None);
        }
    }
}

/// SPEC: T-FUZZ-1, HR-2 (Zero panic on arbitrary journal chain inputs)
#[test]
fn t_fuzz_1_journal_hash_chain_random_bytes_no_panic() {
    let mut rng = StdRng::seed_from_u64(0x1337c0de);
    for _ in 0..500 {
        let mut prev = [0u8; 32];
        rng.fill_bytes(&mut prev);
        let mut curr = [0u8; 32];
        rng.fill_bytes(&mut curr);
        let mut payload = vec![0u8; (rng.next_u32() % 256) as usize];
        rng.fill_bytes(&mut payload);

        let _ = JournalChainEntry::compute_chain_step(&prev, &payload);

        let steps = vec![(prev, payload, curr)];
        let _ = verify_journal_hash_chain(&steps);
    }
}

/// SPEC: T-FUZZ-1, HR-2 (Zero panic on arbitrary strings for residual risks)
#[test]
fn t_fuzz_1_residual_risks_random_strings_no_panic() {
    let mut rng = StdRng::seed_from_u64(0xcafebebe);
    for _ in 0..500 {
        let len = (rng.next_u32() % 32) as usize;
        let mut bytes = vec![0u8; len];
        rng.fill_bytes(&mut bytes);
        let s = String::from_utf8_lossy(&bytes).to_string();
        let _ = verify_residual_risks(&[s]);
    }
}
