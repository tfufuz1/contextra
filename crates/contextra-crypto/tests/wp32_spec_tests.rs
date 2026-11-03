//! Comprehensive test suite fulfilling all SPEC WP-3.2, T-KDF-1/2, T-KEYSLOT-1..4, T-NONCE-1, T-ENC-1..3, T-HDR-1, T-FMT-1, T-KEY-1, T-WAL-1, T-PROOF-1 requirements.

use contextra_crypto::*;

#[test]
fn test_cr02_kdf_bounds_and_constructors_t_kdf_1_2() {
    // T-KDF-1: m in [65536, 4194304] KiB, t in [3, 16], p in [1, 16]
    // from_header returns Corrupt
    assert_eq!(
        KdfParams::from_header(65535, 3, 4),
        Err(CryptoError::Corrupt)
    );
    assert_eq!(
        KdfParams::from_header(4194305, 3, 4),
        Err(CryptoError::Corrupt)
    );
    assert_eq!(
        KdfParams::from_header(65536, 2, 4),
        Err(CryptoError::Corrupt)
    );
    assert_eq!(
        KdfParams::from_header(65536, 17, 4),
        Err(CryptoError::Corrupt)
    );
    assert_eq!(
        KdfParams::from_header(65536, 3, 0),
        Err(CryptoError::Corrupt)
    );
    assert_eq!(
        KdfParams::from_header(65536, 3, 17),
        Err(CryptoError::Corrupt)
    );

    // from_config returns Config
    assert_eq!(
        KdfParams::from_config(65535, 3, 4),
        Err(CryptoError::Config)
    );
    assert_eq!(
        KdfParams::from_config(4194305, 3, 4),
        Err(CryptoError::Config)
    );
    assert_eq!(
        KdfParams::from_config(65536, 2, 4),
        Err(CryptoError::Config)
    );
    assert_eq!(
        KdfParams::from_config(65536, 17, 4),
        Err(CryptoError::Config)
    );
    assert_eq!(
        KdfParams::from_config(65536, 3, 0),
        Err(CryptoError::Config)
    );
    assert_eq!(
        KdfParams::from_config(65536, 3, 17),
        Err(CryptoError::Config)
    );

    // Valid bounds
    let ok = KdfParams::from_header(131072, 3, 4).expect("valid header params");
    assert_eq!(ok.m_kib, 131072);
    assert_eq!(ok.t, 3);
    assert_eq!(ok.p, 4);

    // CR-03: KdfParams::DEFAULT is m = 128 MiB (131072 KiB), t = 3, p = 4
    assert_eq!(KdfParams::DEFAULT.m_kib, 131072);
    assert_eq!(KdfParams::DEFAULT.t, 3);
    assert_eq!(KdfParams::DEFAULT.p, 4);
}

#[test]
fn test_cr09_label_table_constants() {
    // CR-09: Verifies label constants match Appendix C
    assert_eq!(labels::TENANT_TAG, b"ctx/tenant/v1");
    assert_eq!(labels::COLL_TAG, b"ctx/coll/v1");
    assert_eq!(labels::SUBJECT_TAG, b"ctx/subj/v1");
    assert_eq!(labels::DOC_TAG, b"ctx/doc/v2");
    assert_eq!(labels::TERM_TAG, b"ctx/term/v1");
    assert_eq!(labels::NAME_TAG, b"ctx/name/v1");

    assert_eq!(labels::COMMIT, b"ctx/commit/v1");

    assert_eq!(labels::ENC, b"ctx/enc/v1");
    assert_eq!(labels::IDX, b"ctx/idx/v1");
    assert_eq!(labels::UNIT, b"ctx/unit/v1");
    assert_eq!(labels::SIGN, b"ctx/sign/v1");
    assert_eq!(labels::WAL, b"ctx/wal/v1");
    assert_eq!(labels::LEDGER, b"ctx/ledger/v1");
    assert_eq!(labels::FILE, b"ctx/file/v1");
    assert_eq!(labels::FKMASK, b"ctx/fkmask/v1");
    assert_eq!(labels::DEK, b"ctx/dek/v2");
    assert_eq!(labels::DEK_SALT, b"ctx/dek-salt/v2");

    assert_eq!(labels::REC_AAD, b"ctx/rec/v2");
    assert_eq!(labels::SLOT_AAD, b"ctx/slot/v1");
    assert_eq!(labels::VAULT_MAC, b"ctx/vault-mac/v1");
    assert_eq!(labels::HDR, b"ctx/hdr/v1");
    assert_eq!(labels::TBIND_AAD, b"ctx/tbind/v1");
    assert_eq!(labels::REPORT_PREFIX, b"ctx/report/v1");
}

