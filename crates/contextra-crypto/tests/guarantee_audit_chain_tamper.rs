// FILE-CONTEXT
// ZWECK: Independent guarantee test suite for AuditChain anti-tampering, head signature verification, and encrypted commitments.
// INVARIANTEN: Any alteration, removal, insertion, reordering, or truncation of a signed audit chain is detected by verify_chain or verify_head_signature against published head.
// BEFUND-TABELLE:
// | Manipulation | Erkannt durch verify_chain()? | Erkannt durch Head-Signatur / Anchor? | Begründung |
// |--------------|-------------------------------|---------------------------------------|------------|
// | Field Bit-Flip (index, schema_id, rules_hash, data_class, doc_id, tx_id, prev_hash, commitment, entry_hash) | Ja | Ja | Hash mismatch oder prev_hash Link unterbrochen. |
// | Entry Deletion (Position 0..n) | Ja | Ja | prev_hash Link / index Reihenfolge unterbrochen, oder head_hash mismatch. |
// | Entry Duplication (Position 0..n) | Ja | Ja | Index-Dopplung und prev_hash Link Fehler. |
// | Pair Swapping (Position i, j) | Ja | Ja | Transitive prev_hash Kette wird an beiden Stellen unterbrochen. |
// | Tail Truncation (Ohne Head-Hash Update) | Ja | Ja | last.entry_hash != head_hash in verify_chain(). |
// | Tail Truncation (Mit Head-Hash Update auf Sub-Kette) | Nein | Ja | Sub-Kette 0..k ist in sich valide (verify_chain = true), aber head_hash / index weichen von publizierter Head-Signatur ab. External Head Anchor zwingend erforderlich! |
// | Tail Appending (Gültiger Eigenhash) | Nein | Ja | Erweitere Kette 0..n+k ist lokal valide, aber publizierte Head-Signatur sichert nur den früheren Head-Hash/Index. |

#![forbid(unsafe_code)]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use contextra_crypto::{
    compute_record_commitment, AuditChain, AuditChainEntry, DataClass, DeletionProofKeyPair,
    EncryptedCommitmentSalt, KeyManager, KeyRegistry,
};
use contextra_types::{DocId, TxId};
use proptest::prelude::*;
use std::env;

/// Deterministic test seed helper.
fn get_test_seed() -> u64 {
    let seed = if let Ok(s) = env::var("AUDIT_CHAIN_TEST_SEED") {
        s.parse::<u64>().unwrap_or(0xCAFE_BABE_9876_5432)
    } else {
        0xCAFE_BABE_9876_5432
    };
    println!("AUDIT_CHAIN_TEST_SEED={seed}");
    seed
}

/// Independent BLAKE3 reference hash calculation.
///
/// **Serialization Format Specification (AuditChainEntry)**:
#[allow(clippy::too_many_arguments)]
/// 1. `index`: u64 (8 bytes, little-endian)
/// 2. `schema_id_len`: u32 (4 bytes, little-endian)
/// 3. `schema_id`: raw UTF-8 bytes
/// 4. `rules_hash`: [u8; 32]
/// 5. `data_class_len`: u32 (4 bytes, little-endian, always 4)
/// 6. `data_class_variant`: u32 (4 bytes, little-endian bincode representation: Public=0, Internal=1, Confidential=2, Restricted=3, Personal=4, SpecialCategoryPersonal=5)
/// 7. `doc_id`: u64 (8 bytes, little-endian)
/// 8. `tx_id`: u64 (8 bytes, little-endian)
/// 9. `prev_hash`: [u8; 32]
/// 10. `commitment_flag`: 1 byte (1 if Some, 0 if None)
/// 11. `commitment_payload`: [u8; 32] (only if flag == 1)
fn reference_compute_entry_hash(
    index: u64,
    schema_id: &str,
    rules_hash: &[u8; 32],
    data_class: DataClass,
    doc_id: DocId,
    tx_id: TxId,
    prev_hash: &[u8; 32],
    commitment: Option<&[u8; 32]>,
) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();

    // 1. Index (u64 LE)
    hasher.update(&index.to_le_bytes());

    // 2. Schema ID length (u32 LE) + raw string bytes
    hasher.update(&(schema_id.len() as u32).to_le_bytes());
    hasher.update(schema_id.as_bytes());

    // 3. Rules Hash ([u8; 32])
    hasher.update(rules_hash);

    // 4. DataClass bincode serialization format (u32 LE length prefix + u32 LE enum variant discriminant)
    let variant_u32: u32 = match data_class {
        DataClass::Public => 0,
        DataClass::Internal => 1,
        DataClass::Confidential => 2,
        DataClass::Restricted => 3,
        DataClass::Personal => 4,
        DataClass::SpecialCategoryPersonal => 5,
    };
    let class_bytes = variant_u32.to_le_bytes();
    hasher.update(&(class_bytes.len() as u32).to_le_bytes());
    hasher.update(&class_bytes);

    // 5. DocId (u64 LE) & TxId (u64 LE)
    hasher.update(&doc_id.0.to_le_bytes());
    hasher.update(&tx_id.0.to_le_bytes());

    // 6. Prev Hash ([u8; 32])
    hasher.update(prev_hash);

    // 7. Commitment flag + optional payload
    if let Some(comm) = commitment {
        hasher.update(&[1u8]);
        hasher.update(comm);
    } else {
        hasher.update(&[0u8]);
    }

    *hasher.finalize().as_bytes()
}

