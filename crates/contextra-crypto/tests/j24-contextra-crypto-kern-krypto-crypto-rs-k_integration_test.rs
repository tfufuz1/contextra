// FILE-CONTEXT
// ZWECK: Integrationstest für J24 Krypto-Kern Verdrahtung (crypto.rs / kdf.rs / wal_crypto.rs).
// COVERS: KeyManager::try_new_argon2id (KdfHeader::generate_default), KeyManager::create_deletion_proof (derive_deletion_proof_key),
//         KeyManager::derive_segment_key_for_id (derive_segment_key), KeyManager::cipher_for_scoped,
//         KeyManager::derive_kv_key_scoped, IntegrityVerifier::handoff_from (last_seq_no_snapshot & set_last_seq_no).

#![forbid(unsafe_code)]

use contextra_crypto::deletion_proof::{
    DeletionLayer, DeletionScope, ExcludedScope, LayerCleanupProof,
};
use contextra_crypto::kdf::KDF_HEADER_VERSION_1;
use contextra_crypto::wal_crypto::{IntegrityVerifier, WalEntrySnapshot, WalHmac};
use contextra_crypto::{KeyManager, ModelFingerprint};
use contextra_types::{DocId, TenantId, TenantScoped, TxId};

#[test]
fn test_wire_generate_default_via_try_new_argon2id() {
    let (km, header) = KeyManager::try_new_argon2id("owasp-argon2id-passphrase")
        .expect("KeyManager::try_new_argon2id should succeed");

    assert_eq!(header.version, KDF_HEADER_VERSION_1);
    assert_eq!(header.salt.len(), 32);

    let plaintext = b"argon2id-protected-data";
    let (ciphertext, nonce) = km.encrypt_auto_nonce(plaintext).expect("encrypt");
    let decrypted = km.decrypt_auto_nonce(&ciphertext, &nonce).expect("decrypt");

    assert_eq!(decrypted, plaintext);
}

#[test]
fn test_wire_derive_deletion_proof_key_via_create_deletion_proof() {
    let km = KeyManager::try_new("master-passphrase-del", b"salt-12345678")
        .expect("KeyManager::try_new");

    let tenant = TenantId::try_new(101).expect("TenantId");
    let scope = DeletionScope::Document {
        doc_id: DocId(42),
        tenant_id: tenant,
    };

    let cleanup_proofs =
        vec![
            LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0)
                .expect("LayerCleanupProof"),
        ];

    let proof = km
        .create_deletion_proof(
            scope,
            vec![b"doc_42_key".to_vec()],
            TxId(100),
            cleanup_proofs,
            vec![ExcludedScope::LlmParameterMemory],
        )
        .expect("create_deletion_proof");

    let verification_key = km
        .derive_deletion_proof_key()
        .expect("derive_deletion_proof_key");

    assert!(
        proof.verify(&verification_key).expect("verify"),
        "DeletionProof MUST verify against key derived via derive_deletion_proof_key"
    );
}

#[test]
fn test_wire_derive_segment_key_via_derive_segment_key_for_id() {
    let km = KeyManager::try_new("master-passphrase-seg", b"salt-12345678")
        .expect("KeyManager::try_new");

    let tenant = TenantId::try_new(10).expect("TenantId");
    let sub_km1 = km
        .derive_segment_key_for_id(tenant, 100)
        .expect("derive_segment_key_for_id 100");
    let sub_km2 = km
        .derive_segment_key_for_id(tenant, 100)
        .expect("derive_segment_key_for_id 100");
    let sub_km3 = km
        .derive_segment_key_for_id(tenant, 101)
        .expect("derive_segment_key_for_id 101");

    let data = b"segment-payload-tensor-data";
    let (encrypted, nonce) = sub_km1.encrypt_auto_nonce(data).expect("encrypt");

    let dec2 = sub_km2
        .decrypt_auto_nonce(&encrypted, &nonce)
        .expect("dec2");
    assert_eq!(dec2, data);

    assert!(
        sub_km3.decrypt_auto_nonce(&encrypted, &nonce).is_err(),
        "Sub-key for segment 101 MUST NOT decrypt segment 100 ciphertext"
    );
}

#[test]
fn test_wire_cipher_for_scoped_happy_and_mismatch() {
    let km = KeyManager::try_new("master-passphrase-scoped", b"salt-12345678")
        .expect("KeyManager::try_new");

    let tenant_a = TenantId::try_new(100).expect("TenantId A");
    let tenant_b = TenantId::try_new(200).expect("TenantId B");

    let scoped_payload = TenantScoped::new(tenant_a, "dummy_token");

    // Happy path: expected tenant matches bound tenant
    let cipher_scoped = km
        .cipher_for_scoped(scoped_payload.clone(), &tenant_a)
        .expect("cipher_for_scoped happy path");
    let cipher_direct = km.cipher_for(tenant_a).expect("cipher_for");

    let data = b"tenant-a-confidential-bytes";
    let (ct_scoped, nonce) = cipher_scoped.encrypt_auto_nonce(data).expect("encrypt");
    let dec_direct = cipher_direct
        .decrypt_auto_nonce(&ct_scoped, &nonce)
        .expect("decrypt");
    assert_eq!(dec_direct, data);

    // Mismatch path: expected tenant mismatch rejected
    let err = km.cipher_for_scoped(scoped_payload, &tenant_b);
    assert!(
        err.is_err(),
        "cipher_for_scoped MUST reject mismatching tenant scope"
    );
}

