// FILE-CONTEXT
// STAND: 2026-10-05T00:00:00Z
// ZWECK: Multi-Tenant Key Isolation Property Tests (WP-H)
// INVARIANTEN: INV-TENANT-2: scan_prefix(codec.scan_prefix()) liefert ausschließlich Keys des gewählten Tenants

use std::collections::BTreeMap;
use std::sync::Arc;
use tempfile::TempDir;

use proptest::prelude::*;

use contextra_core::{CollectionId, DocId, TenantId, TxId};
use contextra_ports::StorageEngine;
use contextra_store::lsm::{LsmConfig, LsmStorage};
use contextra_store::tenant_codec::{TenantKeyCodec, TenantScopedStorage};

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    /// (a) Roundtrip-Eigenschaft:
    /// decode_tenant_id(encode_chunk_key(t, col, doc)) == Some(t)
    /// decode_tenant_id(encode_graph_key(t, col, entity)) == Some(t)
    /// decode_tenant_id(key_with_colons_and_nulls) == Some(t)
    /// Orakel: Unabhängiges String-Parsing (Splitten an ':').
    #[test]
    fn prop_roundtrip_tenant_id(
        tenant_raw in 1u64..=u64::MAX,
        col_id in 0u64..=u64::MAX,
        doc_id in 0u64..=u64::MAX,
        entity_id in 0u64..=u64::MAX,
        suffix in r".*[:\x00].*",
    ) {
        let tenant = TenantId::try_new(tenant_raw).expect("valid tenant_id > 0");
        let codec = TenantKeyCodec::new(tenant);
        let col = CollectionId(col_id);
        let doc = DocId(doc_id);

        let chunk_key = codec.encode_chunk_key(&col, doc);
        let graph_key = codec.encode_graph_key(&col, entity_id);

        // Custom key mit Suffix enthaltend ':' und 0x00
        let mut custom_key = codec.scan_prefix().to_vec();
        custom_key.extend_from_slice(suffix.as_bytes());

        // Codec decode
        let decoded_chunk = TenantKeyCodec::decode_tenant_id(&chunk_key);
        let decoded_graph = TenantKeyCodec::decode_tenant_id(&graph_key);
        let decoded_custom = TenantKeyCodec::decode_tenant_id(&custom_key);

        prop_assert_eq!(decoded_chunk, Some(tenant));
        prop_assert_eq!(decoded_graph, Some(tenant));
        prop_assert_eq!(decoded_custom, Some(tenant));

        // Unabhängiges Orakel (Brute-Force String Parsing)
        let chunk_str = std::str::from_utf8(&chunk_key).expect("valid utf8 chunk key");
        let chunk_parts: Vec<&str> = chunk_str.split(':').collect();
        prop_assert!(chunk_parts.len() >= 2);
        prop_assert_eq!(chunk_parts[0], "t");
        let oracle_tenant_id: u64 = chunk_parts[1].parse().expect("parsed tenant u64");
        prop_assert_eq!(oracle_tenant_id, tenant.inner());

        let graph_str = std::str::from_utf8(&graph_key).expect("valid utf8 graph key");
        let graph_parts: Vec<&str> = graph_str.split(':').collect();
        prop_assert!(graph_parts.len() >= 2);
        prop_assert_eq!(graph_parts[0], "t");
        let oracle_graph_tenant_id: u64 = graph_parts[1].parse().expect("parsed tenant u64");
        prop_assert_eq!(oracle_graph_tenant_id, tenant.inner());
    }

    /// (b) Präfix-Trennung:
    /// Für t1 != t2 ist der Präfix von t1 NIE Präfix eines Schlüssels oder Collection-Präfix von t2.
    /// Explizit getestet: t1 vs t2 für beliebige t1 != t2 (inkl. Fall 1 vs 10 vs 100).
    #[test]
    fn prop_prefix_isolation(
        t1_raw in 1u64..=1_000_000u64,
        t2_raw in 1u64..=1_000_000u64,
        col1_id in 0u64..=10_000u64,
        col2_id in 0u64..=10_000u64,
        doc2_id in 0u64..=10_000u64,
        entity2_id in 0u64..=10_000u64,
    ) {
        prop_assume!(t1_raw != t2_raw);

        let tenant1 = TenantId::try_new(t1_raw).unwrap();
        let tenant2 = TenantId::try_new(t2_raw).unwrap();

        let codec1 = TenantKeyCodec::new(tenant1);
        let codec2 = TenantKeyCodec::new(tenant2);

        let col1 = CollectionId(col1_id);
        let col2 = CollectionId(col2_id);
        let doc2 = DocId(doc2_id);

        let prefix1 = codec1.scan_prefix();
        let col_prefix1 = codec1.collection_prefix(&col1);

        let prefix2 = codec2.scan_prefix();
        let col_prefix2 = codec2.collection_prefix(&col2);
        let key2_chunk = codec2.encode_chunk_key(&col2, doc2);
        let key2_graph = codec2.encode_graph_key(&col2, entity2_id);

        // Orakel: Byte-Slice-Präfix-Bedingung !key2.starts_with(prefix1)
        prop_assert!(!prefix2.starts_with(prefix1));
        prop_assert!(!col_prefix2.starts_with(prefix1));
        prop_assert!(!key2_chunk.starts_with(prefix1));
        prop_assert!(!key2_graph.starts_with(prefix1));

        prop_assert!(!col_prefix2.starts_with(&col_prefix1));
    }
}