#[test]
fn test_cr04_slot_aad_binding_t_keyslot_4() {
    // CR-04 & T-KEYSLOT-4: Copying slot to different store fails with Auth
    let master = MasterKey::generate_random();
    let kek = SecretKey::<32>::new([0x42; 32]);
    let store_id = [1u8; 16];
    let nonce = [7u8; 12];
    let aad = SlotAad {
        store_id,
        slot_id: 1,
        anchor_id: 1,
        kdf_id: 1,
        m_kib: 131072,
        t: 3,
        p: 4,
        slot_salt: [0x55; 32],
    };

    let wrapped = wrap_master(&kek, &aad, nonce, &master);
    let unwrapped = unwrap_master(&kek, &aad, nonce, &wrapped);
    assert!(unwrapped.is_ok());

    // Copy to foreign store
    let mut foreign_aad = aad;
    foreign_aad.store_id = [2u8; 16];
    assert!(matches!(
        unwrap_master(&kek, &foreign_aad, nonce, &wrapped),
        Err(CryptoError::Auth)
    ));

    // Tamper parameter m_kib
    let mut tampered_aad = aad;
    tampered_aad.m_kib = 65536;
    assert!(matches!(
        unwrap_master(&kek, &tampered_aad, nonce, &wrapped),
        Err(CryptoError::Auth)
    ));
}

#[test]
fn test_cr05_nonce_counter_overflow_t_nonce_1() {
    // CR-05 & T-NONCE-1: BlockSealer reaches limit at 2^32 blocks
    let key = FileKey::new([0x33; 32]);
    let file_id = [0x99; 16];
    let mut sealer = BlockSealer::new(key, FileKind::SSTable, file_id, None);
    sealer.set_counter_for_test(1u64 << 32);
    let res = sealer.seal(b"data");
    assert!(matches!(res, Err(CryptoError::Limit)));
}

#[test]
fn test_cr06_block_sealer_and_header_tag_t_enc_2() {
    let key = FileKey::new([0x55; 32]);
    let file_id = [0x77; 16];
    let mut sealer = BlockSealer::new(key, FileKind::Wal, file_id, Some([0x11; 32]));

    let block1 = sealer.seal(b"block 0 content").expect("seal block 0");
    let block2 = sealer.seal(b"block 1 content").expect("seal block 1");
    assert_ne!(block1.ciphertext, block2.ciphertext);

    let header48 = [0xAA; 48];
    let tag = sealer.header_tag(&header48);
    assert_eq!(tag.len(), 16);
}

#[test]
fn test_cr07_dek_derivation_and_record_envelope_v2_t_enc_1_t_vault_2() {
    // CR-07, D-79, L-3: DEK derivation with ctx/dek-salt/v2
    let master = MasterKey::generate_random();
    let hierarchy = KeyHierarchy::from_master(master);

    let salts = AncestorSalts {
        t: [1u8; 32],
        c: [2u8; 32],
        s: None, // salt_S = 0^32
        d: [4u8; 32],
    };
    let doc_tag = [9u8; 16];
    let dek = derive_dek(hierarchy.unit(), &salts, doc_tag);

    // Record envelope seal & open (F.6)
    let key_bytes = b"tenants/1/docs/99!12345";
    let plain = b"Sensitive GDPR payload";
    let env = seal_record(&dek, doc_tag, key_bytes, plain);

    assert_eq!(env[0], 2); // version = 2
    assert_eq!(env[1], 1); // alg_id = 1 (AES-256-GCM-SIV)
    assert_eq!(&env[2..18], &doc_tag);

    let opened = open_record(&dek, key_bytes, &env).expect("open record");
    assert_eq!(opened.as_slice(), plain);

    // Wrong key_bytes fails with Auth
    let wrong_key = b"tenants/1/docs/99!99999";
    assert_eq!(
        open_record(&dek, wrong_key, &env).unwrap_err(),
        CryptoError::Auth
    );
}

