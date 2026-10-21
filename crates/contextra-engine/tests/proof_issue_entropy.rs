//! Integration test verifying Ed25519 signer seed entropy generation (SEC-1 / F-01).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use contextra_engine::deletion::proof_issue::{FileOrCsprngSignerSource, ProofSignerSource};
use contextra_sys::OsCryptoRng;
use tempfile::tempdir;

#[test]
fn test_proof_issue_os_crypto_rng_entropy_uniqueness() {
    let rng = OsCryptoRng::new();
    let mut seed1 = [0u8; 32];
    let mut seed2 = [0u8; 32];

    rng.try_fill_bytes(&mut seed1)
        .expect("OsCryptoRng entropy generation must succeed");
    rng.try_fill_bytes(&mut seed2)
        .expect("OsCryptoRng entropy generation must succeed");

    assert_ne!(
        seed1, [0u8; 32],
        "Generated CSPRNG seed 1 must not be all zeros"
    );
    assert_ne!(
        seed2, [0u8; 32],
        "Generated CSPRNG seed 2 must not be all zeros"
    );
    assert_ne!(
        seed1, seed2,
        "Two independent OsCryptoRng draws must produce distinct secret seeds"
    );
}

#[test]
fn test_file_or_csprng_signer_source_generates_distinct_seeds() {
    let dir = tempdir().expect("tempdir creation failed");
    let key_path1 = dir.path().join("signer1.key");
    let key_path2 = dir.path().join("signer2.key");

    let source1 = FileOrCsprngSignerSource::new(key_path1.clone());
    let source2 = FileOrCsprngSignerSource::new(key_path2.clone());

    let seed1 = source1
        .get_signer_seed()
        .expect("get_signer_seed for source1 failed");
    let seed2 = source2
        .get_signer_seed()
        .expect("get_signer_seed for source2 failed");

    assert_ne!(
        *seed1, [0u8; 32],
        "Signer seed 1 must be non-zero CSPRNG entropy"
    );
    assert_ne!(
        *seed2, [0u8; 32],
        "Signer seed 2 must be non-zero CSPRNG entropy"
    );
    assert_ne!(
        *seed1, *seed2,
        "Independent FileOrCsprngSignerSource calls must produce distinct seeds"
    );

    // Verify persistence: loading source1 again reads the previously saved key file
    let source1_reloaded = FileOrCsprngSignerSource::new(key_path1);
    let seed1_reloaded = source1_reloaded
        .get_signer_seed()
        .expect("Reloading source1 key file failed");

    assert_eq!(
        *seed1, *seed1_reloaded,
        "Reloading existing key file must return identical secret seed"
    );
}
