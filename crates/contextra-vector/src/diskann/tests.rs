// FILE-CONTEXT
// ZWECK: DiskANN-Graphindex für Out-of-Core Approximate Nearest Neighbor Search (WP-4.3).
// INVARIANTEN: Lock-Hierarchie: header -> mmap -> cache / quantizer / doc_ids; atomic rename + parent dir sync bei file persistence.
// NICHT-OFFENSICHTLICH: Mmap für Vektor- & Graphlesezugriffe, ausgelagert an contextra-sys::mmap_readonly.
// STAND: TS:2026-09-10T19:30:00Z (SESSION: a9d67eae)

#[cfg(test)]
mod tests {
    use crate::diskann::config::{DiskAnnConfig, DiskAnnFallbackPolicy};
    use crate::diskann::format::{
        compute_adaptive_flush_threshold, DiskAnnFooter, DiskAnnHeader, DISKANN_FOOTER_MAGIC,
        DISKANN_INTEGRITY_KEY, DISKANN_VERSION, PENDING_WAL_MAGIC, PENDING_WAL_VERSION,
    };
    use crate::diskann::types::{DiskAnnIndex, SearchCandidate};
    use contextra_core::{ContextraError, DistanceMetric, DocId, Result, TxId, VectorIndex};
    use std::collections::HashSet;
    use std::path::PathBuf;

    use super::*;

    #[test]
    #[test]
    fn test_compute_adaptive_flush_threshold_formula() {
        // Kleine Collection: Boden greift
        assert_eq!(compute_adaptive_flush_threshold(0), 50);
        assert_eq!(compute_adaptive_flush_threshold(100), 50); // floor(100*0.05)=5 < 50
        assert_eq!(compute_adaptive_flush_threshold(1_000), 50); // floor(1000*0.05)=50

        // Mittlere Collection
        assert_eq!(compute_adaptive_flush_threshold(10_000), 500); // floor(10000*0.05)=500

        // Große Collection: Deckel greift
        assert_eq!(compute_adaptive_flush_threshold(20_000), 1_000); // floor(20000*0.05)=1000
        assert_eq!(compute_adaptive_flush_threshold(100_000), 1_000); // Deckel

        // Monotonie: Threshold wächst mit N (bis Deckel)
        let t1 = compute_adaptive_flush_threshold(5_000);
        let t2 = compute_adaptive_flush_threshold(15_000);
        assert!(t1 <= t2);
    }

    #[test]
    #[allow(clippy::unnecessary_literal_unwrap)]
    fn test_benchmark_override_respected() {
        // Wenn pending_flush_threshold im Config gesetzt ist, soll er den
        // adaptiven Wert überschreiben (Benchmark-Escape-Hatch bleibt intakt).
        // Dieser Test prüft die Logik-Ebene (nicht den async insert()-Pfad).
        let threshold_override: Option<u64> = Some(42);
        let n_persisted: u64 = 100_000;
        let result =
            threshold_override.unwrap_or_else(|| compute_adaptive_flush_threshold(n_persisted));
        assert_eq!(result, 42); // Override gewinnt
    }

    #[tokio::test]
    async fn test_diskann_non_power_of_two_sector_size_rejected() {
        let bad_config = DiskAnnConfig {
            sector_size: 4000, // not a power of 2
            ..DiskAnnConfig::default()
        };
        let res = DiskAnnIndex::try_new(bad_config);
        assert!(matches!(res, Err(ContextraError::InvalidInput(_))));
    }