/// Helper: Builds a 50-entry chain with varied attributes and commitments.
fn build_50_entry_chain(seed: u64) -> AuditChain {
    let mut chain = AuditChain::new();
    let data_classes = [
        DataClass::Public,
        DataClass::Internal,
        DataClass::Confidential,
        DataClass::Restricted,
        DataClass::Personal,
        DataClass::SpecialCategoryPersonal,
    ];

    for i in 0..50 {
        let schema = format!("schema_v{}", (i % 5) + 1);
        let rules_hash = blake3::hash(format!("rules_{}_{seed}", i / 5).as_bytes()).into();
        let data_class = data_classes[(i as usize) % data_classes.len()];
        let doc_id = DocId(1000 + i);
        let tx_id = TxId(5000 + i);
        let commitment = if i % 2 == 0 {
            Some(blake3::hash(format!("comm_salt_{i}_{seed}").as_bytes()).into())
        } else {
            None
        };

        chain
            .append(schema, rules_hash, data_class, doc_id, tx_id, commitment)
            .unwrap();
    }

    chain
}

#[test]
fn test_independent_reference_hash_verification_50_entries() {
    let seed = get_test_seed();
    let chain = build_50_entry_chain(seed);
    assert_eq!(chain.len(), 50);

    for entry in chain.entries() {
        let reference_hash = reference_compute_entry_hash(
            entry.index,
            &entry.schema_id,
            &entry.rules_hash,
            entry.data_class,
            entry.doc_id,
            entry.tx_id,
            &entry.prev_hash,
            entry.commitment.as_ref(),
        );

        let code_hash = AuditChainEntry::compute_hash(
            entry.index,
            &entry.schema_id,
            &entry.rules_hash,
            entry.data_class,
            entry.doc_id,
            entry.tx_id,
            &entry.prev_hash,
            entry.commitment.as_ref(),
        )
        .unwrap();

        assert_eq!(reference_hash, code_hash);
        assert_eq!(reference_hash, entry.entry_hash);
    }

    assert!(chain.verify_chain().unwrap());
}