/// Explicit test for Tenant ID 0 reservation and decode rejection
#[test]
fn test_explicit_tenant_id_zero_and_edge_keys() {
    assert!(TenantId::try_new(0).is_err());
    assert_eq!(TenantKeyCodec::decode_tenant_id(b"t:0:col:chunk:1"), None);

    let tenant = TenantId::try_new(42).unwrap();
    let codec = TenantKeyCodec::new(tenant);

    // Key mit ':' und 0x00 im Suffix
    let key_with_delims = [codec.scan_prefix(), b"col:123:\x00:sub_key"].concat();
    assert_eq!(
        TenantKeyCodec::decode_tenant_id(&key_with_delims),
        Some(tenant)
    );
}

/// Spezifischer Test für den kritischen Fall 1 vs 10 vs 100
#[test]
fn test_explicit_numeric_prefix_overlap() {
    let t1 = TenantId::try_new(1).unwrap();
    let t10 = TenantId::try_new(10).unwrap();
    let t100 = TenantId::try_new(100).unwrap();

    let c1 = TenantKeyCodec::new(t1);
    let c10 = TenantKeyCodec::new(t10);
    let c100 = TenantKeyCodec::new(t100);

    let col = CollectionId(5);
    let doc = DocId(42);

    let k1 = c1.encode_chunk_key(&col, doc);
    let k10 = c10.encode_chunk_key(&col, doc);
    let k100 = c100.encode_chunk_key(&col, doc);

    // t:1: ist KEIN Präfix von t:10: oder t:100: wegen des schließenden Doppelpunkts
    assert!(!k10.starts_with(c1.scan_prefix()));
    assert!(!k100.starts_with(c1.scan_prefix()));
    assert!(!k100.starts_with(c10.scan_prefix()));

    assert!(k1.starts_with(c1.scan_prefix()));
    assert_eq!(c1.scan_prefix(), b"t:1:");
    assert_eq!(c10.scan_prefix(), b"t:10:");
    assert_eq!(c100.scan_prefix(), b"t:100:");
}

