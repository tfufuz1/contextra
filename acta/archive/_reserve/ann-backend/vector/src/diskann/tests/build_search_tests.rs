// FILE-CONTEXT
// ZWECK: Modularisierte Testsuiten für DiskANN (WP-VEC-07).
// STAND: 2026-11-11

use super::super::config::{DiskAnnConfig, DiskAnnFallbackPolicy};
use super::super::format::{
    compute_adaptive_flush_threshold, DiskAnnFooter, DiskAnnHeader, DISKANN_FOOTER_MAGIC,
    DISKANN_INTEGRITY_KEY, DISKANN_MAGIC, DISKANN_VERSION,
};
use super::super::types::{DiskAnnIndex, SearchCandidate};
use crate::error::{Result, VectorError};
use crate::types::{DistanceMetric, DocId, TxId};
use tempfile::tempdir;
#[tokio::test]
async fn test_diskann_build_search_roundtrip() -> Result<()> {
    let dir = tempfile::tempdir().map_err(VectorError::Io)?;
    let config = DiskAnnConfig {
        index_path: dir.path().join("smoke.diskann"),
        dimension: 4,
        max_degree: 8,
        beam_width: 8,
        distance_metric: DistanceMetric::Euclidean,
        quantize: true,
        ..DiskAnnConfig::default()
    };
    let index = DiskAnnIndex::try_new(config)?;
    let vectors: Vec<Vec<f32>> = (0..10).map(|i| vec![i as f32, 0.0, 0.0, 0.0]).collect();
    let ids: Vec<DocId> = (0..10u64).map(DocId::from).collect();
    index.build(&vectors, &ids).await?;
    let query = vec![9.0f32, 0.0, 0.0, 0.0];
    let results = index.search(&query, 1).await?;
    assert!(
        !results.is_empty(),
        "Smoke-Test: Suchergebnis darf nicht leer sein"
    );
    assert_eq!(
        results[0].doc_id,
        DocId::from(9u64),
        "Nächster Nachbar zu [9,0,0,0] muss id=9 sein"
    );
    Ok(())
}

#[tokio::test]
async fn test_diskann_exact_file_sizes_no_panic() {
    // BEFUND 3: Dateigrößen 0, 1, 10, 39, 40 Bytes
    for size in [0, 1, 10, 39, 40] {
        let temp_dir = tempfile::tempdir().unwrap();
        let index_path = temp_dir.path().join(format!("diskann_{}.idx", size));
        let dummy_data = vec![0x55u8; size];
        tokio::fs::write(&index_path, &dummy_data).await.unwrap();

        let config = DiskAnnConfig {
            index_path,
            dimension: 8,
            max_degree: 4,
            sector_size: 4096,
            fallback_policy: DiskAnnFallbackPolicy::FailFast,
            ..DiskAnnConfig::default()
        };
        let index = DiskAnnIndex::try_new(config).unwrap();

        let spawn_res = tokio::spawn(async move { index.load().await }).await;
        assert!(
            spawn_res.is_ok(),
            "DiskAnnIndex::load panicked on file size {}!",
            size
        );

        let load_res = spawn_res.unwrap();
        if size < DiskAnnHeader::SIZE {
            assert!(
                load_res.is_err(),
                "Expected Result::Err for DiskANN file size {} < {}",
                size,
                DiskAnnHeader::SIZE
            );
        }
    }
}

