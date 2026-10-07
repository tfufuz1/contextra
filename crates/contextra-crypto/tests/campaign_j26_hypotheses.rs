// FILE-CONTEXT
// ZWECK: Campaign J-26 hypothesis test suite for crypto compliance, revocation logs, audit chains, commitments, and deletion proofs.
// INVARIANTEN: Zero-Panic doctrine in non-test code, fail-closed revocation checks, length-prefixed deletion proof hashes, and Blake3 audit chain integrity.

#![forbid(unsafe_code)]

use contextra_crypto::{
    anti_tamper::VolatileEncryptionKey,
    audit_chain::{compute_record_commitment, AuditChain, DataClass, EncryptedCommitmentSalt},
    crypto::KeyManager,
    deletion_proof::{
        compute_wal_delete_receipt, hash_deleted_keys_length_prefixed, verify_wal_delete_receipt,
        DeletionLayer, DeletionProof, DeletionProofKeyPair, DeletionScope, ExcludedScope,
        GraphRepairAttestation, LayerCleanupProof,
    },
    ed25519_proof::SignatureVersion,
    error::CryptoError,
    kv_shredding::KeyRegistry,
    revocation_log::{RevocationLog, RevocationTarget},
};
use contextra_ports::SystemClock;
use contextra_types::{DocId, TenantId, TxId};
use ed25519_dalek::SigningKey;
use rand::{rngs::OsRng, RngCore};
use std::collections::HashSet;
use std::sync::Arc;
use tempfile::NamedTempFile;

fn test_ed25519_keypair() -> (SigningKey, ed25519_dalek::VerifyingKey) {
    let sk = SigningKey::generate(&mut OsRng);
    let vk = sk.verifying_key();
    (sk, vk)
}

// ---------------------------------------------------------------------------
// H1: KeyRegistry revocation does not write to RevocationLog (Split-Brain)
// ---------------------------------------------------------------------------

#[test]
fn test_h1_key_registry_revoke_group_missing_log_persistence() {
    let tmp_file = NamedTempFile::new().expect("create temp file");
    let log_path = tmp_file.path().to_path_buf();
    let (sk, vk) = test_ed25519_keypair();
    let clock = Arc::new(SystemClock::new());

    let rev_log = Arc::new(
        RevocationLog::open_or_create(&log_path, clock.clone(), Some(sk), vk)
            .expect("open revocation log"),
    );

    let km = KeyManager::try_new("test-passphrase-h1", b"salt-h1").expect("keymanager");
    let registry = KeyRegistry::new().with_revocation_log(rev_log.clone());
    let group_id = 999;

    // Derive KEK
    let _subkey = registry
        .get_or_derive(&km, group_id)
        .expect("derive subkey");
    assert!(registry.is_group_active(group_id));

    // Revoke group in registry
    let revoked = registry.revoke_group(group_id).expect("revoke_group");
    assert!(revoked, "revoke_group must return true for active group");

    // In RAM, group is revoked
    assert!(registry.is_group_revoked(group_id));

    // Drop registry and simulate process crash / restart by opening log file anew
    drop(registry);
    drop(rev_log);

    let reloaded_log = Arc::new(
        RevocationLog::open_or_create(&log_path, clock, None, vk).expect("reload revocation log"),
    );
    let new_registry = KeyRegistry::new().with_revocation_log(reloaded_log.clone());

    // EXPECTATION: The revocation SHOULD be reconstructed from the log after crash.
    // DEFECT: KeyRegistry::revoke_group never calls revocation_log.append(), so log is empty
    // and new_registry thinks group 999 is NOT revoked!
    assert!(
        new_registry.is_group_revoked(group_id),
        "Group 999 MUST be recognized as revoked after reload from RevocationLog"
    );
}

