//! J09 Security tests: Ed25519 signer key management, CSPRNG generation, permissions, and concurrency.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_engine::deletion::proof_issue::{FileOrCsprngSignerSource, ProofSignerSource};
use contextra_types::ContextraError;
use std::fs;
use std::sync::{Arc, Barrier};
use tempfile::tempdir;

#[test]
fn test_signer_key_creation_and_csprng_uniqueness() {
    let dir1 = tempdir().unwrap();
    let dir2 = tempdir().unwrap();

    let key_path1 = dir1.path().join("proof_ed25519.key");
    let key_path2 = dir2.path().join("proof_ed25519.key");

    let source1 = FileOrCsprngSignerSource::new(key_path1.clone());
    let source2 = FileOrCsprngSignerSource::new(key_path2.clone());

    let seed1 = source1.get_signer_seed().unwrap();
    let seed2 = source2.get_signer_seed().unwrap();

    assert_eq!(seed1.len(), 32);
    assert_eq!(seed2.len(), 32);
    assert_ne!(
        *seed1, *seed2,
        "Two distinct CSPRNG key creations MUST generate different seeds"
    );

    // Unix permission verification: 0600
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let meta1 = fs::metadata(&key_path1).unwrap();
        let mode1 = meta1.permissions().mode() & 0o777;
        assert_eq!(mode1, 0o600, "Signer key file mode MUST be 0600 on Unix");
    }

    // Re-opening existing key file returns identical seed
    let seed1_reopened = source1.get_signer_seed().unwrap();
    assert_eq!(
        *seed1, *seed1_reopened,
        "Re-opening existing key file MUST return identical seed"
    );
}

#[test]
fn test_signer_key_creation_race_n_threads() {
    let dir = tempdir().unwrap();
    let key_path = dir.path().join("proof_ed25519.key");

    let num_threads = 10;
    let barrier = Arc::new(Barrier::new(num_threads));
    let mut handles = Vec::new();

    for _ in 0..num_threads {
        let path = key_path.clone();
        let b = Arc::clone(&barrier);
        handles.push(std::thread::spawn(move || {
            let source = FileOrCsprngSignerSource::new(path);
            b.wait();
            source.get_signer_seed().unwrap()
        }));
    }

    let seeds: Vec<_> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    // Exactly one file created on disk
    assert!(key_path.exists());

    // All threads received the EXACT same 32-byte seed
    let first = &seeds[0];
    for seed in &seeds[1..] {
        assert_eq!(
            *seed, *first,
            "All threads racing to create signer key MUST obtain identical seed"
        );
    }
}

#[test]
fn test_corrupt_key_file_fails_closed() {
    let invalid_lengths = vec![0, 1, 31, 33, 64, 128];

    for len in invalid_lengths {
        let dir = tempdir().unwrap();
        let key_path = dir.path().join("proof_ed25519.key");

        // Write invalid size file
        fs::write(&key_path, vec![0xAB; len]).unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perm = fs::metadata(&key_path).unwrap().permissions();
            perm.set_mode(0o600);
            fs::set_permissions(&key_path, perm).unwrap();
        }

        let source = FileOrCsprngSignerSource::new(key_path);
        let res = source.get_signer_seed();

        assert!(
            res.is_err(),
            "Signer key loading MUST fail on invalid file size {}",
            len
        );

        match res {
            Err(ContextraError::Crypto(err)) => {
                assert!(
                    err.contains("32 bytes") || err.contains("Invalid key file length"),
                    "Expected invalid length error, got: {}",
                    err
                );
            }
            other => panic!("Expected ContextraError::Crypto, got {:?}", other),
        }
    }
}

#[test]
fn test_directory_instead_of_key_file_fails_closed() {
    let dir = tempdir().unwrap();
    let key_path = dir.path().join("proof_ed25519.key");

    // Create directory at key_path
    fs::create_dir_all(&key_path).unwrap();

    let source = FileOrCsprngSignerSource::new(key_path);
    let res = source.get_signer_seed();

    assert!(
        res.is_err(),
        "Signer key loading MUST fail if target is a directory"
    );
}
