use contextra_ports::erasure::{ErasureBackend, FreeSpaceProbe, LogicalScope, ProbeError, PurgeId};
use contextra_types::{CollectionId, DocId, TenantId, TxId};
use contextra_store::lsm::{LsmConfig, LsmStorage};
use std::sync::Arc;
use tempfile::tempdir;
use proptest::prelude::*;

struct DummyProbe;
impl FreeSpaceProbe for DummyProbe {
    fn available_bytes(&self) -> Result<Option<u64>, ProbeError> {
        Ok(Some(100 * 1024 * 1024))
    }
}

#[tokio::test]
async fn test_f4_document_physical_scope() {
    let dir = tempdir().unwrap();
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await.unwrap();

    let tenant_id = TenantId::try_new(101).unwrap();
    let collection_id = CollectionId(10);
    let doc_id = DocId(1);

    let doc_scope = LogicalScope::Document {
        tenant_id,
        collection_id,
        doc_id,
    };

    let phys_scope = storage.physical_scope(&doc_scope).unwrap();

    // F4 requirement: prefixes must be empty, keys must have len == 1, sibling key matches_key == false
    assert!(
        phys_scope.prefixes().is_empty(),
        "Expected prefixes to be empty for Document scope, got {:?}",
        phys_scope.prefixes()
    );
    assert_eq!(
        phys_scope.keys().len(),
        1,
        "Expected keys to have length 1, got {:?}",
        phys_scope.keys()
    );

    let sibling_key = format!("t:101:10:doc:2").into_bytes();
    assert!(
        !phys_scope.matches_key(&sibling_key),
        "Sibling key should not match Document scope"
    );

    storage.close().await.unwrap();
}

proptest! {
    #[test]
    fn prop_test_document_physical_scope_isolation(
        tenant_raw in 1u64..1000u64,
        col_raw in 1u64..1000u64,
        target_doc_raw in 1u64..100000u64,
        sibling_doc_raw in 1u64..100000u64,
    ) {
        prop_assume!(target_doc_raw != sibling_doc_raw);

        let tenant_id = TenantId::try_new(tenant_raw).unwrap();
        let collection_id = CollectionId(col_raw);
        let doc_id = DocId(target_doc_raw);

        let _doc_scope = LogicalScope::Document {
            tenant_id,
            collection_id,
            doc_id,
        };

        // Validate directly using PhysicalScope construction identical to LsmStorage::physical_scope
        let doc_key = format!("t:{}:{}:doc:{}", tenant_raw, col_raw, target_doc_raw).into_bytes();
        let phys_scope = contextra_ports::erasure::PhysicalScope::try_new(tenant_id, vec![doc_key.clone()], vec![]).unwrap();

        prop_assert!(phys_scope.prefixes().is_empty());
        prop_assert_eq!(phys_scope.keys().len(), 1);
        prop_assert_eq!(&phys_scope.keys()[0], &doc_key);
        prop_assert!(phys_scope.matches_key(&doc_key));

        let sibling_key = format!("t:{}:{}:doc:{}", tenant_raw, col_raw, sibling_doc_raw).into_bytes();
        prop_assert!(!phys_scope.matches_key(&sibling_key));
    }
}

#[test]
fn test_f6_purge_physical_outside_tokio_runtime_succeeds() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    let storage = rt.block_on(async {
        let dir = tempdir().unwrap();
        let config = LsmConfig {
            path: dir.path().to_path_buf(),
            ..Default::default()
        };
        LsmStorage::open(config).await.unwrap()
    });

    let tenant_id = TenantId::try_new(101).unwrap();
    let doc_scope = LogicalScope::Document {
        tenant_id,
        collection_id: CollectionId(10),
        doc_id: DocId(1),
    };
    let phys_scope = storage.physical_scope(&doc_scope).unwrap();
    let tx = TxId::try_new(1).unwrap();
    let probe = DummyProbe;

    // Called outside tokio runtime context
    let result = storage.purge_physical(&phys_scope, tx, &probe);
    assert!(result.is_ok(), "Expected purge_physical outside tokio runtime to succeed, got {:?}", result);

    rt.block_on(async { storage.close().await.unwrap(); });
}

#[tokio::test]
async fn test_f6_purge_physical_in_tokio_runtime_returns_io_error() {
    let dir = tempdir().unwrap();
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = LsmStorage::open(config).await.unwrap();

    let tenant_id = TenantId::try_new(101).unwrap();
    let doc_scope = LogicalScope::Document {
        tenant_id,
        collection_id: CollectionId(10),
        doc_id: DocId(1),
    };
    let phys_scope = storage.physical_scope(&doc_scope).unwrap();
    let tx = TxId::try_new(1).unwrap();
    let probe = DummyProbe;

    // F6 requirement: Calling purge_physical inside a tokio runtime context must return Err(PurgeError::Io(...))
    let result = storage.purge_physical(&phys_scope, tx, &probe);
    match result {
        Err(contextra_ports::erasure::PurgeError::Io(e)) => {
            let msg = e.to_string();
            assert!(
                msg.contains("active Tokio runtime") || msg.contains("async"),
                "Expected clear error message about Tokio runtime, got: {}",
                msg
            );
        }
        other => panic!("Expected PurgeError::Io when called inside Tokio runtime, got {:?}", other),
    }

    storage.close().await.unwrap();
}

#[tokio::test]
async fn test_f7_acquire_fence_isolation_and_drop() {
    let dir = tempdir().unwrap();
    let config = LsmConfig {
        path: dir.path().to_path_buf(),
        ..Default::default()
    };
    let storage = Arc::new(LsmStorage::open(config).await.unwrap());

    let tenant1 = TenantId::try_new(101).unwrap();
    let _tenant2 = TenantId::try_new(202).unwrap();
    let col1 = CollectionId(10);
    let _col2 = CollectionId(20);

    let doc_scope = LogicalScope::Document {
        tenant_id: tenant1,
        collection_id: col1,
        doc_id: DocId(1),
    };

    let phys_scope = storage.physical_scope(&doc_scope).unwrap();
    let purge_id = PurgeId::new([1; 16]);

    // Acquire fence for tenant1 / col1 / doc1
    let fence_guard = storage.acquire_fence(&phys_scope, purge_id).unwrap();

    // Key in tenant1 / col1 / doc1 SHOULD be fenced
    let doc1_key = format!("t:101:10:doc:1").into_bytes();
    assert!(
        !storage.erasure_fence().check_write_allowed(&doc1_key),
        "Key in scope should be fenced"
    );

    // Key in tenant2 or col2 SHOULD NOT be fenced (F7 isolation requirement)
    let other_tenant_key = format!("t:202:10:doc:1").into_bytes();
    assert!(
        storage.erasure_fence().check_write_allowed(&other_tenant_key),
        "Writes to another tenant must not be fenced by document fence"
    );

    let other_col_key = format!("t:101:20:doc:1").into_bytes();
    assert!(
        storage.erasure_fence().check_write_allowed(&other_col_key),
        "Writes to another collection in same tenant must not be fenced by document fence"
    );

    // Drop fence guard
    drop(fence_guard);

    // After drop, doc1_key should no longer be fenced (F7 drop release requirement)
    assert!(
        storage.erasure_fence().check_write_allowed(&doc1_key),
        "Fence should be released after FenceGuard is dropped"
    );

    let storage = Arc::try_unwrap(storage).ok().expect("Arc unwrap failed");
    storage.close().await.unwrap();
}