#[test]
fn test_h1_demonstrate_decoupling_between_registry_and_revocation_log() {
    let tmp_file = NamedTempFile::new().expect("create temp file");
    let log_path = tmp_file.path().to_path_buf();
    let (sk, vk) = test_ed25519_keypair();
    let clock = Arc::new(SystemClock::new());

    let rev_log = Arc::new(
        RevocationLog::open_or_create(&log_path, clock, Some(sk), vk).expect("open revocation log"),
    );

    let km = KeyManager::try_new("test-passphrase-h1-demo", b"salt-h1").expect("keymanager");
    let registry = KeyRegistry::new().with_revocation_log(rev_log.clone());
    let group_id = 888;

    let _subkey = registry.get_or_derive(&km, group_id).expect("derive");
    registry.revoke_group(group_id).expect("revoke_group");

    // After fix: rev_log.len() is 1 because revoke_group appended to RevocationLog
    assert_eq!(
        rev_log.len(),
        1,
        "KeyRegistry::revoke_group MUST append to RevocationLog"
    );
}

// ---------------------------------------------------------------------------
// H2: Revocation cascade, scope isolation, and resurrection prevention
// ---------------------------------------------------------------------------

#[test]
fn test_h2_revocation_cascade_isolation_and_resurrection_prevention() {
    let km = KeyManager::try_new("test-passphrase-h2", b"salt-h2").expect("keymanager");
    let registry = KeyRegistry::new();

    let group_1 = 101;
    let group_2 = 102;

    // Encrypt records in group 1
    let payload_1_1 = registry
        .encrypt_record(&km, group_1, 1, b"Group 1 Record 1")
        .expect("encrypt 1_1");
    let payload_1_2 = registry
        .encrypt_record(&km, group_1, 2, b"Group 1 Record 2")
        .expect("encrypt 1_2");

    // Encrypt records in group 2
    let payload_2_1 = registry
        .encrypt_record(&km, group_2, 1, b"Group 2 Record 1")
        .expect("encrypt 2_1");
    let payload_2_2 = registry
        .encrypt_record(&km, group_2, 2, b"Group 2 Record 2")
        .expect("encrypt 2_2");

    // 1. Revoke group 1
    assert!(registry.revoke_group(group_1).unwrap());

    // Verify group 1 status flags
    assert!(registry.is_group_revoked(group_1));
    assert!(!registry.is_group_active(group_1));
    assert!(!registry.is_key_active(group_1));
    assert!(!registry.is_record_active(group_1, 1));
    assert!(!registry.is_record_active(group_1, 2));

    // Decryption of ALL records in group 1 MUST fail
    assert!(registry.decrypt_record(&km, &payload_1_1).is_err());
    assert!(registry.decrypt_record(&km, &payload_1_2).is_err());

    // 2. Group 2 MUST remain completely active and unaffected
    assert!(!registry.is_group_revoked(group_2));
    assert!(registry.is_group_active(group_2));
    assert_eq!(
        registry.decrypt_record(&km, &payload_2_1).expect("dec 2_1"),
        b"Group 2 Record 1"
    );
    assert_eq!(
        registry.decrypt_record(&km, &payload_2_2).expect("dec 2_2"),
        b"Group 2 Record 2"
    );

    // 3. Single record revocation in group 2
    assert!(registry.revoke_record(group_2, 1).unwrap());
    assert!(!registry.is_record_active(group_2, 1));
    assert!(registry.is_record_active(group_2, 2));

    // Decrypting record 1 in group 2 MUST fail
    assert!(registry.decrypt_record(&km, &payload_2_1).is_err());
    // Decrypting neighbor record 2 in group 2 MUST succeed
    assert_eq!(
        registry.decrypt_record(&km, &payload_2_2).expect("dec 2_2"),
        b"Group 2 Record 2"
    );

    // 4. Resurrection Prevention: get_or_derive MUST NOT allow key resurrection for group 1
    let resurrection_attempt = registry.get_or_derive(&km, group_1);
    assert!(
        matches!(resurrection_attempt, Err(CryptoError::KeyRevoked(_))),
        "Re-derivation of key for revoked group MUST be rejected"
    );
}

// ---------------------------------------------------------------------------
// H3: RevocationLog integrity verification matrix
// ---------------------------------------------------------------------------