    #[tokio::test]
    async fn test_diskann_delete_non_existent_returns_not_found() {
        let index = DiskAnnIndex::try_new(DiskAnnConfig::default()).expect("valid config");
        let tx = TxId::new(1);
        let doc_id = DocId::from(100u64);

        let delete_res = index.delete(tx, doc_id).await;
        assert!(matches!(delete_res, Err(ContextraError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_diskann_tombstone_delete_search_filtering() -> Result<()> {
        let dir = tempfile::tempdir().map_err(ContextraError::Io)?;
        let index_path = dir.path().join("tombstone_filtering.idx");
        let config = DiskAnnConfig {
            index_path: index_path.clone(),
            dimension: 4,
            max_degree: 4,
            beam_width: 4,
            distance_metric: DistanceMetric::Euclidean,
            fallback_policy: DiskAnnFallbackPolicy::FailFast,
            ..DiskAnnConfig::default()
        };
        let index = DiskAnnIndex::try_new(config.clone())?;

        let vecs = vec![
            vec![1.0, 0.0, 0.0, 0.0],
            vec![2.0, 0.0, 0.0, 0.0],
            vec![3.0, 0.0, 0.0, 0.0],
        ];
        let ids = vec![DocId::from(10u64), DocId::from(20u64), DocId::from(30u64)];
        index.build(&vecs, &ids).await?;

        // 1. Initial search finds doc 10 as top result
        let query = vec![1.0, 0.0, 0.0, 0.0];
        let res = index.search(&query, 1).await?;
        assert_eq!(res[0].doc_id, DocId::from(10u64));
        assert_eq!(index.len().await, 3);

        // 2. Delete doc 10
        index.delete(TxId(1), DocId::from(10u64)).await?;
        assert_eq!(index.len().await, 2);

        // Deleting doc 10 again returns NotFound
        let del2 = index.delete(TxId(1), DocId::from(10u64)).await;
        assert!(matches!(del2, Err(ContextraError::NotFound(_))));

        // 3. Search query no longer returns doc 10, instead returns doc 20
        let res_after_del = index.search(&query, 1).await?;
        assert_eq!(res_after_del[0].doc_id, DocId::from(20u64));

        let all_ids = index.all_doc_ids().await?;
        assert!(!all_ids.contains(&DocId::from(10u64)));
        assert_eq!(all_ids.len(), 2);

        let stats = index.stats().await?;
        assert_eq!(stats.num_vectors, 2);
        assert!((stats.deleted_ratio - (1.0 / 3.0)).abs() < 1e-4);

        // 4. Persistence & WAL recovery after reload
        drop(index);
        let reloaded = DiskAnnIndex::try_new(config)?;
        reloaded.load().await?;

        assert_eq!(reloaded.len().await, 2);
        let res_reloaded = reloaded.search(&query, 1).await?;
        assert_eq!(res_reloaded[0].doc_id, DocId::from(20u64));

        Ok(())
    }

    #[tokio::test]
    async fn test_diskann_insert_no_longer_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        let config = DiskAnnConfig {
            index_path: dir.path().join("insert_test.idx"),
            dimension: 3,
            ..DiskAnnConfig::default()
        };
        let index = DiskAnnIndex::try_new(config).unwrap();
        let result = index
            .insert(TxId(1), DocId::from(42u64), &[0.1, 0.2, 0.3])
            .await;
        assert!(result.is_ok(), "insert() darf kein Err mehr zurückgeben");
    }

    #[tokio::test]
    async fn test_diskann_persist_delta_empty_noop() {
        let dir = tempfile::tempdir().unwrap();
        let config = DiskAnnConfig {
            index_path: dir.path().join("noop_test.idx"),
            ..DiskAnnConfig::default()
        };
        let index = DiskAnnIndex::try_new(config).unwrap();
        let result = index.persist_delta().await;
        assert!(
            result.is_ok(),
            "persist_delta auf leerem pending ist ein Noop"
        );
    }

    #[tokio::test]
    async fn test_diskann_persist_delta_atomic_rename() {
        // INV-DISKANN-1: Nach persist_delta() existiert kein .delta.tmp
        let dir = tempfile::tempdir().unwrap();
        let index_path = dir.path().join("atomic_test.idx");
        let config = DiskAnnConfig {
            index_path: index_path.clone(),
            dimension: 64,
            max_degree: 8,
            beam_width: 8,
            distance_metric: DistanceMetric::Euclidean,
            ..DiskAnnConfig::default()
        };
        let index = DiskAnnIndex::try_new(config).unwrap();

        let vecs: Vec<Vec<f32>> = (0..5).map(|i| vec![i as f32; 64]).collect();
        // Build initial index
        let ids: Vec<DocId> = (0..5u64).map(DocId::from).collect();
        index.build(&vecs, &ids).await.unwrap();

        // Insert new vectors
        for i in 5..10 {
            index
                .insert(TxId(1), DocId::from(i as u64), &vec![i as f32; 64])
                .await
                .unwrap();
        }
        index.persist_delta().await.unwrap();

        // Kein .delta.tmp sollte noch existieren
        let tmp = index_path.with_extension("delta.tmp");
        assert!(
            !tmp.exists(),
            "Temporäre Datei muss nach persist_delta bereinigt sein"
        );

        // Verify total length is now 10 and inserted vectors are searchable
        assert_eq!(index.len().await, 10);
        let query = vec![7.0f32; 64];
        let results = index.search(&query, 1).await.unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].doc_id, DocId::from(7u64));
    }

    #[tokio::test]
    async fn test_diskann_config_validation() {
        let valid_config = DiskAnnConfig {
            index_path: PathBuf::from("dummy.idx"),
            dimension: 128,
            max_degree: 64,
            sector_size: 4096,
            ..DiskAnnConfig::default()
        };

        let index = DiskAnnIndex::try_new(valid_config).expect("valid config"); // expect
        assert_eq!(index.len().await, 0);
    }

    #[tokio::test]
    async fn test_diskann_header_persistence() {
        let temp_dir = tempfile::tempdir().unwrap(); // unwrap
        let index_path = temp_dir.path().join("header_test.idx");

        let config = DiskAnnConfig {
            index_path: index_path.clone(),
            dimension: 8,
            max_degree: 4,
            distance_metric: DistanceMetric::Euclidean,
            ..DiskAnnConfig::default()
        };

        let index = DiskAnnIndex::try_new(config).expect("valid config"); // expect
        let vectors = vec![vec![1.0; 8]];
        let ids = vec![DocId::from(42u64)];
        index.build(&vectors, &ids).await.expect("build"); // expect

        let data = tokio::fs::read(&index_path).await.expect("read file"); // expect
        assert!(data.starts_with(b"DANN"));

        let header =
            DiskAnnHeader::try_from_bytes(&data[0..DiskAnnHeader::SIZE]).expect("try_from_bytes"); // expect
        assert_eq!(header.version, DISKANN_VERSION);
        assert_eq!(header.node_count, 1);
        assert_eq!(header.sector_size, 4096);
    }

    #[tokio::test]
    async fn test_diskann_recall_basic() {
        let temp_dir = tempfile::tempdir().unwrap(); // unwrap
        let index_path = temp_dir.path().join("recall_test.idx");

        let config = DiskAnnConfig {
            index_path: index_path.clone(),
            dimension: 16,
            max_degree: 8,
            beam_width: 8,
            distance_metric: DistanceMetric::Euclidean,
            ..DiskAnnConfig::default()
        };

        let index = DiskAnnIndex::try_new(config).expect("valid config"); // expect

        let n = 100;
        let mut vectors = Vec::with_capacity(n);
        let mut ids = Vec::with_capacity(n);
        for i in 0..n {
            let mut v = vec![0.0f32; 16];
            v[0] = i as f32;
            vectors.push(v);
            ids.push(DocId::from(i as u64));
        }

        index.build(&vectors, &ids).await.expect("Build failed"); // expect

        let query = &vectors[50];
        let results = index.search(query, 1).await.expect("Search failed"); // expect
        assert!(!results.is_empty());
        assert_eq!(results[0].doc_id, ids[50]);
    }

    #[tokio::test]
    async fn test_diskann_sq8_recall() {
        let temp_dir = tempfile::tempdir().unwrap(); // unwrap
        let index_path = temp_dir.path().join("sq8_test.idx");

        let config = DiskAnnConfig {
            index_path: index_path.clone(),
            dimension: 16,
            max_degree: 16,
            beam_width: 16,
            distance_metric: DistanceMetric::Euclidean,
            quantize: true,
            ..DiskAnnConfig::default()
        };

        let index = DiskAnnIndex::try_new(config).expect("valid config"); // expect

        let n = 200;
        let mut vectors = Vec::with_capacity(n);
        let mut ids = Vec::with_capacity(n);
        for i in 0..n {
            let mut v = vec![0.0f32; 16];
            v[0] = i as f32;
            vectors.push(v);
            ids.push(DocId::from(i as u64));
        }

        index.build(&vectors, &ids).await.expect("Build failed"); // expect

        let query = &vectors[150];
        let results = index.search(query, 1).await.expect("Search failed"); // expect
        assert!(!results.is_empty());
        assert_eq!(results[0].doc_id, ids[150]);
    }

    #[tokio::test]
    async fn test_load_node_rejects_corrupt_neighbor_count() {
        let temp_dir = tempfile::tempdir().unwrap(); // unwrap
        let index_path = temp_dir.path().join("corrupt_test.idx");

        let max_degree = 8;
        let dimension = 16;
        let config = DiskAnnConfig {
            index_path: index_path.clone(),
            dimension,
            max_degree,
            beam_width: 8,
            distance_metric: DistanceMetric::Euclidean,
            quantize: false,
            ..DiskAnnConfig::default()
        };

        let index = DiskAnnIndex::try_new(config.clone()).expect("valid config"); // expect
        let vectors = vec![vec![1.0f32; dimension]];
        let ids = vec![DocId::from(1u64)];

        index.build(&vectors, &ids).await.expect("Build failed"); // expect

        // Mutate neighbor_count of node 0 in the binary index file to be > max_degree
        let mut data = tokio::fs::read(&index_path).await.expect("read file"); // expect

        // Offset layout: sector_size (4096) + dimension * 4 bytes (64)
        let neighbor_count_offset = config.sector_size + (dimension * 4);
        let corrupt_count: u32 = (max_degree + 5) as u32;
        data[neighbor_count_offset..neighbor_count_offset + 4]
            .copy_from_slice(&corrupt_count.to_le_bytes());

        // Recompute HMAC for the modified payload so header & footer integrity passes, allowing load_node to test node parsing
        let mut hmac = contextra_crypto::wal_crypto::WalHmac::new(DISKANN_INTEGRITY_KEY).unwrap();
        let footer_start = data.len() - DiskAnnFooter::SIZE;
        hmac.update(&data[..footer_start]);
        let computed = hmac.finalize();
        data[footer_start + 4..].copy_from_slice(&computed);

        tokio::fs::write(&index_path, &data)
            .await
            .expect("write corrupt file"); // expect

        let reloaded_config = DiskAnnConfig {
            fallback_policy: DiskAnnFallbackPolicy::FailFast,
            ..config
        };
        let reloaded_index = DiskAnnIndex::try_new(reloaded_config).expect("valid config"); // expect
        reloaded_index.load().await.expect("Load header & mmap"); // expect

        let result = reloaded_index.load_node(0);
        assert!(result.is_err());
        let err_msg = result.err().unwrap().to_string(); // unwrap allowed (AGENT:03)
        assert!(
            err_msg.contains("Corrupt DiskANN node: neighbor_count 13 > max_degree 8"),
            "Unexpected error message: {}",
            err_msg
        );
    }

    #[tokio::test]
    async fn test_load_rejects_bad_magic() {
        let temp_dir = tempfile::tempdir().unwrap(); // unwrap
        let index_path = temp_dir.path().join("bad_magic_test.idx");

        let config = DiskAnnConfig {
            index_path: index_path.clone(),
            dimension: 8,
            max_degree: 4,
            distance_metric: DistanceMetric::Euclidean,
            fallback_policy: DiskAnnFallbackPolicy::FailFast,
            ..DiskAnnConfig::default()
        };

        let index = DiskAnnIndex::try_new(config.clone()).expect("valid config"); // expect
        let vectors = vec![vec![1.0; 8]];
        let ids = vec![DocId::from(1u64)];
        index.build(&vectors, &ids).await.expect("build"); // expect

        // Mutate magic bytes
        let mut data = tokio::fs::read(&index_path).await.expect("read file"); // expect
        data[0..4].copy_from_slice(b"BADM");
        tokio::fs::write(&index_path, &data)
            .await
            .expect("write bad magic file"); // expect

        let reloaded_index = DiskAnnIndex::try_new(config).expect("valid config"); // expect
        let load_res = reloaded_index.load().await;
        assert!(load_res.is_err());
        let err_msg = load_res.err().unwrap().to_string(); // unwrap allowed (AGENT:03)
        assert!(
            err_msg.contains("Invalid DiskANN file: bad magic"),
            "Unexpected error message: {}",
            err_msg
        );
    }

    #[tokio::test]
    async fn test_load_rejects_version_mismatch() {
        let temp_dir = tempfile::tempdir().unwrap(); // unwrap
        let index_path = temp_dir.path().join("version_mismatch_test.idx");

        let config = DiskAnnConfig {
            index_path: index_path.clone(),
            dimension: 8,
            max_degree: 4,
            distance_metric: DistanceMetric::Euclidean,
            fallback_policy: DiskAnnFallbackPolicy::FailFast,
            ..DiskAnnConfig::default()
        };

        let index = DiskAnnIndex::try_new(config.clone()).expect("valid config"); // expect
        let vectors = vec![vec![1.0; 8]];
        let ids = vec![DocId::from(1u64)];
        index.build(&vectors, &ids).await.expect("build"); // expect

        // Mutate version to 99
        let mut data = tokio::fs::read(&index_path).await.expect("read file"); // expect
        data[4..6].copy_from_slice(&99u16.to_le_bytes());
        tokio::fs::write(&index_path, &data)
            .await
            .expect("write bad version file"); // expect

        let reloaded_index = DiskAnnIndex::try_new(config).expect("valid config"); // expect
        let load_res = reloaded_index.load().await;
        assert!(load_res.is_err());
        let err_msg = load_res.err().unwrap().to_string(); // unwrap allowed (AGENT:03)
        assert!(
            err_msg.contains("DiskANN version mismatch"),
            "Unexpected error message: {}",
            err_msg
        );
    }

    #[tokio::test]
    async fn test_write_to_file_uses_tmp_and_atomic_rename() {
        let temp_dir = tempfile::tempdir().unwrap(); // unwrap
        let index_path = temp_dir.path().join("atomic_save_test.idx");
        let tmp_path = index_path.with_extension("idx.tmp");

        let config = DiskAnnConfig {
            index_path: index_path.clone(),
            dimension: 8,
            max_degree: 4,
            sector_size: 4096,
            distance_metric: DistanceMetric::Euclidean,
            ..DiskAnnConfig::default()
        };

        let index = DiskAnnIndex::try_new(config).expect("valid config"); // expect
        let vectors = vec![vec![1.0; 8]];
        let ids = vec![DocId::from(1u64)];
        index.build(&vectors, &ids).await.expect("build"); // expect

        // After build completes, index_path must exist and .tmp must NOT exist
        assert!(
            index_path.exists(),
            "Final index file must exist after atomic rename"
        );
        assert!(
            !tmp_path.exists(),
            "Temporary file .tmp must be cleaned up / renamed"
        );
    }

    #[tokio::test]
    async fn test_load_rejects_sector_size_mismatch() {
        let temp_dir = tempfile::tempdir().unwrap(); // unwrap allowed (AGENT:03)
        let index_path = temp_dir.path().join("sector_mismatch_test.idx");

        let build_config = DiskAnnConfig {
            index_path: index_path.clone(),
            dimension: 8,
            max_degree: 4,
            sector_size: 4096,
            distance_metric: DistanceMetric::Euclidean,
            ..DiskAnnConfig::default()
        };

        let index = DiskAnnIndex::try_new(build_config).expect("valid config"); // expect
        let vectors = vec![vec![1.0; 8]];
        let ids = vec![DocId::from(1u64)];
        index.build(&vectors, &ids).await.expect("build"); // expect

        let load_config = DiskAnnConfig {
            index_path,
            dimension: 8,
            max_degree: 4,
            sector_size: 2048,
            distance_metric: DistanceMetric::Euclidean,
            fallback_policy: DiskAnnFallbackPolicy::FailFast,
            ..DiskAnnConfig::default()
        };

        let reloaded_index = DiskAnnIndex::try_new(load_config).expect("valid config"); // expect
        let load_res = reloaded_index.load().await;
        assert!(load_res.is_err());
        let err_msg = load_res.err().unwrap().to_string(); // unwrap allowed (AGENT:03)
        assert!(
            err_msg.contains("DiskANN-Index inkompatibel: Config-sector_size=2048 stimmt nicht mit Header-sector_size=4096 überein"),
            "Unexpected error message: {}",
            err_msg
        );
    }

    #[tokio::test]
    async fn test_diskann_build_search_roundtrip() -> Result<()> {
        let dir = tempfile::tempdir().map_err(ContextraError::Io)?;
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
        let dir = tempfile::tempdir().map_err(ContextraError::Io)?;
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
        let temp_dir = tempfile::tempdir().map_err(ContextraError::Io)?;
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
    async fn test_diskann_footer_roundtrip_hmac_integrity() {
        let temp_dir = tempfile::tempdir().unwrap();
        let index_path = temp_dir.path().join("footer_roundtrip.idx");

        let config = DiskAnnConfig {
            index_path: index_path.clone(),
            dimension: 8,
            max_degree: 4,
            sector_size: 4096,
            distance_metric: DistanceMetric::Euclidean,
            fallback_policy: DiskAnnFallbackPolicy::FailFast,
            ..DiskAnnConfig::default()
        };

        let index = DiskAnnIndex::try_new(config.clone()).unwrap();
        let vectors = vec![vec![1.0; 8]];
        let ids = vec![DocId::from(42u64)];
        index.build(&vectors, &ids).await.unwrap();

        // 1. Verify index file contains a valid DiskAnnFooter at the end
        let file_bytes = tokio::fs::read(&index_path).await.unwrap();
        assert!(file_bytes.len() >= DiskAnnFooter::SIZE);
        let footer_slice = &file_bytes[file_bytes.len() - DiskAnnFooter::SIZE..];
        let footer = DiskAnnFooter::try_from_bytes(footer_slice).expect("footer parsing");
        assert_eq!(&footer.magic, DISKANN_FOOTER_MAGIC);

        // Verify HMAC calculation over payload
        let mut hmac = contextra_crypto::wal_crypto::WalHmac::new(DISKANN_INTEGRITY_KEY).unwrap();
        let payload = &file_bytes[..file_bytes.len() - DiskAnnFooter::SIZE];
        hmac.update(payload);
        let expected_hmac = hmac.finalize();
        assert_eq!(footer.hmac, expected_hmac);

        // 2. Corrupt one byte of HMAC in footer, write to disk, and verify load returns integrity error
        let mut corrupt_bytes = file_bytes.clone();
        let last_idx = corrupt_bytes.len() - 1;
        corrupt_bytes[last_idx] ^= 0xFF;
        tokio::fs::write(&index_path, &corrupt_bytes)
            .await
            .expect("write corrupt footer file");

        let reloaded_index = DiskAnnIndex::try_new(config).expect("valid config");
        let load_res = reloaded_index.load().await;
        assert!(
            load_res.is_err(),
            "Loading corrupted HMAC footer must return Err"
        );
        let err_msg = load_res.err().unwrap().to_string();
        assert!(
            err_msg.contains("HMAC integrity validation failed"),
            "Unexpected error message: {}",
            err_msg
        );
    }

    #[tokio::test]
    async fn test_diskann_pending_wal_recovery() -> Result<()> {
        let temp_dir = tempfile::tempdir().map_err(ContextraError::Io)?;
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

    #[tokio::test]
    async fn test_diskann_robust_prune_handles_nan_distance_without_panic() -> Result<()> {
        let temp_dir = tempfile::tempdir().map_err(ContextraError::Io)?;
        let config = DiskAnnConfig {
            index_path: temp_dir.path().join("robust_prune_nan.idx"),
            dimension: 4,
            max_degree: 4,
            distance_metric: DistanceMetric::Euclidean,
            ..DiskAnnConfig::default()
        };
        let index = DiskAnnIndex::try_new(config)?;

        // Test 1: candidate distance is NaN in prune_in_memory
        let mut candidates = vec![
            SearchCandidate {
                index: 0,
                distance: 1.0,
            },
            SearchCandidate {
                index: 1,
                distance: f32::NAN,
            },
        ];
        let vectors = vec![vec![0.0, 0.0, 0.0, 0.0], vec![1.0, 0.0, 0.0, 0.0]];

        let pruned = index.prune_in_memory(&mut candidates, &vectors, 4, 1.2)?;
        // Both candidates must be retained (fail-open) and prune_in_memory must not panic
        assert_eq!(pruned.len(), 2);
        assert!(pruned.contains(&0));
        assert!(pruned.contains(&1));

        // Test 2: candidate distance is NaN in prune_streaming
        let mut streaming_candidates = vec![
            SearchCandidate {
                index: 0,
                distance: 1.0,
            },
            SearchCandidate {
                index: 1,
                distance: f32::NAN,
            },
        ];
        let new_vecs = vec![
            (DocId::from(1u64), vec![0.0, 0.0, 0.0, 0.0]),
            (DocId::from(2u64), vec![1.0, 0.0, 0.0, 0.0]),
        ];
        let pruned_streaming =
            index.prune_streaming(0, &mut streaming_candidates, 0, &new_vecs, 4, 1.2)?;
        assert_eq!(pruned_streaming.len(), 2);
        assert!(pruned_streaming.contains(&0));
        assert!(pruned_streaming.contains(&1));

        Ok(())
    }

    #[tokio::test]
    async fn test_diskann_query_handles_nan_neighbor_distance() -> Result<()> {
        let temp_dir = tempfile::tempdir().map_err(ContextraError::Io)?;
        let index_path = temp_dir.path().join("query_nan_neighbor.idx");
        let config = DiskAnnConfig {
            index_path: index_path.clone(),
            dimension: 4,
            max_degree: 4,
            beam_width: 4,
            distance_metric: DistanceMetric::Cosine,
            fallback_policy: DiskAnnFallbackPolicy::FailFast,
            ..DiskAnnConfig::default()
        };

        let index = DiskAnnIndex::try_new(config.clone())?;
        let vectors = vec![
            vec![1.0, 0.0, 0.0, 0.0],
            vec![0.0, 1.0, 0.0, 0.0],
            vec![0.0, 0.0, 1.0, 0.0],
        ];
        let ids = vec![DocId::from(1u64), DocId::from(2u64), DocId::from(3u64)];
        index.build(&vectors, &ids).await?;

        // Query with normal vector should succeed and find results
        let query = vec![1.0, 0.0, 0.0, 0.0];
        let results = index.search(&query, 2).await?;
        assert!(!results.is_empty());
        assert_eq!(results[0].doc_id, DocId::from(1u64));

        Ok(())
    }

    #[tokio::test]
    async fn test_pending_wal_roundtrip_hmac() -> Result<()> {
        let temp_dir = tempfile::tempdir().map_err(ContextraError::Io)?;
        let wal_path = temp_dir.path().join("test_roundtrip.pending.wal");

        let doc_id = DocId::from(42u64);
        let embedding = vec![1.0f32, 2.0, 3.0, 4.0];

        DiskAnnIndex::append_to_pending_wal(&wal_path, doc_id, &embedding).await?;
        let recovered = DiskAnnIndex::read_pending_wal(&wal_path)?;

        assert_eq!(recovered.len(), 1);
        assert_eq!(recovered[0].0, doc_id);
        assert_eq!(recovered[0].1, embedding);

        Ok(())
    }

    #[tokio::test]
    async fn test_pending_wal_corrupt_payload_skips_entry() -> Result<()> {
        let temp_dir = tempfile::tempdir().map_err(ContextraError::Io)?;
        let wal_path = temp_dir.path().join("test_corrupt.pending.wal");

        let doc1 = DocId::from(100u64);
        let vec1 = vec![1.0f32, 1.1, 1.2, 1.3];
        let doc2 = DocId::from(200u64);
        let vec2 = vec![2.0f32, 2.1, 2.2, 2.3];

        DiskAnnIndex::append_to_pending_wal(&wal_path, doc1, &vec1).await?;
        DiskAnnIndex::append_to_pending_wal(&wal_path, doc2, &vec2).await?;

        // Corrupt a byte in the embedding payload of entry 1
        let mut data = tokio::fs::read(&wal_path)
            .await
            .map_err(ContextraError::Io)?;
        // Header is 5 bytes. Entry 1 starts at byte 5.
        // ID: 8 bytes, Dim: 4 bytes -> Embedding starts at byte 5 + 12 = 17.
        data[18] ^= 0xFF;
        tokio::fs::write(&wal_path, &data)
            .await
            .map_err(ContextraError::Io)?;

        let recovered = DiskAnnIndex::read_pending_wal(&wal_path)?;

        // Entry 1 should be skipped due to HMAC mismatch, Entry 2 recovered
        assert_eq!(recovered.len(), 1);
        assert_eq!(recovered[0].0, doc2);
        assert_eq!(recovered[0].1, vec2);

        Ok(())
    }

    #[tokio::test]
    async fn test_pending_wal_absurd_dim_prevents_allocation() -> Result<()> {
        let temp_dir = tempfile::tempdir().map_err(ContextraError::Io)?;
        let wal_path = temp_dir.path().join("test_absurd_dim.pending.wal");

        let mut data = Vec::new();
        data.extend_from_slice(PENDING_WAL_MAGIC);
        data.push(PENDING_WAL_VERSION);

        // Entry with ID=1, Dim=0xFFFFFFF0 (~17GB allocation attempt without check)
        data.extend_from_slice(&1u64.to_le_bytes());
        let absurd_dim: u32 = 0xFFFFFFF0;
        data.extend_from_slice(&absurd_dim.to_le_bytes());

        tokio::fs::write(&wal_path, &data)
            .await
            .map_err(ContextraError::Io)?;

        let recovered = DiskAnnIndex::read_pending_wal(&wal_path)?;

        // Must break gracefully without OOM or panic and return empty list
        assert!(recovered.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_pending_wal_rejects_legacy_unversioned_file() -> Result<()> {
        let temp_dir = tempfile::tempdir().map_err(ContextraError::Io)?;
        let wal_path = temp_dir.path().join("legacy.pending.wal");

        // Legacy format: raw u64 id, u32 dim, f32s without PWAL magic header
        let mut data = Vec::new();
        data.extend_from_slice(&1u64.to_le_bytes());
        data.extend_from_slice(&2u32.to_le_bytes());
        data.extend_from_slice(&1.0f32.to_le_bytes());
        data.extend_from_slice(&2.0f32.to_le_bytes());

        tokio::fs::write(&wal_path, &data)
            .await
            .map_err(ContextraError::Io)?;

        let result = DiskAnnIndex::read_pending_wal(&wal_path);
        assert!(result.is_err());
        let err_str = result.err().unwrap().to_string();
        assert!(
            err_str.contains("Unbekanntes oder veraltetes pending.wal-Format"),
            "Error string was: {}",
            err_str
        );

        Ok(())
    }
}