#[test]
fn test_exhaustive_field_level_bit_flips_50_entries() {
    let seed = get_test_seed();
    let base_chain = build_50_entry_chain(seed);

    for i in 0..base_chain.len() {
        // (a.1) Mutate index
        let mut tampered = base_chain.clone();
        tampered.entries[i].index ^= 1;
        assert!(
            !tampered.verify_chain().unwrap(),
            "Index tampering at entry {i} must be detected"
        );

        // (a.2) Mutate schema_id
        let mut tampered = base_chain.clone();
        tampered.entries[i].schema_id.push_str("_tampered");
        assert!(
            !tampered.verify_chain().unwrap(),
            "Schema ID tampering at entry {i} must be detected"
        );

        // (a.3) Mutate rules_hash byte
        let mut tampered = base_chain.clone();
        tampered.entries[i].rules_hash[0] ^= 0xFF;
        assert!(
            !tampered.verify_chain().unwrap(),
            "Rules hash tampering at entry {i} must be detected"
        );

        // (a.4) Mutate data_class
        let mut tampered = base_chain.clone();
        tampered.entries[i].data_class = match tampered.entries[i].data_class {
            DataClass::Public => DataClass::SpecialCategoryPersonal,
            _ => DataClass::Public,
        };
        assert!(
            !tampered.verify_chain().unwrap(),
            "DataClass tampering at entry {i} must be detected"
        );

        // (a.5) Mutate doc_id
        let mut tampered = base_chain.clone();
        tampered.entries[i].doc_id = DocId(tampered.entries[i].doc_id.0 ^ 0xFFFF);
        assert!(
            !tampered.verify_chain().unwrap(),
            "DocId tampering at entry {i} must be detected"
        );

        // (a.6) Mutate tx_id
        let mut tampered = base_chain.clone();
        tampered.entries[i].tx_id = TxId(tampered.entries[i].tx_id.0 ^ 0xFFFF);
        assert!(
            !tampered.verify_chain().unwrap(),
            "TxId tampering at entry {i} must be detected"
        );

        // (a.7) Mutate prev_hash
        let mut tampered = base_chain.clone();
        tampered.entries[i].prev_hash[0] ^= 0xAA;
        assert!(
            !tampered.verify_chain().unwrap(),
            "PrevHash tampering at entry {i} must be detected"
        );

        // (a.8) Mutate commitment
        let mut tampered = base_chain.clone();
        tampered.entries[i].commitment = match tampered.entries[i].commitment {
            Some(comm) => {
                let mut mutated = comm;
                mutated[0] ^= 0x55;
                Some(mutated)
            }
            None => Some([0x77u8; 32]),
        };
        assert!(
            !tampered.verify_chain().unwrap(),
            "Commitment tampering at entry {i} must be detected"
        );

        // (a.9) Mutate entry_hash directly
        let mut tampered = base_chain.clone();
        tampered.entries[i].entry_hash[0] ^= 0x01;
        assert!(
            !tampered.verify_chain().unwrap(),
            "EntryHash tampering at entry {i} must be detected"
        );
    }
}

#[test]
fn test_exhaustive_entry_deletion_duplication_and_swapping() {
    let seed = get_test_seed();
    let base_chain = build_50_entry_chain(seed);

    // (b) Delete entry at every position
    for pos in 0..base_chain.len() {
        let mut tampered = base_chain.clone();
        tampered.entries.remove(pos);

        // Without adjusting index/prev_hash, verify_chain fails
        assert!(
            !tampered.verify_chain().unwrap(),
            "Deletion at pos {pos} must fail verify_chain"
        );
    }

    // (c) Duplicate entry at every position
    for pos in 0..base_chain.len() {
        let mut tampered = base_chain.clone();
        let dupe = tampered.entries[pos].clone();
        tampered.entries.insert(pos, dupe);
        assert!(
            !tampered.verify_chain().unwrap(),
            "Duplication at pos {pos} must fail verify_chain"
        );
    }

    // (d) Swap two entries (testing n=12 pairs across first 12 entries)
    let n = 12;
    for i in 0..n {
        for j in (i + 1)..n {
            let mut tampered = base_chain.clone();
            tampered.entries.swap(i, j);
            assert!(
                !tampered.verify_chain().unwrap(),
                "Swapping entries {i} and {j} must fail verify_chain"
            );
        }
    }
}