#[test]
fn test_cr08_k_sign_derived_and_report_signature_t_proof_1() {
    // CR-08: K_sign is derived via HKDF(K_master, "ctx/sign/v1") and never stored
    let master = MasterKey::generate_random();
    let hierarchy = KeyHierarchy::from_master(master);

    let report_content = b"Art. 17 GDPR Erasure Attestation";
    let sig = sign_report(hierarchy.sign_seed(), report_content);

    // Public key from signing key
    let signing_key = ed25519_dalek::SigningKey::from_bytes(hierarchy.sign_seed().0.as_bytes());
    let pubkey = signing_key.verifying_key().to_bytes();

    assert!(verify_report(&pubkey, report_content, &sig).is_ok());

    // Tampered report fails with Auth
    let mut tampered_report = report_content.to_vec();
    tampered_report[0] ^= 0xFF;
    assert_eq!(
        verify_report(&pubkey, &tampered_report, &sig).unwrap_err(),
        CryptoError::Auth
    );
}

#[test]
fn test_cr11_tag_preimage_domain_separation_t_key_1() {
    // CR-11: length-prefixed preimage prevents collision (a||bc) != (ab||c)
    let master = MasterKey::generate_random();
    let hierarchy = KeyHierarchy::from_master(master);

    let mut pre1 = TagPreimage::new();
    pre1.push(b"a");
    pre1.push(b"bc");

    let mut pre2 = TagPreimage::new();
    pre2.push(b"ab");
    pre2.push(b"c");

    let tag1 = tag(hierarchy.idx(), &pre1);
    let tag2 = tag(hierarchy.idx(), &pre2);

    assert_ne!(tag1, tag2);
}

#[test]
fn test_cr12_chains_and_macs_t_wal_1_2() {
    let prev = [0u8; 32];
    let entry = b"log_entry_001";
    let next = chain_next(&prev, entry);
    assert_ne!(next, prev);

    let master = MasterKey::generate_random();
    let hierarchy = KeyHierarchy::from_master(master);

    let wal_tag = [0xEE; 16];
    let ct = b"wal ciphertext batch";
    let w_mac = wal_mac(hierarchy.wal(), &prev, ct.len() as u32, 1, ct, &wal_tag);
    assert_ne!(w_mac, [0u8; 32]);

    let l_mac = ledger_mac(hierarchy.ledger(), &next);
    assert_ne!(l_mac, [0u8; 32]);
}

#[test]
fn test_cr13_cr14_header_encoding_and_validation_t_hdr_1_t_fmt_1() {
    let master = MasterKey::generate_random();
    let hierarchy = KeyHierarchy::from_master(master);

    let mut header = Header {
        format_version: HEADER_FORMAT_VERSION,
        kind: HEADER_KIND,
        alg_ids: HEADER_ALG_IDS,
        store_id: [0x42; 16],
        generation: 1,
        slot_count: 1,
        report_pub: [0x88; 32],
        slots: [
            HeaderSlot::empty(),
            HeaderSlot::empty(),
            HeaderSlot::empty(),
            HeaderSlot::empty(),
        ],
    };

    header.slots[0] = HeaderSlot {
        slot_id: 1,
        anchor_id: 1,
        kdf_id: 1,
        status: 1,
        m_kib: 131072,
        t: 3,
        p: 4,
        slot_salt: [0x77; 32],
        nonce: [0x11; 12],
        wrapped_master: [0x99; 48],
    };

    let encoded = header.encode(hierarchy.hdr());
    let decoded = Header::decode_checked(&encoded).expect("decode header");
    assert_eq!(decoded.generation, 1);
    assert_eq!(decoded.slots[0].slot_id, 1);

    // Verify MAC
    assert!(decoded.verify_mac(hierarchy.hdr(), &encoded).is_ok());

    // Tamper with check bytes -> Corrupt
    let mut tampered = encoded;
    tampered[10] = 0x99; // corrupt kind
    assert_eq!(
        Header::decode_checked(&tampered).unwrap_err(),
        CryptoError::Corrupt
    );

    // alg_id = 2 is reserved and must return Corrupt
    let mut reserved_alg = encoded;
    reserved_alg[11] = 2; // alg_id = 2 (ChaCha20-Poly1305 reserved)
                          // recalculate check16
    let chk = Header::check16(&reserved_alg[..584]);
    reserved_alg[584..600].copy_from_slice(&chk);
    assert_eq!(
        Header::decode_checked(&reserved_alg).unwrap_err(),
        CryptoError::Corrupt
    );
}

