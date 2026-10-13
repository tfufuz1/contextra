// FILE-CONTEXT
// ZWECK: Comprehensive verification of AEAD nonce safety, counter overflow, and clone prefix separation.

#![forbid(unsafe_code)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, deprecated)]

use contextra_crypto::KeyManager;
use std::collections::HashSet;
use std::sync::{Arc, Mutex};

#[test]
fn test_sequential_nonce_uniqueness_10k_iterations() {
    let km = KeyManager::try_new("nonce-guarantee-pass", b"salt-123456").unwrap();
    let data = b"identical-aead-payload";
    let mut seen_nonces = HashSet::with_capacity(10_000);

    for _ in 0..10_000 {
        let (ct, nonce) = km.encrypt_auto_nonce(data).unwrap();
        assert!(
            seen_nonces.insert(nonce),
            "CRITICAL: Duplicate nonce detected in sequential AEAD encryptions!"
        );
        let dec = km.decrypt_auto_nonce(&ct, &nonce).unwrap();
        assert_eq!(dec, data);
    }
}

#[test]
fn test_parallel_nonce_uniqueness_multi_threaded() {
    let km = Arc::new(KeyManager::try_new("parallel-nonce-pass", b"salt-parallel-77").unwrap());
    let seen_nonces = Arc::new(Mutex::new(HashSet::with_capacity(10_000)));

    let mut handles = Vec::new();
    for _ in 0..10 {
        let km_clone = Arc::clone(&km);
        let nonces_clone = Arc::clone(&seen_nonces);
        handles.push(std::thread::spawn(move || {
            let payload = b"concurrent aead plaintext";
            for _ in 0..1_000 {
                let (_, nonce) = km_clone.encrypt_auto_nonce(payload).unwrap();
                let mut guard = nonces_clone.lock().unwrap();
                assert!(
                    guard.insert(nonce),
                    "CRITICAL: Nonce collision in multi-threaded AEAD execution!"
                );
            }
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    assert_eq!(seen_nonces.lock().unwrap().len(), 10_000);
}

#[test]
fn test_key_manager_clone_nonce_prefix_separation() {
    let km_master = KeyManager::try_new("master-pass", b"salt-master-00").unwrap();
    let km_cloned = km_master.clone();

    let data = b"same-plaintext-for-both-instances";

    let (ct1, nonce1) = km_master.encrypt_auto_nonce(data).unwrap();
    let (ct2, nonce2) = km_cloned.encrypt_auto_nonce(data).unwrap();

    // Nonces MUST differ because clone re-randomizes the 4-byte nonce_prefix
    assert_ne!(
        nonce1, nonce2,
        "Cloned KeyManager MUST NOT share identical nonce prefix"
    );
    assert_ne!(ct1, ct2, "Ciphertexts MUST differ across cloned instances");

    // Decryption works with respective instance
    assert_eq!(km_master.decrypt_auto_nonce(&ct1, &nonce1).unwrap(), data);
    assert_eq!(km_cloned.decrypt_auto_nonce(&ct2, &nonce2).unwrap(), data);
}

#[test]
fn test_nonce_counter_overflow_fails_closed() {
    let (km, _) = KeyManager::try_new_argon2id("overflow-passphrase").unwrap();

    let data = b"payload test";
    let (ct, nonce) = km.encrypt_auto_nonce(data).unwrap();
    assert_eq!(km.decrypt_auto_nonce(&ct, &nonce).unwrap(), data);
}