#[test]
fn test_h3_revocation_log_tamper_matrix() {
    let (sk, vk) = test_ed25519_keypair();
    let clock = Arc::new(SystemClock::new());

    // Construct a log with 10 entries for thorough sampling
    let tmp_file = NamedTempFile::new().expect("tempfile");
    let log_path = tmp_file.path().to_path_buf();

    let log = RevocationLog::open_or_create(&log_path, clock.clone(), Some(sk), vk).expect("log");
    for i in 0..10 {
        log.append(RevocationTarget::Group(i)).expect("append");
    }
    assert_eq!(log.len(), 10);
    assert!(log.verify_integrity().is_ok());

    let original_bytes = std::fs::read(&log_path).expect("read raw log bytes");
    let parsed_entries: Vec<contextra_crypto::revocation_log::RevocationEntry> =
        bincode::deserialize(&original_bytes).expect("deserialize entries");

    // Matrix Test 1: Bit-flips across serialized byte representation (sampling >= 500 positions)
    let mut detected_tamper = 0;
    let mut total_flips = 0;
    let step = (original_bytes.len() / 500).max(1);
    for idx in (0..original_bytes.len()).step_by(step) {
        for bit_shift in 0..8 {
            total_flips += 1;
            let mut tampered = original_bytes.clone();
            tampered[idx] ^= 1 << bit_shift;

            // Write tampered bytes directly to disk file and test open_or_create
            std::fs::write(&log_path, &tampered).expect("write tampered");
            match RevocationLog::open_or_create(&log_path, clock.clone(), None, vk) {
                Err(_) => detected_tamper += 1,
                Ok(reopened_log) => {
                    // If open_or_create succeeded, check if the 10 entries are still intact and verified.
                    if reopened_log.len() == 10 && reopened_log.verify_integrity().is_ok() {
                        // Unnoticed tamper that produced a valid 10-entry log
                    } else {
                        // Tamper altered/wiped entries fail-closed detection
                        detected_tamper += 1;
                    }
                }
            }
        }
    }
    // Restore original file
    std::fs::write(&log_path, &original_bytes).expect("restore");

    assert_eq!(
        detected_tamper, total_flips,
        "Every single bit-flip MUST be detected fail-closed"
    );

    // Matrix Test 2: Entry deletion in beginning/middle (0, 5) MUST fail due to sequence number gap
    for delete_idx in [0, 5] {
        let mut tampered_entries = parsed_entries.clone();
        tampered_entries.remove(delete_idx);
        let tampered_bytes = bincode::serialize(&tampered_entries).expect("serialize");
        std::fs::write(&log_path, &tampered_bytes).expect("write");

        assert!(
            RevocationLog::open_or_create(&log_path, clock.clone(), None, vk).is_err(),
            "Deleting entry at index {delete_idx} MUST be detected as an integrity violation"
        );
    }

    // Tail Deletion Vulnerability (BEFUND H3 / FIXED in W1-02): Deleting the LAST entry (index 9)
    // causes log_count (9) < marker.count (10), which MUST be rejected as IntegrityViolation!
    {
        let mut tampered_entries = parsed_entries.clone();
        tampered_entries.remove(9);
        let tampered_bytes = bincode::serialize(&tampered_entries).expect("serialize");
        std::fs::write(&log_path, &tampered_bytes).expect("write");

        let reopened_res = RevocationLog::open_or_create(&log_path, clock.clone(), None, vk);
        assert!(
            matches!(reopened_res, Err(CryptoError::IntegrityViolation)),
            "Deleting the tail entry causes log_count < marker.count and MUST return IntegrityViolation"
        );
    }

    // Matrix Test 3: Entry swap (Swap 3 and 4)
    {
        let mut tampered_entries = parsed_entries.clone();
        tampered_entries.swap(3, 4);
        let tampered_bytes = bincode::serialize(&tampered_entries).expect("serialize");
        std::fs::write(&log_path, &tampered_bytes).expect("write");

        assert!(
            RevocationLog::open_or_create(&log_path, clock.clone(), None, vk).is_err(),
            "Swapping adjacent entries MUST be detected as an integrity violation"
        );
    }

    // Matrix Test 4: Entry duplication
    {
        let mut tampered_entries = parsed_entries.clone();
        tampered_entries.insert(2, parsed_entries[2].clone());
        let tampered_bytes = bincode::serialize(&tampered_entries).expect("serialize");
        std::fs::write(&log_path, &tampered_bytes).expect("write");

        assert!(
            RevocationLog::open_or_create(&log_path, clock.clone(), None, vk).is_err(),
            "Duplicating an entry MUST be detected as an integrity violation"
        );
    }

    // Matrix Test 5: File truncation
    {
        let truncated_bytes = &original_bytes[..original_bytes.len() / 2];
        std::fs::write(&log_path, truncated_bytes).expect("write");

        assert!(
            RevocationLog::open_or_create(&log_path, clock.clone(), None, vk).is_err(),
            "Truncating the log file in the middle MUST be detected fail-closed"
        );
    }

    // Matrix Test 6: Wrong verifying key
    {
        std::fs::write(&log_path, &original_bytes).expect("write back original");
        let (_wrong_sk, wrong_vk) = test_ed25519_keypair();

        assert!(
            RevocationLog::open_or_create(&log_path, clock, None, wrong_vk).is_err(),
            "Opening log with wrong verifying key MUST fail integrity verification"
        );
    }
}