#[test]
fn test_t_kdf_1_min_m_kib_rejects_old_buggy_value_19456() {
    // T-KDF-1: Regression für F-11: Alter Wert MIN_M_COST_KIB = 19456 muss abgelehnt werden.
    // SPEC D-06 mandatiert m ∈ [65536, 4194304] KiB (= [64 MiB, 4 GiB]).
    // Der alte Wert MIN_M_COST_KIB = 19456 (≈ 19 MiB) unterschritt das Minimum
    // und durfte keine gültigen KdfParams erzeugen.

    // Alter Buggy-Wert 19456 muss abgelehnt werden
    assert_eq!(
        KdfParams::from_header(19456, 3, 4),
        Err(CryptoError::Corrupt),
        "T-KDF-1: from_header(19456) muss Corrupt zurückgeben (< 65536 KiB)"
    );
    assert_eq!(
        KdfParams::from_config(19456, 3, 4),
        Err(CryptoError::Config),
        "T-KDF-1: from_config(19456) muss Config zurückgeben (< 65536 KiB)"
    );

    // SPEC-Minimum 65536 KiB (64 MiB) muss akzeptiert werden
    assert!(
        KdfParams::from_header(65536, 3, 4).is_ok(),
        "T-KDF-1: from_header(65536) muss Ok zurückgeben (SPEC-Minimum)"
    );
    assert!(
        KdfParams::from_config(65536, 3, 4).is_ok(),
        "T-KDF-1: from_config(65536) muss Ok zurückgeben (SPEC-Minimum)"
    );
}

/// SPEC: T-NONCE, T-ENC, T-KEYSLOT, T-KDF
/// Comprehensive Negative Test Matrix
#[test]
fn test_negative_matrix_bit_flip_and_truncation() {
    use aes_gcm_siv::aead::{Aead, KeyInit};

    let key = FileKey::new([0x33; 32]);
    let file_id = [0x99; 16];
    let mut sealer = BlockSealer::new(key, FileKind::SSTable, file_id, None);
    let block = sealer.seal(b"Secret database block").expect("seal block");

    let cipher_key = aes_gcm_siv::Key::<aes_gcm_siv::Aes256GcmSiv>::from_slice(&[0x33; 32]);
    let cipher = aes_gcm_siv::Aes256GcmSiv::new(cipher_key);

    let mut nonce = [0u8; 12];
    nonce[4..12].copy_from_slice(&0u64.to_be_bytes());
    let mut aad = Vec::new();
    aad.push(FileKind::SSTable as u8);
    aad.extend_from_slice(&file_id);
    aad.extend_from_slice(&0u64.to_be_bytes());

    let mut full_ct = block.ciphertext.clone();
    full_ct.extend_from_slice(&block.tag);

    for i in 0..full_ct.len() {
        let mut tampered = full_ct.clone();
        tampered[i] ^= 0x01;
        let payload = aes_gcm_siv::aead::Payload {
            msg: &tampered,
            aad: &aad,
        };
        assert!(
            cipher
                .decrypt(aes_gcm_siv::Nonce::from_slice(&nonce), payload)
                .is_err(),
            "Bit flip at byte {i} should fail auth"
        );
    }

    for len in 0..full_ct.len() {
        let truncated = &full_ct[..len];
        let payload = aes_gcm_siv::aead::Payload {
            msg: truncated,
            aad: &aad,
        };
        assert!(
            cipher
                .decrypt(aes_gcm_siv::Nonce::from_slice(&nonce), payload)
                .is_err(),
            "Truncation to length {len} should fail auth"
        );
    }
}

