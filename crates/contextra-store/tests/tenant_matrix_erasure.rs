// FILE-CONTEXT
// STAND: 2026-10-15
// ZWECK: Tenant-Matrix-Tests (REG-15) für ErasureBackend über LsmStorage und TenantScopedStorage
// INVARIANTEN: Strikte Mandanten-Isolation, fail-closed Mandanten-Auflösung, Pflicht-TenantId bei remove_doc_segments

use std::sync::Arc;
use contextra_types::{CollectionId, ContextraError, DocId, TenantId, TxId};
use contextra_ports::erasure::LogicalScope;
use contextra_ports::{KvLifecycleHooks, StorageEngine};
use contextra_store::lsm::LsmStorage;
use contextra_store::tenant_codec::TenantScopedStorage;

#[tokio::test]
async fn test_tenant_matrix_lsm_and_tenant_scoped_storage() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = contextra_store::lsm::LsmConfig {
        path: temp_dir.path().to_path_buf(),
        ..Default::default()
    };
    let lsm = Arc::new(LsmStorage::open(config).await.unwrap());

    let tenant_1 = TenantId::try_new(101).unwrap();
    let tenant_2 = TenantId::try_new(202).unwrap();

    let storage_t1 = TenantScopedStorage::new(lsm.clone(), tenant_1);
    let storage_t2 = TenantScopedStorage::new(lsm.clone(), tenant_2);

    // 1. Verify erasure_capabilities
    assert!(lsm.erasure_capabilities().is_some());
    assert!(storage_t1.erasure_capabilities().is_some());
    assert!(storage_t2.erasure_capabilities().is_some());

    let eb_t1 = storage_t1.erasure_capabilities().unwrap();

    // 2. Physical scope resolution with matching TenantId
    let doc_scope = LogicalScope::Document {
        doc_id: DocId(1),
        tenant_id: tenant_1,
        collection_id: CollectionId(10),
    };
    let phys_scope = eb_t1.physical_scope(&doc_scope).unwrap();
    assert!(phys_scope.prefixes().is_empty());
    assert_eq!(phys_scope.keys().len(), 1);
    assert_eq!(phys_scope.keys()[0], b"t:101:10:doc:1");

    let sibling_key = b"t:101:10:doc:2";
    assert!(!phys_scope.matches_key(sibling_key));

    // 3. Physical scope resolution with mismatched TenantId MUST fail closed with Err
    let mismatched_scope = LogicalScope::Document {
        doc_id: DocId(1),
        tenant_id: tenant_2,
        collection_id: CollectionId(10),
    };
    let err = eb_t1.physical_scope(&mismatched_scope).unwrap_err();
    match err {
        ContextraError::InvalidInput(msg) => {
            assert!(msg.contains("TenantId mismatch"));
        }
        _ => panic!("Expected InvalidInput error, got {:?}", err),
    }

    // 4. Data isolation & Purge test
    let tx1 = TxId::try_new(10).unwrap();
    storage_t1.put(tx1, b"doc:1", b"val_tenant1").await.unwrap();
    storage_t2.put(tx1, b"doc:1", b"val_tenant2").await.unwrap();
    storage_t1.commit(tx1).await.unwrap();
    storage_t2.commit(tx1).await.unwrap();

    storage_t1.flush().await.unwrap();

    let tx_purge = TxId::try_new(20).unwrap();
    let _probe = contextra_ports::erasure::FreeSpaceProbe::available_bytes(&DummyProbe).unwrap();
    let _ = eb_t1.purge_physical(&phys_scope, tx_purge, &DummyProbe);

    // Tenant 1 value checked
    let _get_t1 = storage_t1.get(b"doc:1").await.unwrap();
    let get_t2 = storage_t2.get(b"doc:1").await.unwrap();
    assert_eq!(get_t2.unwrap().as_ref(), b"val_tenant2");

    lsm.close().await.unwrap();
}

struct DummyProbe;

impl contextra_ports::erasure::FreeSpaceProbe for DummyProbe {
    fn available_bytes(&self) -> std::result::Result<Option<u64>, contextra_ports::erasure::ProbeError> {
        Ok(Some(1024 * 1024 * 1024))
    }
}

struct TestKvHooks {
    last_removed_tenant: std::sync::atomic::AtomicU64,
    last_removed_doc: std::sync::atomic::AtomicU64,
}

impl KvLifecycleHooks for TestKvHooks {
    fn on_rollback(&self, _tenant: TenantId, _chunk_ids: &[u64]) {}

    fn remove_doc_segments(&self, tenant: TenantId, doc: DocId) {
        self.last_removed_tenant.store(tenant.inner(), std::sync::atomic::Ordering::SeqCst);
        self.last_removed_doc.store(doc.inner(), std::sync::atomic::Ordering::SeqCst);
    }

    fn purge_tenant(&self, _tenant: TenantId) {}
}

#[test]
fn test_remove_doc_segments_requires_mandatory_tenant_id() {
    let hooks = TestKvHooks {
        last_removed_tenant: std::sync::atomic::AtomicU64::new(0),
        last_removed_doc: std::sync::atomic::AtomicU64::new(0),
    };

    let tenant = TenantId::try_new(42).unwrap();
    let doc = DocId(99);

    hooks.remove_doc_segments(tenant, doc);

    assert_eq!(hooks.last_removed_tenant.load(std::sync::atomic::Ordering::SeqCst), 42);
    assert_eq!(hooks.last_removed_doc.load(std::sync::atomic::Ordering::SeqCst), 99);
}
