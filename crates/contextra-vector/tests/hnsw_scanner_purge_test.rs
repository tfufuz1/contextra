// FILE-CONTEXT
// ZWECK: Tests für fail-closed HNSW Hard-Purge und LayerScanner (Welle 2/3).
// INVARIANTEN: Tombstone-only meldet residual > 0; nach Hard-Purge residual == 0; Byte-Needle Test.
// HOTSPOTS: tests/hnsw_scanner_purge_test.rs

use std::sync::Arc;
use tempfile::tempdir;

use contextra_ports::clock::SystemClock;
use contextra_ports::scanner::{ScanTarget, ScanTechnique};
use contextra_ports::VectorIndex;
use contextra_types::{DocId, RequestContext, TenantId, TxId};
use contextra_vector::hnsw::{hnsw_purger, hnsw_scanner, HnswConfigBuilder, HnswIndex};

#[tokio::test]
async fn test_tombstone_vs_hard_purge() {
    let clock = SystemClock::new();
    let config = HnswConfigBuilder::new(4)
        .m(8)
        .ef_construction(16)
        .ef_search(16)
        .rebuild_threshold(0.0)
        .build()
        .expect("Config build");

    let index = Arc::new(HnswIndex::try_new(config).expect("Index init"));
    let tx1 = TxId(100);
    let doc1 = DocId::new(42);
    let doc2 = DocId::new(43);

    index
        .insert(tx1, doc1, &[1.0, 0.0, 0.0, 0.0])
        .await
        .unwrap();
    index
        .insert(tx1, doc2, &[0.0, 1.0, 0.0, 0.0])
        .await
        .unwrap();
    index.commit(tx1).await.unwrap();
    // 1. Nur Tombstone setzen
    let tx2 = TxId(101);
    index.delete(tx2, doc1).await.unwrap();
    index.commit(tx2).await.unwrap();

    let ctx = RequestContext::new(TenantId::SYSTEM, TxId(200));
    let target = ScanTarget::new("42", blake3::hash(b"42").into());

    let scanner = hnsw_scanner(index.clone());
    let witness_before = scanner
        .scan(
            &ctx,
            target.clone(),
            ScanTechnique::IndexIntrospection,
            &clock,
        )
        .expect("Scan before purge");

    assert!(
        witness_before.remaining() > 0,
        "Tombstone alone must leave live residual > 0 (found {})",
        witness_before.remaining()
    );

    // 2. Hard-Purge ausführen
    let purger = hnsw_purger(index.clone());
    let stats = purger.purge(&ctx, &target).expect("Hard purge");
    assert_eq!(stats.purged_units, 1);
    assert!(stats.bytes_freed > 0);

    // 3. Zweiter Scan nach Hard-Purge
    let witness_after = scanner
        .scan(&ctx, target, ScanTechnique::IndexIntrospection, &clock)
        .expect("Scan after purge");

    assert_eq!(
        witness_after.remaining(),
        0,
        "After hard-purge, remaining live residual must be 0"
    );
}

#[tokio::test]
async fn test_hard_purge_recall_and_search_safety() {
    let config = HnswConfigBuilder::new(4)
        .m(8)
        .ef_construction(16)
        .ef_search(16)
        .build()
        .expect("Config build");

    let index = Arc::new(HnswIndex::try_new(config).expect("Index init"));
    let tx = TxId(100);

    for i in 1..=10 {
        let doc = DocId::new(i);
        let val = i as f32;
        index
            .insert(tx, doc, &[val, val * 0.1, 0.0, 0.0])
            .await
            .unwrap();
    }
    index.commit(tx).await.unwrap();

    let ctx = RequestContext::new(TenantId::SYSTEM, TxId(200));
    let target_doc5 = ScanTarget::new("5", blake3::hash(b"5").into());

    let purger = hnsw_purger(index.clone());
    purger.purge(&ctx, &target_doc5).expect("Purge doc 5");

    // Search query around doc 5
    let results = index
        .search(&[5.0, 0.5, 0.0, 0.0], 5)
        .await
        .expect("Search");

    for doc in &results {
        assert_ne!(
            doc.doc_id,
            DocId::new(5),
            "Purged doc 5 must not be found in search"
        );
    }

    assert!(
        !results.is_empty(),
        "Remaining documents must still be returned by search"
    );
}

