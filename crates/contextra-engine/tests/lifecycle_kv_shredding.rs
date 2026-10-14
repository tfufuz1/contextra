// FILE-CONTEXT
// ZWECK: KV Cache Crypto-Shredding integration test via engine open & deletion paths.

use contextra_crypto::kv_shredding::KeyRegistry;
use contextra_engine::{Contextra, ContextraConfig};
use contextra_kvcache::segment::{ShreddableSegmentKey, Tier2EncryptedSegment};
use contextra_kvcache::{KvShreddingWiring, KvShreddingWiringDeps};
use contextra_types::{DocId, TenantId};
use std::sync::Arc;
use tempfile::tempdir;

#[tokio::test]
async fn test_kv_cache_crypto_shredding_and_tenant_isolation() -> contextra_types::Result<()> {
    let tmp = tempdir().unwrap();
    let config = ContextraConfig {
        consolidation_enabled: false,
        ..Default::default()
    };

    let db = Contextra::open_with_config(tmp.path(), config).await?;

    let key_registry = Arc::new(KeyRegistry::new());
    let shredding_wiring = Arc::new(
        KvShreddingWiring::build(KvShreddingWiringDeps {
            key_registry: Arc::clone(&key_registry),
        })
        .unwrap(),
    );

    let tenant1 = TenantId::try_new(101).unwrap();
    let tenant2 = TenantId::try_new(102).unwrap();
    let doc_id = DocId(500);

    let key1 = ShreddableSegmentKey::try_new_random("pass-tenant-1").unwrap();
    let key2 = ShreddableSegmentKey::try_new_random("pass-tenant-2").unwrap();

    let segment1 = Tier2EncryptedSegment::new(tenant1, 500, b"tenant 1 secret payload", key1.clone()).unwrap();
    let segment2 = Tier2EncryptedSegment::new(tenant2, 500, b"tenant 2 secret payload", key2.clone()).unwrap();

    shredding_wiring.register_segment_key(tenant1, 500, key1);
    shredding_wiring.register_segment_key(tenant2, 500, key2);

    assert_eq!(segment1.read_and_decrypt().unwrap(), b"tenant 1 secret payload");
    assert_eq!(segment2.read_and_decrypt().unwrap(), b"tenant 2 secret payload");

    // Shred tenant1's key
    let adapter = db.kv_hooks().expect("kv_hooks configured on Contextra");
    adapter.remove_doc_segments(tenant1, doc_id);

    // Tenant 1 segment is no longer decryptable
    assert!(segment1.read_and_decrypt().is_err());
    assert!(shredding_wiring.is_shredded(tenant1, 500));

    // Tenant 2 remains untouched and readable (INV-KDF-1)
    assert_eq!(segment2.read_and_decrypt().unwrap(), b"tenant 2 secret payload");
    assert!(!shredding_wiring.is_shredded(tenant2, 500));

    db.close().await?;
    Ok(())
}