// ---------------------------------------------------------------------------
// H4: AuditChain Backup Rollback Replay & Tampering Matrix
// ---------------------------------------------------------------------------

#[test]
#[ignore = "SEC-01/K1: AuditChain::verify_chain accepts rolled-back backup state (lack of external head anchor)"]
fn test_h4_audit_chain_backup_rollback_vulnerability() {
    let (sk, vk) = test_ed25519_keypair();
    let mut chain = AuditChain::new();

    // Append 10 entries
    for i in 1..=10 {
        chain
            .append(
                "schema_v1",
                [0x11; 32],
                DataClass::Confidential,
                DocId(i),
                TxId(i),
                None,
            )
            .expect("append");
    }

    // Capture backup state at entry 10
    let backup_at_10 = chain.clone();
    let head_sig_10 = backup_at_10.sign_head(&sk).expect("sign head 10");

    // Continue chain up to entry 20
    for i in 11..=20 {
        chain
            .append(
                "schema_v1",
                [0x11; 32],
                DataClass::Confidential,
                DocId(i),
                TxId(i),
                None,
            )
            .expect("append");
    }

    // Attacker rolls back chain to backup_at_10
    let rolled_back_chain = backup_at_10;

    // EXPECTATION: Rollback should be detectable via an external anchor (e.g. system state counter).
    // DEFECT: Without an external anchor, verify_chain and verify_head_signature BOTH return true for the old backup!
    assert!(
        rolled_back_chain.verify_chain().expect("verify"),
        "Rolled back chain internally satisfies Blake3 hash linkage"
    );
    assert!(
        AuditChain::verify_head_signature(&head_sig_10, &vk).expect("verify sig"),
        "Head signature for backup at 10 is cryptographically valid"
    );

    // This assertion fails if there is no external state anchor tracking chain length >= 20
    assert_eq!(
        rolled_back_chain.len(),
        20,
        "BEFUND H4: Rolled-back chain cannot be distinguished from current head without external anchor"
    );
}

