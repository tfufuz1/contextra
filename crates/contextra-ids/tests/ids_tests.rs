//! Conformance and property tests for contextra-ids copied from types

use contextra_ids::*;

#[test]
fn test_doc_id_from_key_and_display() {
    let id = DocId::from_key("test_doc").unwrap();
    assert_ne!(id.as_u64(), 0);
    assert_eq!(DocId::from_key(""), Err(IdError::EmptyKey));
    assert!(format!("{}", id).starts_with("DocId("));
}

#[test]
fn test_entity_id_from_key_and_display() {
    let id = EntityId::from_key("test_entity").unwrap();
    assert_ne!(id.as_u64(), 0);
    assert_eq!(EntityId::from_key(""), Err(IdError::EmptyKey));
    assert!(format!("{}", id).starts_with("EntityId("));
}

#[test]
fn test_tx_id_invariants() {
    let invalid = TxId::INVALID;
    assert!(!invalid.is_valid());
    let reg = TxId::new(100);
    assert!(reg.is_valid());
    assert!(!reg.is_internal());
    let internal = TxId::new(TxId::INTERNAL_BASE + 1);
    assert!(internal.is_valid());
    assert!(internal.is_internal());
}

#[test]
fn test_tenant_id_invariants() {
    let sys = TenantId::SYSTEM;
    assert!(sys.is_system());
    assert_eq!(TenantId::try_new(0), Err(IdError::TenantSystemReserved));
    let t = TenantId::try_new(42).unwrap();
    assert_eq!(t.inner(), 42);
    assert!(!t.is_system());
}

#[test]
fn test_collection_id_invariants() {
    assert_eq!(CollectionId::try_new(0), Err(IdError::CollectionZero));
    let c = CollectionId::try_new(10).unwrap();
    assert_eq!(c.inner(), 10);
}

#[test]
fn test_refs_max_len_and_empty() {
    assert_eq!(DocKey::try_new(vec![]), Err(IdError::EmptyIdentifier));
    assert_eq!(
        DocKey::try_new(vec![b'x'; MAX_REF_LEN + 1]),
        Err(IdError::IdentifierTooLong)
    );
    let key = DocKey::try_new(b"hello".to_vec()).unwrap();
    assert_eq!(key.as_bytes(), b"hello");
}

#[test]
fn test_tenant_scoped_isolation() {
    let scoped = TenantScoped::new(TenantId::new(1), 100);
    assert_eq!(scoped.tenant_id(), &TenantId::new(1));
    assert_eq!(*scoped.value(), 100);
    assert_eq!(
        scoped.clone().into_inner_checked(&TenantId::new(1)),
        Ok(100)
    );
    assert_eq!(
        scoped.into_inner_checked(&TenantId::new(2)),
        Err(TenantScopeViolation::Mismatch {
            expected: TenantId::new(2),
            actual: TenantId::new(1)
        })
    );
}