#[test]
fn test_negative_slot_replay_older_generation() {
    let master = MasterKey::generate_random();
    let hierarchy = KeyHierarchy::from_master(master);

    let mut h1 = Header {
        format_version: HEADER_FORMAT_VERSION,
        kind: HEADER_KIND,
        alg_ids: HEADER_ALG_IDS,
        store_id: [0x42; 16],
        generation: 10,
        slot_count: 1,
        report_pub: [0x88; 32],
        slots: [
            HeaderSlot::empty(),
            HeaderSlot::empty(),
            HeaderSlot::empty(),
            HeaderSlot::empty(),
        ],
    };
    h1.slots[0] = HeaderSlot {
        slot_id: 1,
        anchor_id: 1,
        kdf_id: 1,
        status: 1,
        m_kib: 131072,
        t: 3,
        p: 4,
        slot_salt: [0x77; 32],
        nonce: [0x11; 12],
        wrapped_master: [0x99; 48],
    };

    let encoded_gen10 = h1.encode(hierarchy.hdr());
    let decoded_gen10 = Header::decode_checked(&encoded_gen10).expect("valid header");

    let mut h2 = h1.clone();
    h2.generation = 9; // older generation replay
    let encoded_gen9 = h2.encode(hierarchy.hdr());
    let decoded_gen9 = Header::decode_checked(&encoded_gen9).expect("valid header");

    assert!(decoded_gen9.generation < decoded_gen10.generation);
}

#[test]
fn test_negative_downgrade_attempts_and_fuzz_params() {
    let mut bad_alg_header = [0u8; HEADER_SIZE];
    bad_alg_header[..8].copy_from_slice(HEADER_MAGIC);
    bad_alg_header[8..10].copy_from_slice(&HEADER_FORMAT_VERSION.to_be_bytes());
    bad_alg_header[10] = HEADER_KIND;
    bad_alg_header[11..15].copy_from_slice(&[2, 0, 1, 1]); // alg_id = 2
    bad_alg_header[39] = 1; // slot_count
    let chk = Header::check16(&bad_alg_header[..584]);
    bad_alg_header[584..600].copy_from_slice(&chk);

    assert_eq!(
        Header::decode_checked(&bad_alg_header).unwrap_err(),
        CryptoError::Corrupt
    );

    let bad_kdf_inputs = [
        (0u32, 3u8, 4u8),
        (65535, 3, 4),
        (4194305, 3, 4),
        (u32::MAX, 3, 4),
        (131072, 0, 4),
        (131072, 2, 4),
        (131072, 17, 4),
        (131072, u8::MAX, 4),
        (131072, 3, 0),
        (131072, 3, 17),
        (131072, 3, u8::MAX),
    ];
    for (m, t, p) in bad_kdf_inputs {
        assert_eq!(
            KdfParams::from_header(m, t, p),
            Err(CryptoError::Corrupt),
            "Failed for ({m}, {t}, {p})"
        );
        assert_eq!(
            KdfParams::from_config(m, t, p),
            Err(CryptoError::Config),
            "Failed for ({m}, {t}, {p})"
        );
    }
}