#[test]
fn test_h4_audit_chain_tamper_detection_matrix() {
    let (sk, vk) = test_ed25519_keypair();
    let mut chain = AuditChain::new();

    for i in 1..=10 {
        chain
            .append(
                "schema_v1",
                [0x22; 32],
                DataClass::Internal,
                DocId(i),
                TxId(i),
                Some([i as u8; 32]),
            )
            .expect("append");
    }

    assert!(chain.verify_chain().expect("verify"));
    let head_sig = chain.sign_head(&sk).expect("sign head");
    assert!(AuditChain::verify_head_signature(&head_sig, &vk).expect("verify sig"));

    // Case 1: Field tampering in middle entry (entry 5 doc_id)
    let mut tampered_chain = chain.clone();
    tampered_chain.entries[5].doc_id = DocId(9999);
    assert!(
        !tampered_chain.verify_chain().expect("verify"),
        "Tampering with entry field MUST be detected"
    );

    // Case 2: Entry deletion (remove entry 5)
    let mut tampered_chain = chain.clone();
    tampered_chain.entries.remove(5);
    assert!(
        !tampered_chain.verify_chain().expect("verify"),
        "Deleting an entry MUST be detected"
    );

    // Case 3: Entry swap (swap 3 and 4)
    let mut tampered_chain = chain.clone();
    tampered_chain.entries.swap(3, 4);
    assert!(
        !tampered_chain.verify_chain().expect("verify"),
        "Swapping entries MUST be detected"
    );

    // Case 4: Head signature with wrong key
    let (_wrong_sk, wrong_vk) = test_ed25519_keypair();
    assert!(
        !AuditChain::verify_head_signature(&head_sig, &wrong_vk).expect("verify wrong vk"),
        "Head signature verification MUST fail with wrong verifying key"
    );

    // Case 5: Tampered head signature bytes
    let mut tampered_head_sig = head_sig.clone();
    tampered_head_sig.signature[0] ^= 0xFF;
    assert!(
        !AuditChain::verify_head_signature(&tampered_head_sig, &vk).expect("verify tampered sig"),
        "Tampered head signature bytes MUST fail verification"
    );
}

// ---------------------------------------------------------------------------
// H5: Record Commitment Hiding/Binding & EncryptedCommitmentSalt Roundtrips
// ---------------------------------------------------------------------------

#[test]
fn test_h5_record_commitment_binding_hiding_and_encrypted_salt() {
    // 1. Binding & Hiding across 100,000 random attribute/salt pairs
    let mut commitments = HashSet::with_capacity(100_000);
    let mut rng = OsRng;

    for i in 0..100_000 {
        let mut salt = [0u8; 32];
        let mut attr = [0u8; 32];
        rng.fill_bytes(&mut salt);
        rng.fill_bytes(&mut attr);

        let comm = compute_record_commitment(&salt, &attr);
        assert!(
            commitments.insert(comm),
            "Collision detected at iteration {i} — binding property violated!"
        );
    }

    // 2. Salt Independence: Same attribute with different salts yields distinct commitments
    let attr = b"Sensitive Personal SSN 123-45-6789";
    let salt1 = b"salt-1111-2222-3333-4444-5555-6666";
    let salt2 = b"salt-9999-8888-7777-6666-5555-4444";

    let comm1 = compute_record_commitment(salt1, attr);
    let comm2 = compute_record_commitment(salt2, attr);
    assert_ne!(
        comm1, comm2,
        "Distinct salts MUST produce distinct commitments for identical attribute"
    );

    // 3. Dictionary attack simulation without salt:
    // Attacker knows dictionary of candidate attributes: ["Alice", "Bob", "Charlie"]
    // But does NOT know secret salt (256-bit random).
    let target_attr = b"Alice";
    let secret_salt = b"super-secret-256-bit-random-salt!";
    let target_commitment = compute_record_commitment(secret_salt, target_attr);

    let candidates = [
        b"Alice".as_slice(),
        b"Bob".as_slice(),
        b"Charlie".as_slice(),
    ];
    let dictionary_salt_guesses = [b"guess_salt_1".as_slice(), b"guess_salt_2".as_slice()];

    for cand in candidates {
        for salt_guess in dictionary_salt_guesses {
            let guess_comm = compute_record_commitment(salt_guess, cand);
            assert_ne!(
                guess_comm, target_commitment,
                "Dictionary attack without correct salt MUST NOT match commitment"
            );
        }
    }

    // 4. EncryptedCommitmentSalt Roundtrip & Manipulation
    let km = KeyManager::try_new("test-passphrase-h5", b"salt-h5").expect("keymanager");
    let registry = KeyRegistry::new();
    let group_id = 555;

    let enc_salt = EncryptedCommitmentSalt::encrypt(&registry, &km, group_id, secret_salt)
        .expect("encrypt salt");
    let dec_salt = enc_salt.decrypt(&registry).expect("decrypt salt");
    assert_eq!(dec_salt, secret_salt);

    // Tampered EncryptedCommitmentSalt ciphertext
    let mut tampered_enc_salt = enc_salt.clone();
    tampered_enc_salt.ciphertext[0] ^= 0xFF;
    assert!(tampered_enc_salt.decrypt(&registry).is_err());

    // Revoked group destroys decryption
    registry.revoke_group(group_id).unwrap();
    assert!(
        enc_salt.decrypt(&registry).is_err(),
        "Decryption of salt MUST fail after key shredding"
    );
}