/// (c) Scan-Isolation mit echten `LsmStorage` und `TenantScopedStorage` Instanzen.
/// Orakel: Zwei unabhängige `BTreeMap<Vec<u8>, Vec<u8>>` pro Mandant.
#[tokio::test]
async fn test_scan_isolation_with_lsm_storage_and_oracle() {
    let dir = TempDir::new().expect("tempdir created");
    let mut config = LsmConfig::default();
    config.path = dir.path().to_path_buf();

    let lsm = Arc::new(LsmStorage::open(config).await.expect("lsm storage open"));

    let tenant_a = TenantId::try_new(10).unwrap();
    let tenant_b = TenantId::try_new(100).unwrap();

    let store_a = TenantScopedStorage::new(Arc::clone(&lsm), tenant_a);
    let store_b = TenantScopedStorage::new(Arc::clone(&lsm), tenant_b);

    let mut oracle_a: BTreeMap<Vec<u8>, Vec<u8>> = BTreeMap::new();
    let mut oracle_b: BTreeMap<Vec<u8>, Vec<u8>> = BTreeMap::new();

    let tx_id = TxId::new(1);

    // Ähnliche/überlappende Schlüsselnamen in A und B schreiben
    let keys_a = vec![
        (b"user:1".to_vec(), b"alice".to_vec()),
        (b"user:2".to_vec(), b"bob".to_vec()),
        (b"config:timeout".to_vec(), b"30s".to_vec()),
    ];

    let keys_b = vec![
        (b"user:1".to_vec(), b"eve".to_vec()),
        (b"user:2".to_vec(), b"mallory".to_vec()),
        (b"config:timeout".to_vec(), b"60s".to_vec()),
        (b"data:blob".to_vec(), b"secret_b".to_vec()),
    ];

    for (k, v) in &keys_a {
        store_a.put(tx_id, k, v).await.unwrap();
        oracle_a.insert(k.clone(), v.clone());
    }

    for (k, v) in &keys_b {
        store_b.put(tx_id, k, v).await.unwrap();
        oracle_b.insert(k.clone(), v.clone());
    }

    store_a.commit(tx_id).await.unwrap();

    // 1. scan_prefix Isolation
    let scanned_a = store_a.scan_prefix(b"user:").await.unwrap();
    let expected_scanned_a: Vec<(Vec<u8>, Vec<u8>)> = oracle_a
        .iter()
        .filter(|(k, _)| k.starts_with(b"user:"))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    assert_eq!(scanned_a, expected_scanned_a);

    let scanned_b = store_b.scan_prefix(b"user:").await.unwrap();
    let expected_scanned_b: Vec<(Vec<u8>, Vec<u8>)> = oracle_b
        .iter()
        .filter(|(k, _)| k.starts_with(b"user:"))
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    assert_eq!(scanned_b, expected_scanned_b);

    // 2. Unbeschränkter scan (Range scan) Isolation
    let all_a = store_a
        .scan(std::ops::Bound::Unbounded, std::ops::Bound::Unbounded, None)
        .await
        .unwrap();
    let expected_all_a: Vec<(Vec<u8>, Vec<u8>)> = oracle_a
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    assert_eq!(all_a, expected_all_a);

    let all_b = store_b
        .scan(std::ops::Bound::Unbounded, std::ops::Bound::Unbounded, None)
        .await
        .unwrap();
    let expected_all_b: Vec<(Vec<u8>, Vec<u8>)> = oracle_b
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    assert_eq!(all_b, expected_all_b);

    // 3. delete_prefix Isolation
    let del_tx = TxId::new(2);
    let deleted_a_count = store_a.delete_prefix(del_tx, b"user:").await.unwrap();
    store_a.commit(del_tx).await.unwrap();

    // Orakel-Update für Tenant A
    oracle_a.retain(|k, _| !k.starts_with(b"user:"));

    assert_eq!(deleted_a_count, 2);

    // Prüfung nach Löschung
    let post_del_a = store_a
        .scan(std::ops::Bound::Unbounded, std::ops::Bound::Unbounded, None)
        .await
        .unwrap();
    let expected_post_del_a: Vec<(Vec<u8>, Vec<u8>)> = oracle_a
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    assert_eq!(post_del_a, expected_post_del_a);

    // WICHTIG: Tenant B muss vollkommen unberührt sein
    let post_del_b = store_b
        .scan(std::ops::Bound::Unbounded, std::ops::Bound::Unbounded, None)
        .await
        .unwrap();
    let expected_post_del_b: Vec<(Vec<u8>, Vec<u8>)> = oracle_b
        .iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    assert_eq!(post_del_b, expected_post_del_b);
}