#[tokio::test]
async fn test_persisted_file_byte_needle() {
    let dir = tempdir().expect("tempdir");
    let file_path = dir.path().join("test_index.hnsw");

    let config = HnswConfigBuilder::new(4)
        .m(8)
        .ef_construction(16)
        .ef_search(16)
        .build()
        .expect("Config build");

    let index = Arc::new(HnswIndex::try_new(config).expect("Index init"));
    let tx = TxId(100);
    let needle_doc = DocId::new(888_888);

    // Vector with distinct byte signature
    let needle_vec = vec![123.456f32, 789.123, 456.789, 321.654];
    index.insert(tx, needle_doc, &needle_vec).await.unwrap();
    index.commit(tx).await.unwrap();

    index.save(&file_path).await.expect("Save index");
    index.load_mmap(&file_path).await.expect("Load mmap");

    // Verify raw file contains needle bytes before purge
    let file_bytes_before = std::fs::read(&file_path).expect("Read saved file");
    let target_bytes = 888_888u64.to_le_bytes();
    let contains_doc_before = file_bytes_before
        .windows(target_bytes.len())
        .any(|w| w == target_bytes);
    assert!(
        contains_doc_before,
        "Saved file must contain needle doc_id bytes initially"
    );

    // Hard-purge needle doc
    let ctx = RequestContext::new(TenantId::SYSTEM, TxId(200));
    let target = ScanTarget::new("888888", blake3::hash(b"888888").into());
    let purger = hnsw_purger(index.clone());
    purger.purge(&ctx, &target).expect("Purge needle doc");

    // Re-save clean index
    index.save(&file_path).await.expect("Re-save clean index");

    let file_bytes_after = std::fs::read(&file_path).expect("Read re-saved file");
    let contains_doc_after = file_bytes_after
        .windows(target_bytes.len())
        .any(|w| w == target_bytes);
    assert!(
        !contains_doc_after,
        "Re-saved file after hard-purge must NOT contain needle doc_id bytes"
    );
}

#[tokio::test]
async fn test_hard_purge_entry_point_and_last_node() {
    let config = HnswConfigBuilder::new(4)
        .m(8)
        .ef_construction(16)
        .ef_search(16)
        .build()
        .expect("Config build");

    let index = Arc::new(HnswIndex::try_new(config).expect("Index init"));
    let tx = TxId(100);

    let doc1 = DocId::new(1);
    let doc2 = DocId::new(2);

    index.insert(tx, doc1, &[1.0, 0.0, 0.0, 0.0]).await.unwrap();
    index.insert(tx, doc2, &[0.0, 1.0, 0.0, 0.0]).await.unwrap();
    index.commit(tx).await.unwrap();

    let ctx = RequestContext::new(TenantId::SYSTEM, TxId(200));

    // Hard purge doc1
    let target1 = ScanTarget::new("1", blake3::hash(b"1").into());
    let purger = hnsw_purger(index.clone());
    purger.purge(&ctx, &target1).expect("Purge doc 1");

    // Search should find doc2
    let search1 = index
        .search(&[0.0, 1.0, 0.0, 0.0], 5)
        .await
        .expect("Search after purging ep");
    assert_eq!(search1.len(), 1);
    assert_eq!(search1[0].doc_id, doc2);

    // Hard purge doc2 (last node)
    let target2 = ScanTarget::new("2", blake3::hash(b"2").into());
    purger.purge(&ctx, &target2).expect("Purge doc 2");

    // Search on empty index
    let search2 = index
        .search(&[0.0, 1.0, 0.0, 0.0], 5)
        .await
        .expect("Search on empty index");
    assert!(
        search2.is_empty(),
        "Search on empty index must return 0 results safely"
    );
}

#[tokio::test]
async fn test_sabotaged_purge_detected_by_scanner() {
    let clock = SystemClock::new();
    let config = HnswConfigBuilder::new(4)
        .m(8)
        .ef_construction(16)
        .ef_search(16)
        .rebuild_threshold(0.0)
        .build()
        .expect("Config build");

    let index = Arc::new(HnswIndex::try_new(config).expect("Index init"));
    let tx1 = TxId(100);
    let doc1 = DocId::new(77);

    index
        .insert(tx1, doc1, &[1.0, 2.0, 3.0, 4.0])
        .await
        .unwrap();
    index.commit(tx1).await.unwrap();

    // Sabotaged "deletion": mark tombstone only without zeroizing vector memory or removing slot
    let tx2 = TxId(101);
    index.delete(tx2, doc1).await.unwrap();
    index.commit(tx2).await.unwrap();

    let ctx = RequestContext::new(TenantId::SYSTEM, TxId(200));
    let target = ScanTarget::new("77", blake3::hash(b"77").into());

    let scanner = hnsw_scanner(index.clone());
    let witness = scanner
        .scan(&ctx, target, ScanTechnique::IndexIntrospection, &clock)
        .expect("Scan sabotaged index");

    assert!(
        witness.remaining() > 0,
        "Scanner MUST detect live residual when purge is sabotaged (un-zeroized tombstoned node)"
    );
}