#[test]
fn test_wire_derive_kv_key_scoped_happy_and_mismatch() {
    let km = KeyManager::try_new("master-passphrase-kv-scoped", b"salt-12345678")
        .expect("KeyManager::try_new");

    let tenant_a = TenantId::try_new(100).expect("TenantId A");
    let tenant_b = TenantId::try_new(200).expect("TenantId B");

    let fp = ModelFingerprint::new([0x33u8; 32], "llama-3.2-1b", "Q4_0");
    let scoped_fp = TenantScoped::new(tenant_a, &fp);

    // Happy path: expected tenant matches bound tenant
    let key_scoped = km
        .derive_kv_key_scoped(scoped_fp.clone(), &tenant_a)
        .expect("derive_kv_key_scoped happy path");
    let key_direct = km
        .derive_kv_key(tenant_a, &fp)
        .expect("derive_kv_key direct");

    let data = b"kv-cache-layer-data";
    let (ct, nonce) = key_scoped.encrypt_auto_nonce(data).expect("encrypt");
    let dec = key_direct.decrypt_auto_nonce(&ct, &nonce).expect("decrypt");
    assert_eq!(dec, data);

    // Mismatch path: expected tenant mismatch rejected
    let err = km.derive_kv_key_scoped(scoped_fp, &tenant_b);
    assert!(
        err.is_err(),
        "derive_kv_key_scoped MUST reject mismatching tenant scope"
    );
}

#[test]
fn test_wire_last_seq_no_snapshot_and_set_last_seq_no_via_handoff_from() {
    let key = b"wal-integrity-key-32-bytes-long!";
    let mut verifier_a = IntegrityVerifier::new(key);

    // Helper to create valid WAL v3 entry
    let create_v3_entry =
        |prev_hmac: [u8; 32], seq_no: u64, k: &[u8], v: &[u8]| -> WalEntrySnapshot {
            let tx_id = seq_no;
            let mut mac = WalHmac::new(key).expect("mac init");
            mac.update(&prev_hmac);
            mac.update(&seq_no.to_le_bytes());
            mac.update(&tx_id.to_le_bytes());
            mac.update(&[0u8]); // Put op
            mac.update(&(k.len() as u32).to_le_bytes());
            mac.update(k);
            mac.update(&(v.len() as u32).to_le_bytes());
            mac.update(v);
            let checksum = mac.finalize();
            WalEntrySnapshot {
                tx_id,
                seq_no,
                op_type: 0,
                key: k.to_vec(),
                value: v.to_vec(),
                checksum,
                prev_hmac,
            }
        };

    // Entry 1 processed on verifier A
    let e1 = create_v3_entry([0u8; 32], 1, b"key1", b"val1");
    verifier_a.verify_and_update(&e1, 10).expect("e1 valid");

    assert_eq!(verifier_a.last_seq_no_snapshot(), Some(1));

    // Full handoff to verifier B (transfers both last_hmac and last_seq_no)
    let mut verifier_b = IntegrityVerifier::new(key);
    verifier_b.handoff_from(&verifier_a);

    assert_eq!(verifier_b.last_hmac_snapshot(), e1.checksum);
    assert_eq!(verifier_b.last_seq_no_snapshot(), Some(1));

    // Entry 2 (seq_no = 2) processed on verifier B
    let e2 = create_v3_entry(e1.checksum, 2, b"key2", b"val2");
    verifier_b
        .verify_and_update(&e2, 20)
        .expect("e2 valid on verifier B after handoff");

    assert_eq!(verifier_b.last_seq_no_snapshot(), Some(2));

    // Enforce sequence non-monotonicity check after handoff
    let duplicate_e2 = create_v3_entry(e1.checksum, 1, b"key2", b"val2");
    assert!(
        verifier_b.verify_and_update(&duplicate_e2, 25).is_err(),
        "Non-monotonic seq 1 MUST be rejected after handoff"
    );

    let gap_e4 = create_v3_entry(e2.checksum, 4, b"key4", b"val4");
    assert!(
        verifier_b.verify_and_update(&gap_e4, 30).is_err(),
        "Sequence gap (expected 3, got 4) MUST be rejected after handoff"
    );
}
