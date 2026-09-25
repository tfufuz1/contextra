use contextra_crypto::deletion_proof::{
    DeletionProof, DeletionProofKeyPair, DeletionScope,
};
use contextra_types::{TenantId, TxId};
use proptest::prelude::*;

fn hash_keys(keys: &[Vec<u8>]) -> [u8; 32] {
    let keypair = DeletionProofKeyPair::generate();
    let tenant_id = match TenantId::try_new(1) {
        Ok(t) => t,
        Err(_) => return [0u8; 32],
    };
    let scope = DeletionScope::Tenant { tenant_id };
    let proof = match DeletionProof::create_v3(
        scope,
        keys.to_vec(),
        TxId(1),
        vec![],
        vec![],
        keypair.signing_key(),
    ) {
        Ok(p) => p,
        Err(_) => return [0u8; 32],
    };

    proof.deleted_keys_hash
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]

    #[test]
    fn prop_no_hash_collision_across_key_splits(
        keys_a in prop::collection::vec(prop::collection::vec(any::<u8>(), 0..64), 1..10),
        keys_b in prop::collection::vec(prop::collection::vec(any::<u8>(), 0..64), 1..10),
    ) {
        // Only assert non-equality if the multiset of keys is actually different after sorting
        let mut sorted_a = keys_a.clone();
        sorted_a.sort();
        let mut sorted_b = keys_b.clone();
        sorted_b.sort();

        if sorted_a != sorted_b {
            let hash_a = hash_keys(&keys_a);
            let hash_b = hash_keys(&keys_b);
            prop_assert_ne!(hash_a, hash_b);
        }
    }

    #[test]
    fn prop_hash_determinism_independent_of_input_order(
        keys in prop::collection::vec(prop::collection::vec(any::<u8>(), 0..64), 1..10),
        seed in any::<u64>(),
    ) {
        let mut shuffled = keys.clone();
        // Deterministic simple shuffle using LCG
        let len = shuffled.len();
        if len > 1 {
            let mut state = seed;
            for i in (1..len).rev() {
                state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
                let j = (state as usize) % (i + 1);
                shuffled.swap(i, j);
            }
        }

        let hash_orig = hash_keys(&keys);
        let hash_shuffled = hash_keys(&shuffled);
        prop_assert_eq!(hash_orig, hash_shuffled);
    }
}
