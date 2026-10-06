// ZWECK: Integration tests for KV lifecycle hooks production wiring, rollback, document deletion, and tenant purge isolation.
// ORAKEL: Direct comparison with TenantIsolatedKvStore state vs Contextra facade behavior.

use contextra_engine::*;
use contextra_kvcache::{KvSegment, TenantIsolatedKvStore};
use contextra_ports::KvLifecycleHooks;
use contextra_types::{DocId, TenantId};
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_kv_hooks_not_wired_by_default_repro() -> contextra_types::Result<()> {
    let dir = tempdir().unwrap();
    let db = Contextra::open(dir.path()).await?;

    let col = db.collection("default").await?;
    // [BELEGT]: In Contextra::open, kv_hooks is None because kv_store is not stored as a field on Contextra
    // or passed to collections during initialization/retrieval.
    assert!(
        col.kv_hooks().is_some(),
        "kv_hooks is set on default collection in production Contextra::open path"
    );

    db.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_eviction_worker_and_hooks_share_same_store() -> contextra_types::Result<()> {
    let store: Arc<TenantIsolatedKvStore> = Arc::new(TenantIsolatedKvStore::new());
    let _worker = build_eviction_worker(store.clone());

    let dir = tempdir().unwrap();
    let db = Contextra::open(dir.path()).await?;
    let mut col = db.collection("default").await?;

    col.set_kv_hooks(store.clone());

    let hooks = col
        .kv_hooks()
        .expect("kv_hooks should be set on collection after set_kv_cache_store");

    // Verify pointer equality between store and hooks trait object pointer
    let store_ptr = Arc::as_ptr(&store) as *const ();
    let hooks_ptr = Arc::as_ptr(&hooks) as *const ();
    assert_eq!(
        store_ptr, hooks_ptr,
        "EvictionWorker store and Collection kv_hooks share the exact same Arc store instance"
    );

    db.close().await?;
    Ok(())
}

#[tokio::test]
async fn test_rollback_doc_delete_tenant_purge_kv_segment_isolation() -> contextra_types::Result<()>
{
    let store = Arc::new(TenantIsolatedKvStore::new());
    let tenant1 = TenantId::try_new(1).unwrap();
    let tenant2 = TenantId::try_new(2).unwrap();

    // Populate KV segments for both tenants
    store.insert_segment(tenant1, KvSegment::new(tenant1, 100, vec![1; 256]));
    store.insert_segment(tenant1, KvSegment::new(tenant1, 101, vec![1; 256]));
    store.insert_segment(tenant2, KvSegment::new(tenant2, 200, vec![2; 256]));

    let hooks: &dyn KvLifecycleHooks = store.as_ref();

    // 1. Rollback scenario for tenant1
    hooks.on_rollback(tenant1, &[100]);
    assert_eq!(
        store.get_segments(tenant1),
        vec![101],
        "Rollback removed chunk 100 for tenant1"
    );
    assert_eq!(
        store.get_segments(tenant2),
        vec![200],
        "Tenant2 control segments remain untouched"
    );

    // 2. Document deletion scenario for tenant1
    let doc101 = DocId::new(101);
    hooks.remove_doc_segments(tenant1, doc101);
    assert_eq!(
        store.get_segments(tenant1),
        Vec::<u64>::new(),
        "Document deletion removed segment 101 for tenant1"
    );
    assert_eq!(
        store.get_segments(tenant2),
        vec![200],
        "Tenant2 control segments remain untouched"
    );

    // 3. Tenant purge scenario for tenant2
    hooks.purge_tenant(tenant2);
    assert_eq!(
        store.get_segments(tenant2),
        Vec::<u64>::new(),
        "Tenant purge cleared all segments for tenant2"
    );

    Ok(())
}