#[tokio::test]
async fn test_diskann_corrupt_offset_out_of_bounds_no_panic() {
    // BEFUND 3: Valid Header (size >= 40) but corrupt node_count / offset causing offset + 8 out of bounds
    let temp_dir = tempfile::tempdir().unwrap();
    let index_path = temp_dir.path().join("corrupt_offset.idx");

    let config = DiskAnnConfig {
        index_path: index_path.clone(),
        dimension: 8,
        max_degree: 4,
        sector_size: 4096,
        distance_metric: DistanceMetric::Euclidean,
        ..DiskAnnConfig::default()
    };

    let index = DiskAnnIndex::try_new(config.clone()).unwrap();
    let vectors = vec![vec![1.0; 8]];
    let ids = vec![DocId::from(1u64)];
    index.build(&vectors, &ids).await.unwrap();

    // Mutate node_count in header to 10,000 without expanding file size -> offset out of bounds
    let mut data = tokio::fs::read(&index_path).await.unwrap();
    let corrupt_count: u64 = 10_000;
    data[6..14].copy_from_slice(&corrupt_count.to_le_bytes());
    tokio::fs::write(&index_path, &data).await.unwrap();

    let reloaded_config = DiskAnnConfig {
        fallback_policy: DiskAnnFallbackPolicy::FailFast,
        ..config
    };
    let reloaded = DiskAnnIndex::try_new(reloaded_config).unwrap();

    let reloaded_clone = reloaded.clone();
    let spawn_res = tokio::spawn(async move { reloaded_clone.load().await }).await;
    assert!(
        spawn_res.is_ok(),
        "DiskAnnIndex::load panicked on corrupt node_count/offset!"
    );

    let load_res = spawn_res.unwrap();
    assert!(
        load_res.is_err(),
        "Expected Result::Err on corrupt offset beyond file size!"
    );

    // Also test load_node directly on reloaded index with truncated/corrupt file
    let catch_node =
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| reloaded.load_node(9999)));
    assert!(
        catch_node.is_ok(),
        "load_node panicked on out of bounds node index!"
    );
    assert!(catch_node.unwrap().is_err());
}

#[tokio::test]
async fn test_diskann_loaded_doc_ids_match_original() -> Result<()> {
    let dir = tempfile::tempdir().map_err(VectorError::Io)?;
    let config = DiskAnnConfig {
        index_path: dir.path().join("doc_ids.diskann"),
        dimension: 4,
        max_degree: 4,
        sector_size: 4096,
        distance_metric: DistanceMetric::Euclidean,
        quantize: false,
        ..DiskAnnConfig::default()
    };
    let index = DiskAnnIndex::try_new(config.clone())?;
    let vectors: Vec<Vec<f32>> = (0..5).map(|i| vec![i as f32, 1.0, 2.0, 3.0]).collect();
    let expected_ids: Vec<DocId> = vec![
        DocId::from(1001u64),
        DocId::from(1002u64),
        DocId::from(1003u64),
        DocId::from(1004u64),
        DocId::from(1005u64),
    ];
    index.build(&vectors, &expected_ids).await?;

    // Verify loaded doc_ids in fresh index instance match original IDs
    let reloaded = DiskAnnIndex::try_new(config)?;
    reloaded.load().await?;
    let loaded_ids = reloaded.inner.doc_ids.read().clone();
    assert_eq!(
        loaded_ids, expected_ids,
        "Loaded doc_ids must exactly match original doc_ids (regression for D-2.1 offset bug)"
    );
    Ok(())
}

