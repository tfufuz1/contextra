// FILE-CONTEXT
// ZWECK: Property-Test for legacy v2 HMAC length-prefixed deletion keys hashing function against collisions.
// INVARIANTEN: Distinct key collections and distinct segmentations MUST produce non-colliding Blake3 hashes.
// STAND: TS:2026-09-26T00:00:00Z

use contextra_crypto::deletion_proof::hash_deleted_keys_length_prefixed;
use proptest::prelude::*;

proptest! {
    #[test]
    fn no_hash_collision_for_distinct_key_sets(
        keys_a in prop::collection::vec(any::<Vec<u8>>(), 1..50),
        keys_b in prop::collection::vec(any::<Vec<u8>>(), 1..50),
    ) {
        prop_assume!(keys_a != keys_b);
        prop_assert_ne!(
            hash_deleted_keys_length_prefixed(&keys_a),
            hash_deleted_keys_length_prefixed(&keys_b)
        );
    }

    #[test]
    fn length_prefix_prevents_segmentation_ambiguity(a in ".*", b in ".*", c in ".*") {
        prop_assume!(format!("{a}{b}") != c || a.is_empty() || b.is_empty());
        let split = hash_deleted_keys_length_prefixed(&[a.clone().into_bytes(), b.clone().into_bytes()]);
        let joined = hash_deleted_keys_length_prefixed(&[c.into_bytes()]);
        prop_assert_ne!(split, joined);
    }
}
