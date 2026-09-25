use contextra_crypto::deletion_proof::{
    DeletionLayer, DeletionProof, DeletionProofKeyPair, DeletionScope, ExcludedScope,
    LayerCleanupProof,
};
use contextra_types::{DocId, TenantId, TxId};
use proptest::prelude::*;

fn hash_keys_v3(keys: &[Vec<u8>]) -> [u8; 32] {
    let keypair = DeletionProofKeyPair::generate();
    let tenant_id = TenantId::try_new(1).expect("valid tenant_id");
    let scope = DeletionScope::Tenant { tenant_id };
    let proof = DeletionProof::create_v3(
        scope,
        keys.to_vec(),
        TxId(1),
        vec![],
        vec![],
        keypair.signing_key(),
    )
    .expect("create_v3 successful");

    proof.deleted_keys_hash
}

fn hash_keys_v2(keys: &[Vec<u8>]) -> [u8; 32] {
    let hmac_key = vec![0u8; 32];
    let tenant_id = TenantId::try_new(1).expect("valid tenant_id");
    let scope = DeletionScope::Tenant { tenant_id };
    let proof = DeletionProof::create(
        scope,
        keys.to_vec(),
        TxId(1),
        vec![],
        vec![],
        &hmac_key,
    )
    .expect("create v2 successful");

    proof.deleted_keys_hash
}

#[test]
fn test_explicit_hash_collision_ab_c_vs_a_bc() {
    let keys1 = vec![b"ab".to_vec(), b"c".to_vec()];
    let keys2 = vec![b"a".to_vec(), b"bc".to_vec()];

    let hash1_v3 = hash_keys_v3(&keys1);
    let hash2_v3 = hash_keys_v3(&keys2);
    assert_ne!(
        hash1_v3, hash2_v3,
        "v3: ['ab', 'c'] and ['a', 'bc'] MUST produce different hashes due to length-prefix"
    );

    let hash1_v2 = hash_keys_v2(&keys1);
    let hash2_v2 = hash_keys_v2(&keys2);
    assert_ne!(
        hash1_v2, hash2_v2,
        "v2: ['ab', 'c'] and ['a', 'bc'] MUST produce different hashes due to length-prefix"
    );
}

#[test]
fn test_input_order_determinism() {
    let keys_unordered = vec![b"z".to_vec(), b"a".to_vec(), b"m".to_vec()];
    let keys_sorted = vec![b"a".to_vec(), b"m".to_vec(), b"z".to_vec()];

    let hash_unordered = hash_keys_v3(&keys_unordered);
    let hash_sorted = hash_keys_v3(&keys_sorted);

    assert_eq!(
        hash_unordered, hash_sorted,
        "Key sorting before length-prefixed hashing guarantees deterministic key hash"
    );
}

#[test]
fn test_v1_v2_v3_signature_regression() {
    let hmac_key = vec![42u8; 32];
    let tenant_id = TenantId::try_new(1).expect("valid tenant_id");
    let scope = DeletionScope::Document {
        doc_id: DocId::new(10),
        tenant_id,
    };
    let layer_proof =
        LayerCleanupProof::new_after_verified_empty(DeletionLayer::LsmMemtable, 0)
            .expect("layer proof creation");

    // Test v2 HMAC proof creation and verification
    let proof_v2 = DeletionProof::create(
        scope.clone(),
        vec![b"key1".to_vec(), b"key2".to_vec()],
        TxId::new(100),
        vec![layer_proof.clone()],
        vec![ExcludedScope::LlmParameterMemory],
        &hmac_key,
    )
    .expect("create v2 proof");

    assert_eq!(proof_v2.signature_version, 2);
    assert!(
        proof_v2.verify(&hmac_key).expect("verify v2"),
        "v2 HMAC signature MUST verify successfully"
    );

    // Test v3 Ed25519 proof creation and verification
    let keypair = DeletionProofKeyPair::generate();
    let proof_v3 = DeletionProof::create_v3(
        scope,
        vec![b"key1".to_vec(), b"key2".to_vec()],
        TxId::new(100),
        vec![layer_proof],
        vec![ExcludedScope::LlmParameterMemory],
        keypair.signing_key(),
    )
    .expect("create v3 proof");

    assert_eq!(proof_v3.signature_version, 3);
    assert!(
        proof_v3.verify(&keypair.verifying_key).expect("verify v3"),
        "v3 Ed25519 signature MUST verify successfully"
    );
    assert!(
        proof_v3.verify_external(&keypair.verifying_key).is_ok(),
        "v3 external verification MUST succeed"
    );
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]

    #[test]
    fn prop_no_hash_collision_across_key_splits(
        keys_a in prop::collection::vec(prop::collection::vec(any::<u8>(), 0..64), 1..10),
        keys_b in prop::collection::vec(prop::collection::vec(any::<u8>(), 0..64), 1..10),
    ) {
        let mut sorted_a = keys_a.clone();
        sorted_a.sort();
        let mut sorted_b = keys_b.clone();
        sorted_b.sort();

        if sorted_a != sorted_b {
            let hash_a = hash_keys_v3(&keys_a);
            let hash_b = hash_keys_v3(&keys_b);
            prop_assert_ne!(hash_a, hash_b);
        }
    }

    #[test]
    fn prop_hash_determinism_independent_of_input_order(
        keys in prop::collection::vec(prop::collection::vec(any::<u8>(), 0..64), 1..10),
        seed in any::<u64>(),
    ) {
        let mut shuffled = keys.clone();
        let len = shuffled.len();
        if len > 1 {
            let mut state = seed;
            for i in (1..len).rev() {
                state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                let j = (state as usize) % (i + 1);
                shuffled.swap(i, j);
            }
        }

        let hash_orig = hash_keys_v3(&keys);
        let hash_shuffled = hash_keys_v3(&shuffled);
        prop_assert_eq!(hash_orig, hash_shuffled);
    }
}