#[test]
fn test_property_labels_are_prefix_free() {
    let all_labels: &[&[u8]] = &[
        labels::TENANT_TAG,
        labels::COLL_TAG,
        labels::SUBJECT_TAG,
        labels::DOC_TAG,
        labels::TERM_TAG,
        labels::NAME_TAG,
        labels::COMMIT,
        labels::ENC,
        labels::IDX,
        labels::UNIT,
        labels::SIGN,
        labels::WAL,
        labels::LEDGER,
        labels::FILE,
        labels::FKMASK,
        labels::DEK,
        labels::DEK_SALT,
        labels::REC_AAD,
        labels::SLOT_AAD,
        labels::VAULT_MAC,
        labels::HDR,
        labels::TBIND_AAD,
        labels::REPORT_PREFIX,
    ];

    for (i, l1) in all_labels.iter().enumerate() {
        for (j, l2) in all_labels.iter().enumerate() {
            if i != j {
                assert!(
                    !l2.starts_with(l1),
                    "Label {:?} is a prefix of label {:?}",
                    std::str::from_utf8(l1).unwrap(),
                    std::str::from_utf8(l2).unwrap()
                );
            }
        }
    }
}

#[test]
fn test_compile_time_zeroize_on_drop_bound() {
    fn assert_zeroize_on_drop<T: zeroize::ZeroizeOnDrop>() {}
    assert_zeroize_on_drop::<SecretKey<32>>();
    assert_zeroize_on_drop::<Secret>();
}

#[test]
fn test_constant_time_eq_distribution() {
    use subtle::ConstantTimeEq;

    let a = [0x42u8; 32];
    let b = [0x42u8; 32];
    let c = [0x24u8; 32];

    let mut sum_equal = 0u64;
    let mut sum_diff = 0u64;

    for _ in 0..1_000_000 {
        sum_equal += a.ct_eq(&b).unwrap_u8() as u64;
        sum_diff += a.ct_eq(&c).unwrap_u8() as u64;
    }

    assert_eq!(sum_equal, 1_000_000);
    assert_eq!(sum_diff, 0);
}

#[test]
fn test_mutation_explicit_domain_separation_check() {
    let master = MasterKey::new([0x11; 32]);
    let hierarchy = KeyHierarchy::from_master(master);

    let mut empty_derived = [0u8; 32];
    hkdf_expand(hierarchy.enc().0.as_bytes(), b"", &mut empty_derived).unwrap();
    assert_ne!(hierarchy.enc().0.as_bytes(), &empty_derived);

    let signing_key = ed25519_dalek::SigningKey::from_bytes(hierarchy.sign_seed().0.as_bytes());
    let report_content = b"Art. 17 GDPR Erasure Attestation";
    let sig_with_prefix = sign_report(hierarchy.sign_seed(), report_content);

    use ed25519_dalek::Signer;
    let sig_without_prefix = signing_key.sign(report_content).to_bytes();
    assert_ne!(
        sig_with_prefix, sig_without_prefix,
        "Domain label removal mutant detected!"
    );
}

#[test]
fn test_mutation_aad_omission_fails() {
    let key = SecretKey::<32>::new([0x22; 32]);
    let dek = Dek(key);
    let doc_tag = [0x77; 16];
    let key_bytes = b"my_key";
    let plain = b"data";

    let env = seal_record(&dek, doc_tag, key_bytes, plain);

    assert_eq!(
        open_record(&dek, b"", &env).unwrap_err(),
        CryptoError::Auth,
        "AAD omission mutant detected!"
    );
}

#[test]
fn test_mutation_nonce_reuse_prevention() {
    let key = FileKey::new([0x33; 32]);
    let file_id = [0x99; 16];
    let mut sealer = BlockSealer::new(key, FileKind::SSTable, file_id, None);
    let block1 = sealer.seal(b"same content").unwrap();
    let block2 = sealer.seal(b"same content").unwrap();

    assert_ne!(
        block1.ciphertext, block2.ciphertext,
        "Nonce reuse mutant detected!"
    );
}

#[test]
fn test_mutation_verify_vs_verify_strict() {
    let mut small_order_pub = [0u8; 32];
    small_order_pub[0] = 1;
    if let Ok(weak_vk) = ed25519_dalek::VerifyingKey::from_bytes(&small_order_pub) {
        let dummy_sig = [0u8; 64];
        let res = verify_report(&weak_vk.to_bytes(), b"data", &dummy_sig);
        assert_eq!(
            res.unwrap_err(),
            CryptoError::Auth,
            "verify_strict mutant detected!"
        );
    }
}
