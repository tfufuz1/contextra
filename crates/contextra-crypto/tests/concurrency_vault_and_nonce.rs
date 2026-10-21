// FILE-CONTEXT
// ZWECK: Concurrency, Vault Zeroize, and Nonce Uniqueness Hardening Test Suite (SEC-01/K1).
// INVARIANTEN: Zero-panic, thread-safe AuditChain concurrency under Mutex, non-colliding nonces, zeroize safety.

#![forbid(unsafe_code)]
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use contextra_crypto::{
    anti_tamper::VolatileEncryptionKey,
    audit_chain::{AuditChain, DataClass},
    crypto::KeyManager,
    kv_shredding::KeyRegistry,
    CryptoKey,
};
use contextra_types::{DocId, TxId};
use std::collections::HashSet;
use std::sync::{Arc, Barrier, Mutex};
use std::thread;

#[test]
fn test_parallel_audit_chain_appends() {
    const THREADS: usize = 10;
    const APPENDS_PER_THREAD: usize = 100;
    const TOTAL_APPENDS: usize = THREADS * APPENDS_PER_THREAD;

    let chain = Arc::new(Mutex::new(AuditChain::new()));
    let barrier = Arc::new(Barrier::new(THREADS));
    let mut handles = Vec::new();

    for t_id in 0..THREADS {
        let chain_cloned = Arc::clone(&chain);
        let barrier_cloned = Arc::clone(&barrier);

        handles.push(thread::spawn(move || {
            barrier_cloned.wait();

            for i in 0..APPENDS_PER_THREAD {
                let doc_num = (t_id * APPENDS_PER_THREAD + i) as u64;
                let mut guard = chain_cloned.lock().expect("lock audit chain");
                guard
                    .append(
                        "parallel_schema",
                        [0xAA; 32],
                        DataClass::Confidential,
                        DocId(doc_num),
                        TxId(doc_num + 1000),
                        None,
                    )
                    .expect("append to audit chain");
            }
        }));
    }

    for handle in handles {
        handle.join().expect("thread join");
    }

    let final_chain = chain.lock().expect("lock final chain");
    assert_eq!(final_chain.len(), TOTAL_APPENDS);

    // Verify contiguous indices 0..TOTAL_APPENDS
    let mut indices_seen = HashSet::new();
    for (expected_idx, entry) in final_chain.entries().iter().enumerate() {
        assert_eq!(entry.index, expected_idx as u64);
        assert!(indices_seen.insert(entry.index), "Duplicate index found!");
    }
    assert_eq!(indices_seen.len(), TOTAL_APPENDS);

    // Complete cryptographic chain verification
    assert!(
        final_chain.verify_chain().expect("verify_chain"),
        "Parallel appended AuditChain MUST satisfy cryptographic hash linkage"
    );
}

#[test]
fn test_aead_nonce_uniqueness_multithreaded() {
    const THREADS: usize = 8;
    const MSGS_PER_THREAD: usize = 1250; // Total 10,000 nonces
    const TOTAL_MSGS: usize = THREADS * MSGS_PER_THREAD;

    let key =
        Arc::new(CryptoKey::try_new("aead-nonce-passphrase", b"aead-salt-777").expect("CryptoKey"));
    let nonce_set = Arc::new(Mutex::new(HashSet::with_capacity(TOTAL_MSGS)));
    let barrier = Arc::new(Barrier::new(THREADS));
    let mut handles = Vec::new();

    for _ in 0..THREADS {
        let key_clone = Arc::clone(&key);
        let set_clone = Arc::clone(&nonce_set);
        let barrier_clone = Arc::clone(&barrier);

        handles.push(thread::spawn(move || {
            barrier_clone.wait();
            let payload = b"sensitive concurrent payload";

            for _ in 0..MSGS_PER_THREAD {
                let (_, nonce) = key_clone.encrypt_auto_nonce(payload).expect("encrypt");
                let mut set_guard = set_clone.lock().expect("lock nonce set");
                assert!(
                    set_guard.insert(nonce),
                    "CRITICAL: Nonce collision detected!"
                );
            }
        }));
    }

    for handle in handles {
        handle.join().expect("thread join");
    }

    let total_uniques = nonce_set.lock().expect("lock set").len();
    assert_eq!(
        total_uniques, TOTAL_MSGS,
        "All generated nonces MUST be distinct"
    );
}

#[test]
fn test_vault_concurrency_and_zeroize_safety() {
    const THREADS: usize = 8;
    const ITERATIONS: usize = 50;

    let km = Arc::new(KeyManager::try_new("vault-passphrase", b"vault-salt").expect("km"));
    let registry = Arc::new(KeyRegistry::new());
    let barrier = Arc::new(Barrier::new(THREADS));
    let mut handles = Vec::new();

    for t_id in 0..THREADS {
        let km_clone = Arc::clone(&km);
        let registry_clone = Arc::clone(&registry);
        let barrier_clone = Arc::clone(&barrier);

        handles.push(thread::spawn(move || {
            barrier_clone.wait();
            let group_id = 100 + t_id as u64;

            for i in 0..ITERATIONS {
                let payload = format!("Payload from thread {t_id} iteration {i}");
                let enc_res = registry_clone.encrypt_record(
                    &km_clone,
                    group_id,
                    i as u64,
                    payload.as_bytes(),
                );

                match enc_res {
                    Ok(enc) => {
                        let dec_res = registry_clone.decrypt_record(&km_clone, &enc);
                        if let Ok(dec) = dec_res {
                            assert_eq!(dec, payload.as_bytes());
                        } else {
                            // If decryption failed, group was revoked concurrently
                            assert!(registry_clone.is_group_revoked(group_id));
                        }
                    }
                    Err(err) => {
                        // After revocation, encryption fails closed with KeyRevoked
                        assert!(matches!(
                            err,
                            contextra_crypto::error::CryptoError::KeyRevoked(_)
                        ));
                    }
                }

                // Periodically trigger revocation in thread 0 for group 100
                if t_id == 0 && i == ITERATIONS / 2 {
                    let _ = registry_clone.revoke_group(group_id);
                }
            }
        }));
    }

    for handle in handles {
        handle.join().expect("thread join");
    }

    // Verify VolatileEncryptionKey zeroization and drop semantics under concurrency
    let key_bytes = [0x77u8; 32];
    let volatile_key = Arc::new(Mutex::new(VolatileEncryptionKey::new(key_bytes)));

    let mut wipe_handles = Vec::new();
    let wipe_barrier = Arc::new(Barrier::new(4));

    for _ in 0..4 {
        let vk_clone = Arc::clone(&volatile_key);
        let wb_clone = Arc::clone(&wipe_barrier);

        wipe_handles.push(thread::spawn(move || {
            wb_clone.wait();
            let mut vk = vk_clone.lock().expect("lock volatile_key");
            vk.emergency_wipe();
        }));
    }

    for h in wipe_handles {
        h.join().expect("wipe thread join");
    }

    let final_vk = volatile_key.lock().expect("lock final_vk");
    assert_eq!(
        final_vk.inspect_key_bytes_for_test(),
        &[0x00u8; 32],
        "Secret material MUST be zeroized after emergency_wipe"
    );
}