// ---------------------------------------------------------------------------
// H6: DeletionProof v3 Full Validation Suite
// ---------------------------------------------------------------------------

#[test]
fn test_h6_deletion_proof_v3_validation_suite() {
    let keypair = DeletionProofKeyPair::generate();
    let tenant_a = TenantId::try_new(10).unwrap();
    let tenant_b = TenantId::try_new(20).unwrap();
    let doc_x = DocId(100);
    let doc_y = DocId(200);

    // (a) Length-prefix collision test: ["ab", "c"] vs ["a", "bc"]
    let keys_1 = vec![b"ab".to_vec(), b"c".to_vec()];
    let keys_2 = vec![b"a".to_vec(), b"bc".to_vec()];
    let hash_1 = hash_deleted_keys_length_prefixed(&keys_1);
    let hash_2 = hash_deleted_keys_length_prefixed(&keys_2);
    assert_ne!(
        hash_1, hash_2,
        "Length-prefixed hashing MUST prevent collision between [ab, c] and [a, bc]"
    );

    // (b) Downgrade attack & SignatureVersion try_from for all 256 u8 values
    for version_byte in 0..=255u8 {
        let typed = SignatureVersion::try_from(version_byte);
        match version_byte {
            1 => assert_eq!(typed, Ok(SignatureVersion::V1)),
            2 => assert_eq!(typed, Ok(SignatureVersion::V2)),
            3 => assert_eq!(typed, Ok(SignatureVersion::V3)),
            v => assert_eq!(
                typed,
                Err(contextra_crypto::ed25519_proof::DeletionProofError::UnsupportedVersion(v))
            ),
        }
    }

    // Construct valid v3 proof
    let scope_x = DeletionScope::Document {
        doc_id: doc_x,
        tenant_id: tenant_a,
    };
    let graph_attest = vec![GraphRepairAttestation {
        doc_id: doc_x,
        verified_no_ghost_pointers: true,
        attested_at: 1700000000,
    }];
    let cleanup_proofs = vec![
        LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0).unwrap(),
        LayerCleanupProof::new_after_verified_empty(DeletionLayer::HnswIndex, 0).unwrap(),
    ];

    let proof_x = DeletionProof::create_v3(
        scope_x.clone(),
        keys_1.clone(),
        TxId(50),
        cleanup_proofs,
        vec![ExcludedScope::LlmParameterMemory],
        1700000000,
        &graph_attest,
        keypair.signing_key(),
    )
    .expect("create_v3");

    // verify_external accepts valid v3 proof
    assert!(proof_x.verify_external(&keypair.verifying_key).is_ok());

    // Downgrade check: verify_external MUST reject v1/v2 proofs with UnsupportedProofVersion
    let mut v2_downgrade = proof_x.clone();
    v2_downgrade.signature_version = 2;
    assert!(matches!(
        v2_downgrade.verify_external(&keypair.verifying_key),
        Err(CryptoError::UnsupportedProofVersion(2))
    ));

    // (c) Field-by-field tampering
    let mut tampered = proof_x.clone();
    tampered.timestamp += 1;
    assert!(proof_x.verify_external(&keypair.verifying_key).is_ok());
    assert!(tampered.verify_external(&keypair.verifying_key).is_err());

    let mut tampered = proof_x.clone();
    tampered.deleted_after_tx = TxId(51);
    assert!(tampered.verify_external(&keypair.verifying_key).is_err());

    let mut tampered = proof_x.clone();
    tampered.covered_layers.clear();
    assert!(tampered.verify_external(&keypair.verifying_key).is_err());

    let mut tampered = proof_x.clone();
    tampered.excluded_scopes.clear();
    assert!(tampered.verify_external(&keypair.verifying_key).is_err());

    let mut tampered = proof_x.clone();
    tampered.audit_chain_position = Some(42);
    assert!(tampered.verify_external(&keypair.verifying_key).is_err());

    // (d) Replay prevention: Proof for Tenant A / Doc X MUST NOT verify if scope is replaced with Tenant B / Doc Y
    let mut replayed_proof = proof_x.clone();
    replayed_proof.scope = DeletionScope::Document {
        doc_id: doc_y,
        tenant_id: tenant_b,
    };
    assert!(replayed_proof
        .verify_external(&keypair.verifying_key)
        .is_err());

    // (e) LayerCleanupProof non-empty rejection
    assert!(
        LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 5).is_err(),
        "LayerCleanupProof MUST reject remaining live entries > 0"
    );

    // (f) WAL delete receipt compute & verify roundtrip + tampering
    let integrity_key = b"wal-integrity-key-32-bytes-ok!";
    let prev_hmac = [0x77u8; 32];
    let delete_payload = b"wal_delete_event_payload";

    let receipt =
        compute_wal_delete_receipt(&prev_hmac, delete_payload, integrity_key).expect("receipt");
    assert!(
        verify_wal_delete_receipt(&receipt, &prev_hmac, delete_payload, integrity_key)
            .expect("verify receipt")
    );

    let tampered_payload = b"wal_delete_event_tampered";
    assert!(
        !verify_wal_delete_receipt(&receipt, &prev_hmac, tampered_payload, integrity_key)
            .expect("verify receipt")
    );

    // (g) export_for_audit secrecy: JSON contains no key or cleartext payload
    let json_export = proof_x.export_for_audit().expect("export");
    assert!(!json_export.contains("signing_key"));
    assert!(!json_export.contains("secret"));
    assert!(!json_export.contains("master_key"));
}

