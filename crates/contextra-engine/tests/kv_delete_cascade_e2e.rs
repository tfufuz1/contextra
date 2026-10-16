#![forbid(unsafe_code)]

use contextra_kvcache::{KvSegment, TenantIsolatedKvStore};
use contextra_types::{DocId, TenantId};
use std::sync::Arc;

const CANARY_MARKER: &[u8] = b"CANARY_KV_CASCADE_MARKER_999";

fn search_directory_for_canary(dir_path: &std::path::Path, canary: &[u8]) -> bool {
    if !dir_path.exists() {
        return false;
    }
    let mut stack = vec![dir_path.to_path_buf()];
    while let Some(path) = stack.pop() {
        if path.is_dir() {
            if let Ok(entries) = std::fs::read_dir(&path) {
                for entry in entries.flatten() {
                    stack.push(entry.path());
                }
            }
        } else if path.is_file() {
            if let Ok(content) = std::fs::read(&path) {
                if content.windows(canary.len()).any(|w| w == canary) {
                    return true;
                }
            }
        }
    }
    false
}

#[tokio::test]
async fn test_kv_delete_cascade_removes_segment_and_canary() {
    let dir = tempfile::tempdir().unwrap();
    let db = contextra_engine::Contextra::open(dir.path()).await.unwrap();

    let kv_store = Arc::new(TenantIsolatedKvStore::new());
    // Attach kv_hooks to Contextra instance (collection inherits it)
    db.set_kv_hooks(kv_store.clone());

    let tenant = TenantId::try_new(101).unwrap();
    let col = db.collection_for_tenant("test_col", tenant).await.unwrap();

    let doc_key = "doc_canary_1";
    let doc_id = DocId::from_key(doc_key).unwrap();

    let embedding = vec![0.1f32; 768];
    col.insert(doc_key, &embedding, None).await.unwrap();

    // Put KV segment with canary payload in KV store
    let mut canary_payload = vec![0xAA; 32];
    canary_payload.extend_from_slice(CANARY_MARKER);
    canary_payload.extend_from_slice(&[0xBB; 32]);

    let segment_id = u64::try_from(doc_id.inner()).unwrap();
    let segment = KvSegment::new(tenant, segment_id, canary_payload.clone());
    kv_store.insert_segment(tenant, segment);

    // Oracle verification BEFORE delete:
    // 1. Direct query of store
    assert!(
        kv_store.get_segment_bytes(tenant, segment_id).is_some(),
        "Oracle: Segment MUST exist in store before delete"
    );

    // Delete document via facade
    col.delete(doc_key).await.unwrap();

    // Oracle verification AFTER delete:
    // 1. Direct query of store MUST return None
    assert!(
        kv_store.get_segment_bytes(tenant, segment_id).is_none(),
        "Oracle: Segment MUST be removed from store after delete"
    );

    // 2. Scan DB path for canary marker
    let found_canary = search_directory_for_canary(dir.path(), CANARY_MARKER);
    assert!(
        !found_canary,
        "Oracle: Canary marker MUST NOT be present in raw disk files after delete"
    );

    db.close().await.unwrap();
}

#[tokio::test]
async fn test_purge_tenant_purges_tenant1_and_preserves_tenant2() {
    let dir = tempfile::tempdir().unwrap();
    let db = contextra_engine::Contextra::open(dir.path()).await.unwrap();

    let kv_store = Arc::new(TenantIsolatedKvStore::new());
    db.set_kv_hooks(kv_store.clone());

    let tenant1 = TenantId::try_new(201).unwrap();
    let tenant2 = TenantId::try_new(202).unwrap();

    let col1 = db
        .collection_for_tenant("shared_col", tenant1)
        .await
        .unwrap();
    let col2 = db
        .collection_for_tenant("shared_col", tenant2)
        .await
        .unwrap();

    let doc1_key = "doc_t1_1";
    let doc2_key = "doc_t2_1";

    let doc1_id = DocId::from_key(doc1_key).unwrap();
    let doc2_id = DocId::from_key(doc2_key).unwrap();

    let embedding = vec![0.1f32; 768];
    col1.insert(doc1_key, &embedding, None).await.unwrap();
    col2.insert(doc2_key, &embedding, None).await.unwrap();

    let seg1_id = u64::try_from(doc1_id.inner()).unwrap();
    let seg2_id = u64::try_from(doc2_id.inner()).unwrap();

    kv_store.insert_segment(tenant1, KvSegment::new(tenant1, seg1_id, vec![1, 2, 3]));
    kv_store.insert_segment(tenant2, KvSegment::new(tenant2, seg2_id, vec![4, 5, 6]));

    assert!(kv_store.get_segment_bytes(tenant1, seg1_id).is_some());
    assert!(kv_store.get_segment_bytes(tenant2, seg2_id).is_some());

    // Purge tenant1
    db.purge_tenant(tenant1).await.unwrap();

    // Oracle check for Tenant 1:
    assert!(
        kv_store.get_segment_bytes(tenant1, seg1_id).is_none(),
        "Tenant 1 segments MUST be purged"
    );
    assert!(
        col1.get(doc1_key).await.unwrap().is_none(),
        "Tenant 1 document MUST be purged from collection"
    );

    // Oracle check for Tenant 2:
    assert!(
        kv_store.get_segment_bytes(tenant2, seg2_id).is_some(),
        "Tenant 2 segments MUST remain intact"
    );
    assert!(
        col2.get(doc2_key).await.unwrap().is_some(),
        "Tenant 2 document MUST remain readable in collection"
    );

    db.close().await.unwrap();
}
