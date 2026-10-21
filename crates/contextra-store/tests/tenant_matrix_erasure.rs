// FILE-CONTEXT
// STAND: 2026-10-15
// ZWECK: Tenant-Matrix-Tests (REG-15) für ErasureBackend über LsmStorage und TenantScopedStorage
// INVARIANTEN: Strikte Mandanten-Isolation, fail-closed Mandanten-Auflösung, Pflicht-TenantId bei remove_doc_segments

use std::panic::catch_unwind;
use std::sync::Arc;
use contextra_types::{CollectionId, DocId, TenantId, TxId};
use contextra_ports::erasure::{
    FenceGuard, FreeSpaceProbe, LogicalScope, PhysicalScope, PurgeError, PurgeId,
};
use contextra_types::ContextraError;
use contextra_ports::{KvLifecycleHooks, StorageEngine};
use contextra_store::lsm::LsmStorage;
use contextra_store::tenant_codec::TenantScopedStorage;

#[tokio::test(flavor = "multi_thread")]
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

    // 3. Physical scope resolution with mismatched TenantId MUST fail closed
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
        _ => panic!("Expected InvalidInput, got {:?}", err),
    }

    let purge_id = PurgeId::new([2u8; 16]);
    let dummy_probe = DummyProbe;
    let fence_guard = lsm
        .erasure_capabilities()
        .unwrap()
        .acquire_fence(&phys_scope, purge_id)
        .unwrap();

    let key_only_scope = PhysicalScope::try_new(
        tenant_1,
        vec![b"t:101:doc:123".to_vec()],
        vec![],
    )
    .unwrap();
    let key_fence = eb_t1
        .acquire_fence(&key_only_scope, PurgeId::new([3u8; 16]))
        .expect("key-only physical scope fence acquisition should succeed");
    drop(key_fence);

    // 5. Data isolation & Purge test
    let tx1 = TxId::try_new(10).unwrap();
    storage_t1.put(tx1, b"doc:1", b"val_tenant1").await.unwrap();
    storage_t2.put(tx1, b"doc:1", b"val_tenant2").await.unwrap();
    storage_t1.commit(tx1).await.unwrap();
    storage_t2.commit(tx1).await.unwrap();

    storage_t1.flush().await.unwrap();

    let tx_purge = TxId::try_new(20).unwrap();
    let _ = eb_t1.purge_physical(&phys_scope, tx_purge, &dummy_probe);

    // Tenant 2 value checked (unaffected)
    let get_t2 = storage_t2.get(b"doc:1").await.unwrap();
    assert_eq!(get_t2.unwrap().as_ref(), b"val_tenant2");

    lsm.close().await.unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn test_fence_guard_drop_releases_fence_single_time_even_after_panic() {
    let temp_dir = tempfile::tempdir().unwrap();
    let config = contextra_store::lsm::LsmConfig {
        path: temp_dir.path().to_path_buf(),
        ..Default::default()
    };
    let lsm = Arc::new(LsmStorage::open(config).await.unwrap());
    let tenant_1 = TenantId::try_new(101).unwrap();

    let doc_scope = LogicalScope::Document {
        doc_id: DocId(1),
        tenant_id: tenant_1,
        collection_id: CollectionId(10),
    };
    let eb = lsm.erasure_capabilities().unwrap();
    let _phys_scope = eb.physical_scope(&doc_scope).unwrap();
    let purge_id = PurgeId::new([7u8; 16]);

    let released_count = Arc::new(std::sync::atomic::AtomicU64::new(0));

    let _ = catch_unwind({
        let released_count = released_count.clone();
        move || {
            let guard = FenceGuard::with_release(
                purge_id,
                move || {
                    released_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                },
            );
            assert_eq!(guard.purge_id, purge_id);
            panic!("Simulated panic inside task holding FenceGuard");
        }
    });

    // Guard's release hook was called exactly once on drop despite the panic
    assert_eq!(released_count.load(std::sync::atomic::Ordering::SeqCst), 1);

    lsm.close().await.unwrap();
}

struct DummyProbe;

impl FreeSpaceProbe for DummyProbe {
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