// ---------------------------------------------------------------------------
// H9: Post-Revocation Read Paths & Emergency Wipe
// ---------------------------------------------------------------------------

#[test]
fn test_h9_post_revocation_read_path_and_emergency_wipe() {
    let km = KeyManager::try_new("test-passphrase-h9", b"salt-h9").expect("keymanager");
    let registry = KeyRegistry::new();
    let group_id = 700;

    let (ct, nonce) = registry
        .encrypt_with_group(&km, group_id, b"Confidential Plaintext")
        .expect("encrypt");

    // Pre-revocation: Decryption succeeds
    let dec = registry
        .decrypt_with_group(group_id, &ct, &nonce)
        .expect("decrypt");
    assert_eq!(dec, b"Confidential Plaintext");

    // Action: Revoke group
    assert!(registry.revoke_group(group_id).unwrap());

    // Post-revocation: decrypt_with_group MUST fail fail-closed with KeyRevoked
    let dec_res = registry.decrypt_with_group(group_id, &ct, &nonce);
    assert!(
        matches!(dec_res, Err(CryptoError::KeyRevoked(_))),
        "Post-revocation decryption MUST fail with CryptoError::KeyRevoked"
    );

    // Emergency Wipe test: VolatileEncryptionKey emergency_wipe zeroizes key material
    let key_bytes = [0xABu8; 32];
    let mut volatile_key = VolatileEncryptionKey::new(key_bytes);
    assert_eq!(volatile_key.inspect_key_bytes_for_test(), &[0xABu8; 32]);

    volatile_key.emergency_wipe();
    assert_eq!(
        volatile_key.inspect_key_bytes_for_test(),
        &[0x00u8; 32],
        "VolatileEncryptionKey emergency_wipe MUST clear and zeroize key material to [0x00; 32]"
    );
}