#[tokio::test]
async fn test_diskann_incremental_insert_connectivity_and_searchability() -> Result<()> {
    let temp_dir = tempfile::tempdir().map_err(VectorError::Io)?;
    let index_path = temp_dir.path().join("connectivity.idx");

    let config = DiskAnnConfig {
        index_path: index_path.clone(),
        dimension: 16,
        max_degree: 8,
        beam_width: 16,
        distance_metric: DistanceMetric::Euclidean,
        ..DiskAnnConfig::default()
    };

    let index = DiskAnnIndex::try_new(config)?;

    // 1. Build initial index with 100 vectors
    let base_n = 100;
    let mut base_vecs = Vec::with_capacity(base_n);
    let mut base_ids = Vec::with_capacity(base_n);
    for i in 0..base_n {
        let mut v = vec![0.0f32; 16];
        v[0] = i as f32;
        base_vecs.push(v);
        base_ids.push(DocId::from(i as u64 + 1));
    }
    index.build(&base_vecs, &base_ids).await?;

    // 2. Incrementally insert 5 new vectors (pending_ratio = 5/105 < 0.10, so incremental path is triggered)
    let new_n = 5;
    let mut new_ids = Vec::with_capacity(new_n);
    for i in 0..new_n {
        let id = DocId::from((base_n + i + 1) as u64);
        let mut v = vec![0.0f32; 16];
        v[0] = (base_n + i) as f32 + 0.5;
        index.insert(TxId(1), id, &v).await?;
        new_ids.push((id, v));
    }

    index.persist_delta().await?;

    // 3. Verify total node count
    assert_eq!(index.len().await, base_n + new_n);

    // 4. Verify graph connectivity: newly inserted nodes must be in the graph and reachable
    let ep = {
        let header = index.inner.header.read().unwrap();
        header.entry_point
    };

    // BFS / Greedy reachability check from entry_point
    let mut visited = HashSet::new();
    let mut queue = std::collections::VecDeque::new();
    queue.push_back(ep);
    visited.insert(ep);

    while let Some(curr) = queue.pop_front() {
        let node = index.load_node(curr)?;
        for &nbr in &node.neighbors {
            if visited.insert(nbr) {
                queue.push_back(nbr);
            }
        }
    }

    for idx in 0..(base_n + new_n) as u32 {
        assert!(
            visited.contains(&idx),
            "Node {} (newly inserted or base) is not reachable from entry_point {}!",
            idx,
            ep
        );
    }

    // 5. Verify searchability of all new vectors
    for (id, vec) in &new_ids {
        let res = index.search(vec, 1).await?;
        assert!(!res.is_empty());
        assert_eq!(
            res[0].doc_id, *id,
            "Newly inserted doc_id {:?} should be top search result",
            id
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_diskann_pending_wal_recovery() -> Result<()> {
    let temp_dir = tempfile::tempdir().map_err(VectorError::Io)?;
    let index_path = temp_dir.path().join("wal_recovery.idx");

    let config = DiskAnnConfig {
        index_path: index_path.clone(),
        dimension: 4,
        max_degree: 4,
        beam_width: 4,
        distance_metric: DistanceMetric::Euclidean,
        fallback_policy: DiskAnnFallbackPolicy::FailFast,
        ..DiskAnnConfig::default()
    };

    // 1. Erstelle Index und baue Basis mit 5 Vektoren
    let index = DiskAnnIndex::try_new(config.clone())?;
    let base_vecs = vec![
        vec![1.0, 0.0, 0.0, 0.0],
        vec![2.0, 0.0, 0.0, 0.0],
        vec![3.0, 0.0, 0.0, 0.0],
        vec![4.0, 0.0, 0.0, 0.0],
        vec![5.0, 0.0, 0.0, 0.0],
    ];
    let base_ids: Vec<DocId> = (1..=5u64).map(DocId::from).collect();
    index.build(&base_vecs, &base_ids).await?;

    // 2. Füge 3 Vektoren ein (unterhalb von PENDING_FLUSH_THRESHOLD=50) -> WAL wird geschrieben, persist_delta NICHT aufgerufen
    let uncommitted_doc_id = DocId::from(999u64);
    let uncommitted_vec = vec![99.0, 0.0, 0.0, 0.0];
    index
        .insert(TxId(1), uncommitted_doc_id, &uncommitted_vec)
        .await?;

    // Prüfe, dass pending.wal existiert
    let pending_wal = index_path.with_extension("pending.wal");
    assert!(
        pending_wal.exists(),
        "pending.wal muss nach insert() auf Disk existieren"
    );

    // 3. Simuliere Absturz: Verwürfe die Index-Instanz ohne persist_delta() aufzurufen
    drop(index);

    // 4. Erstelle neue Index-Instanz und rufe load() auf
    let reloaded_index = DiskAnnIndex::try_new(config)?;
    reloaded_index.load().await?;

    // 5. Verifiziere, dass recover_pending_delta() gelaufen ist und der Vektor auffindbar ist
    assert_eq!(
        reloaded_index.len().await,
        6,
        "Der wiederhergestellte Vektor muss im Index enthalten sein"
    );
    let results = reloaded_index.search(&uncommitted_vec, 1).await?;
    assert!(!results.is_empty());
    assert_eq!(
        results[0].doc_id, uncommitted_doc_id,
        "Der wiederhergestellte Vektor muss per Suche auffindbar sein"
    );

    // Verifiziere, dass pending.wal nach verarbeiteter Recovery gelöscht wurde
    assert!(
        !pending_wal.exists(),
        "pending.wal muss nach erfolgreicher Recovery gelöscht sein"
    );

    Ok(())
}
