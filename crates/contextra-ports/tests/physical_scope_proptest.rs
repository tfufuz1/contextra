//! Proptest property-based tests for PhysicalScope invariants (§5.4).

use contextra_ports::erasure::{ErasureError, PhysicalScope, ScopeOrigin};
use contextra_types::TenantId;
use proptest::prelude::*;

fn dummy_origin(tenant_id_num: u64) -> ScopeOrigin {
    let mut hasher = blake3::Hasher::new();
    hasher.update(&tenant_id_num.to_le_bytes());
    ScopeOrigin::new(999, *hasher.finalize().as_bytes())
}

proptest! {
    #[test]
    fn prop_empty_scope_rejected(tenant_id_num in 1u64..10_000) {
        let tenant_id = TenantId(tenant_id_num);
        let origin = dummy_origin(tenant_id_num);
        let res = PhysicalScope::try_new(tenant_id, origin, vec![], vec![]);
        prop_assert_eq!(res, Err(ErasureError::MalformedScope));
    }

    #[test]
    fn prop_prefix_without_colon_rejected(tenant_id_num in 1u64..10_000) {
        let tenant_id = TenantId(tenant_id_num);
        let origin = dummy_origin(tenant_id_num);
        let bad_prefix = format!("t:{}:10", tenant_id_num).into_bytes(); // missing trailing colon
        let res = PhysicalScope::try_new(tenant_id, origin, vec![], vec![bad_prefix]);
        prop_assert_eq!(res, Err(ErasureError::MalformedScope));
    }

    #[test]
    fn prop_foreign_tenant_key_rejected(
        tenant_id_num in 1u64..10_000,
        other_tenant_num in 10_001u64..20_000,
        suffix in "[a-z0-9]{1,10}"
    ) {
        let tenant_id = TenantId(tenant_id_num);
        let origin = dummy_origin(tenant_id_num);
        let bad_key = format!("t:{}:{}", other_tenant_num, suffix).into_bytes();
        let res = PhysicalScope::try_new(tenant_id, origin, vec![bad_key], vec![]);
        prop_assert_eq!(res, Err(ErasureError::MalformedScope));
    }

    #[test]
    fn prop_empty_prefix_rejected(tenant_id_num in 1u64..10_000) {
        let tenant_id = TenantId(tenant_id_num);
        let origin = dummy_origin(tenant_id_num);
        let res = PhysicalScope::try_new(tenant_id, origin, vec![], vec![vec![]]);
        prop_assert_eq!(res, Err(ErasureError::MalformedScope));
    }

    #[test]
    fn prop_collection_ids_4_and_42_prefix_isolation(tenant_id_num in 1u64..10_000) {
        let tenant_id = TenantId(tenant_id_num);
        let origin = dummy_origin(tenant_id_num);

        let prefix_col4 = format!("t:{}:4:", tenant_id_num).into_bytes();
        let prefix_col42 = format!("t:{}:42:", tenant_id_num).into_bytes();
        let key_col42 = format!("t:{}:42:doc:1", tenant_id_num).into_bytes();

        let scope_col4 = PhysicalScope::try_new(tenant_id, origin, vec![], vec![prefix_col4.clone()]).unwrap();
        let scope_col42 = PhysicalScope::try_new(tenant_id, origin, vec![], vec![prefix_col42.clone()]).unwrap();

        // Scope col4 prefix "t:1:4:" MUST NOT match key "t:1:42:doc:1"
        prop_assert!(!scope_col4.matches_key(&key_col42));
        prop_assert!(!scope_col4.matches_key(&prefix_col42));

        // Scope col42 MUST match key "t:1:42:doc:1"
        prop_assert!(scope_col42.matches_key(&key_col42));
        prop_assert!(scope_col42.matches_key(&prefix_col42));
    }

    #[test]
    fn prop_valid_scope_canonical_and_matches(
        tenant_id_num in 1u64..10_000,
        suffix1 in "[a-z0-9]{1,5}",
        suffix2 in "[a-z0-9]{6,10}"
    ) {
        let tenant_id = TenantId(tenant_id_num);
        let origin = dummy_origin(tenant_id_num);
        let prefix1 = format!("t:{}:{}:", tenant_id_num, suffix1).into_bytes();
        let key1 = format!("t:{}:{}:{}", tenant_id_num, suffix1, suffix2).into_bytes();
        let foreign_key = format!("t:{}:other", tenant_id_num + 99999).into_bytes();

        let scope = PhysicalScope::try_new(
            tenant_id,
            origin,
            vec![key1.clone()],
            vec![prefix1.clone()],
        ).expect("valid scope should succeed");

        prop_assert!(!scope.is_empty());

        // Key1 is subsumed by prefix1, so keys slice becomes empty while prefixes contains prefix1
        prop_assert_eq!(scope.prefixes(), &[prefix1.clone()]);
        prop_assert!(scope.keys().is_empty());

        // Matches key1 via prefix1 match
        prop_assert!(scope.matches_key(&key1));
        prop_assert!(scope.matches_key(&prefix1));
        prop_assert!(!scope.matches_key(&foreign_key));

        // Digest is non-zero
        let digest = scope.scope_digest();
        prop_assert_ne!(digest, [0u8; 32]);
    }

    #[test]
    fn prop_digest_determinism(
        tenant_id_num in 1u64..10_000,
        key_suffix in "[a-z0-9]{1,10}"
    ) {
        let tenant_id = TenantId(tenant_id_num);
        let origin = dummy_origin(tenant_id_num);
        let prefix = format!("t:{}:{}:", tenant_id_num, key_suffix).into_bytes();

        let scope1 = PhysicalScope::try_new(tenant_id, origin, vec![], vec![prefix.clone()]).unwrap();
        let scope2 = PhysicalScope::try_new(tenant_id, origin, vec![], vec![prefix]).unwrap();

        prop_assert_eq!(scope1.scope_digest(), scope2.scope_digest());
    }
}