#[test]
fn test_tail_truncation_and_head_signature_anchor() {
    let seed = get_test_seed();
    let keypair = DeletionProofKeyPair::generate();
    let base_chain = build_50_entry_chain(seed);

    let head_sig = base_chain.sign_head(keypair.signing_key()).unwrap();
    let published_head_hash = base_chain.head_hash();
    let published_chain_len = base_chain.len();

    // Verify intact original chain against published signature
    assert!(AuditChain::verify_head_signature(&head_sig, &keypair.verifying_key).unwrap());

    // (e) Truncate chain by k entries (1..=10)
    for k in 1..=10 {
        // Scenario 1: Truncate entries list without touching head_hash
        let mut truncated_unmodified_head = base_chain.clone();
        truncated_unmodified_head
            .entries
            .truncate(published_chain_len - k);

        assert!(
            !truncated_unmodified_head.verify_chain().unwrap(),
            "Truncated chain without head_hash update fails verify_chain"
        );

        // Scenario 2: Construct truncated sub-chain by appending first (published_chain_len - k) entries
        let mut truncated_subchain = AuditChain::new();
        for entry in &base_chain.entries[0..(published_chain_len - k)] {
            truncated_subchain
                .append(
                    &entry.schema_id,
                    entry.rules_hash,
                    entry.data_class,
                    entry.doc_id,
                    entry.tx_id,
                    entry.commitment,
                )
                .unwrap();
        }

        // Note: verify_chain returns Ok(true) on a self-consistent sub-chain
        assert!(
            truncated_subchain.verify_chain().unwrap(),
            "Sub-chain is locally consistent"
        );

        // BUT verification against published Head Signature or Head Anchor FAILS
        assert_ne!(
            truncated_subchain.head_hash(),
            published_head_hash,
            "Truncated sub-chain head_hash differs from published head_hash"
        );
        assert_ne!(
            truncated_subchain.entries.last().unwrap().index,
            head_sig.chain_index,
            "Truncated sub-chain index differs from published signed chain_index"
        );

        // Constructing a head signature for the truncated sub-chain yields a distinct head_hash
        let subchain_sig = truncated_subchain.sign_head(keypair.signing_key()).unwrap();
        assert_ne!(
            subchain_sig.head_hash, head_sig.head_hash,
            "Truncated subchain head signature hash must differ from original signed head"
        );
    }
}

#[test]
fn test_tail_appending_and_signature_detection() {
    let seed = get_test_seed();
    let keypair = DeletionProofKeyPair::generate();
    let mut base_chain = build_50_entry_chain(seed);

    let original_head_sig = base_chain.sign_head(keypair.signing_key()).unwrap();
    let original_head_hash = base_chain.head_hash();

    // (f) Append new valid entries to the chain
    base_chain
        .append(
            "schema_appended",
            [0x99u8; 32],
            DataClass::Public,
            DocId(9999),
            TxId(9999),
            None,
        )
        .unwrap();

    // Extended chain is locally valid under verify_chain()
    assert!(base_chain.verify_chain().unwrap());

    // BUT original Head Signature / published anchor detects that chain was appended
    assert_ne!(
        base_chain.head_hash(),
        original_head_hash,
        "Appended chain head_hash differs from published head_hash"
    );
    assert_ne!(
        base_chain.entries.last().unwrap().index,
        original_head_sig.chain_index,
        "Appended chain last index differs from published signed index"
    );
}

#[test]
fn test_signature_verification_edge_cases() {
    let seed = get_test_seed();
    let keypair = DeletionProofKeyPair::generate();
    let wrong_keypair = DeletionProofKeyPair::generate();
    let chain = build_50_entry_chain(seed);

    let head_sig = chain.sign_head(keypair.signing_key()).unwrap();

    // 1. Verification with wrong public key fails
    assert!(!AuditChain::verify_head_signature(&head_sig, &wrong_keypair.verifying_key).unwrap());

    // 2. Corrupted signature bytes fail verification
    let mut corrupted_sig = head_sig.clone();
    corrupted_sig.signature[0] ^= 0xFF;
    assert!(!AuditChain::verify_head_signature(&corrupted_sig, &keypair.verifying_key).unwrap());

    // 3. Signature with mismatched head_hash fails
    let mut mismatched_hash_sig = head_sig.clone();
    mismatched_hash_sig.head_hash[0] ^= 0xAA;
    assert!(
        !AuditChain::verify_head_signature(&mismatched_hash_sig, &keypair.verifying_key).unwrap()
    );

    // 4. Signature with mismatched chain_index fails
    let mut mismatched_idx_sig = head_sig;
    mismatched_idx_sig.chain_index += 10;
    assert!(
        !AuditChain::verify_head_signature(&mismatched_idx_sig, &keypair.verifying_key).unwrap()
    );
}

#[test]
fn test_encrypted_commitment_salt_security_and_tamper() {
    let clock = std::sync::Arc::new(contextra_ports::SystemClock::new());
    let sk = ed25519_dalek::SigningKey::generate(&mut rand::rngs::OsRng);
    let vk = sk.verifying_key();
    let registry = KeyRegistry::new_in_memory(clock, Some(sk), vk);
    let km = KeyManager::try_new("audit-master-passphrase", b"salt-1234567890").unwrap();
    let group_id = 100;

    let real_salt = b"random_commitment_salt_32bytes!";
    let attribute_data = b"sensitive_user_attribute_value";

    // Compute commitment
    let commitment = compute_record_commitment(real_salt, attribute_data);

    // Encrypt salt
    let enc_salt = EncryptedCommitmentSalt::encrypt(&registry, &km, group_id, real_salt).unwrap();

    // 1. Decrypt with valid key registry
    let decrypted_salt = enc_salt.decrypt(&registry).unwrap();
    assert_eq!(decrypted_salt, real_salt);
    assert_eq!(
        compute_record_commitment(&decrypted_salt, attribute_data),
        commitment
    );

    // 2. Tampered ciphertext fails decryption
    let mut tampered_enc = enc_salt.clone();
    tampered_enc.ciphertext[0] ^= 0xFF;
    assert!(tampered_enc.decrypt(&registry).is_err());

    // 3. Corrupted nonce fails decryption
    let mut tampered_nonce = enc_salt.clone();
    tampered_nonce.nonce[0] ^= 0xFF;
    assert!(tampered_nonce.decrypt(&registry).is_err());

    // 4. Decryption with wrong/non-existent group ID fails
    let mut wrong_group = enc_salt.clone();
    wrong_group.group_id = 999;
    assert!(wrong_group.decrypt(&registry).is_err());

    // 5. Decryption after subkey revocation (shredding) fails
    assert!(registry.revoke_subkey(group_id).unwrap());
    assert!(enc_salt.decrypt(&registry).is_err());
}

proptest! {
    #[test]
    fn prop_audit_chain_random_tamper_detection(
        chain_len in 1usize..200usize,
        tamper_pos in 0usize..200usize,
        mutation_type in 0u8..6u8,
    ) {
        let keypair = DeletionProofKeyPair::generate();
        let mut chain = AuditChain::new();

        for i in 0..chain_len {
            chain.append(
                format!("schema_{i}"),
                [i as u8; 32],
                DataClass::Internal,
                DocId(i as u64),
                TxId(i as u64),
                if i % 3 == 0 { Some([(i * 2) as u8; 32]) } else { None },
            ).unwrap();
        }

        let original_head_sig = chain.sign_head(keypair.signing_key()).unwrap();
        let original_head_hash = chain.head_hash();

        let pos = tamper_pos % chain_len;

        match mutation_type {
            0 => {
                // Flip bit in doc_id
                chain.entries[pos].doc_id = DocId(chain.entries[pos].doc_id.0 ^ 1);
                prop_assert!(!chain.verify_chain().unwrap());
            }
            1 => {
                // Delete entry
                chain.entries.remove(pos);
                let chain_ok = chain.verify_chain().unwrap();
                let sig_ok = chain.head_hash() == original_head_hash
                    && AuditChain::verify_head_signature(&original_head_sig, &keypair.verifying_key).unwrap();
                prop_assert!(!chain_ok || !sig_ok);
            }
            2 => {
                // Duplicate entry
                let dupe = chain.entries[pos].clone();
                chain.entries.insert(pos, dupe);
                prop_assert!(!chain.verify_chain().unwrap());
            }
            3 => {
                // Mutate prev_hash
                chain.entries[pos].prev_hash[0] ^= 0xFF;
                prop_assert!(!chain.verify_chain().unwrap());
            }
            4 => {
                // Mutate rules_hash
                chain.entries[pos].rules_hash[0] ^= 0xFF;
                prop_assert!(!chain.verify_chain().unwrap());
            }
            _ => {
                // Append new entry
                chain.append(
                    "appended_schema",
                    [0xEEu8; 32],
                    DataClass::Public,
                    DocId(8888),
                    TxId(8888),
                    None,
                ).unwrap();
                // Chain is locally consistent, but head_hash differs from original head
                prop_assert_ne!(chain.head_hash(), original_head_hash);
            }
        }
    }
}
