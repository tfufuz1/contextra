// FILE-CONTEXT
// ZWECK: Unit-Tests für MmapIndex (Boundary Checks, Asynchrones Oeffnen, Error-Paths).
// INVARIANTEN: Zero-Panic; Bounds-Pruefungen auf Slices; fehlerhafte Dateien werden abgefangen.
// HOTSPOTS: test_mmap_index_boundary_checks, test_mmap_open_async
// STAND: TS:2026-11-12T00:00:00Z

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::await_holding_lock
)]

use crate::error::{Result, VectorError};
use crate::persistence::header::HnswHeader;
use crate::persistence::mmap::MmapIndex;
use crate::persistence::node::NodeRecord;

#[tokio::test]
async fn test_mmap_index_boundary_checks() -> Result<()> {
    #[cfg(feature = "ports-compat")]
    {
        use crate::hnsw::{HnswConfig, HnswIndex};
        use crate::types::{DistanceMetric, DocId, TxId};

        let temp_dir = tempfile::tempdir().map_err(|e| VectorError::Storage(e.to_string()))?;
        let path = temp_dir.path().join("bounds_test.hnsw");

        let config = HnswConfig {
            dimension: 4,
            m: 16,
            ef_construction: 64,
            ef_search: 16,
            distance_metric: DistanceMetric::Euclidean,
            quantize: false,
            ..Default::default()
        };
        let index = HnswIndex::try_new(config.clone())?;
        let tx = TxId::new(1);

        for i in 1..=10u64 {
            let vec = vec![i as f32; 4];
            index.insert(tx, DocId::from(i), &vec).await?;
        }
        index.commit(tx).await?;
        index.save(&path)?;

        let mmap_index = MmapIndex::open(&path)?;

        let valid_rec = mmap_index.get_node_record(0)?;
        assert_eq!(valid_rec.doc_id, 1);

        let err_oob = mmap_index.get_node_record(100);
        assert!(err_oob.is_err());

        let err_oob_calc = mmap_index.get_node_record(usize::MAX);
        assert!(err_oob_calc.is_err());
    }

    Ok(())
}

#[tokio::test]
async fn test_mmap_open_async() -> Result<()> {
    #[cfg(feature = "ports-compat")]
    {
        use crate::hnsw::{HnswConfig, HnswIndex};
        use crate::types::{DistanceMetric, DocId, TxId};

        let temp_dir = tempfile::tempdir().map_err(|e| VectorError::Storage(e.to_string()))?;
        let path = temp_dir.path().join("test_async.hnsw");

        let config = HnswConfig {
            dimension: 4,
            m: 8,
            ef_construction: 32,
            ef_search: 16,
            distance_metric: DistanceMetric::Euclidean,
            quantize: false,
            ..Default::default()
        };
        let index = HnswIndex::try_new(config.clone())?;
        let tx = TxId::new(1);

        let mut vectors = Vec::with_capacity(100);
        for i in 1..=100u64 {
            let vec = vec![i as f32, (i * 2) as f32, (i * 3) as f32, (i * 4) as f32];
            index.insert(tx, DocId::from(i), &vec).await?;
            vectors.push((DocId::from(i), vec));
        }
        index.commit(tx).await?;

        index.save(&path)?;

        let mmap_index = HnswIndex::try_new(config)?;
        mmap_index.load_mmap(&path)?;

        let query = vec![50.1, 100.2, 150.3, 200.4];
        let search_results = mmap_index.search(&query, 5).await?;
        assert_eq!(search_results.len(), 5);

        let mut brute_force: Vec<(DocId, f32)> = vectors
            .iter()
            .map(|(doc_id, v)| {
                let dist: f32 = v
                    .iter()
                    .zip(query.iter())
                    .map(|(a, b)| (a - b).powi(2))
                    .sum::<f32>()
                    .sqrt();
                (*doc_id, dist)
            })
            .collect();
        brute_force.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        let top_mmap_doc = search_results[0].doc_id;
        let top_bf_doc = brute_force[0].0;
        assert_eq!(
            top_mmap_doc, top_bf_doc,
            "Mmap search result didn't match ground truth"
        );
    }

    Ok(())
}

#[test]
fn test_mmap_index_error_paths() {
    let err_open = MmapIndex::open("non_existent_file.hnsw");
    assert!(err_open.is_err());
}

#[test]
fn test_open_rejects_invalid_magic() -> Result<()> {
    let temp_dir = tempfile::tempdir().map_err(|e| VectorError::Storage(e.to_string()))?;
    let path = temp_dir.path().join("invalid_magic.hnsw");
    let header = HnswHeader::new(128, 16, 1, 0, -1.0, 1.0, 0, -1, 64, 64, 1);
    let mut bytes = header.to_bytes().to_vec();
    bytes[0..4].copy_from_slice(&0xDEADBEEFu32.to_le_bytes());
    std::fs::write(&path, &bytes).map_err(|e| VectorError::Storage(e.to_string()))?;

    let res = MmapIndex::open(&path);
    assert!(res.is_err(), "Expected error for invalid magic");
    if let Err(VectorError::Storage(msg)) = res {
        assert!(
            msg.contains("bad magic"),
            "Unexpected error message: {}",
            msg
        );
    } else {
        panic!("Expected VectorError::Storage, got error");
    }
    Ok(())
}

#[test]
fn test_open_rejects_unsupported_version() -> Result<()> {
    let temp_dir = tempfile::tempdir().map_err(|e| VectorError::Storage(e.to_string()))?;
    let path = temp_dir.path().join("unsupported_version.hnsw");
    let header = HnswHeader::new(128, 16, 1, 0, -1.0, 1.0, 0, -1, 64, 64, 1);
    let mut bytes = header.to_bytes().to_vec();
    let unsupported_version = 99u16;
    bytes[4..6].copy_from_slice(&unsupported_version.to_le_bytes());
    std::fs::write(&path, &bytes).map_err(|e| VectorError::Storage(e.to_string()))?;

    let res = MmapIndex::open(&path);
    assert!(res.is_err(), "Expected error for unsupported version");
    if let Err(VectorError::Storage(msg)) = res {
        assert!(
            msg.contains("Unsupported HNSW version"),
            "Unexpected error message: {}",
            msg
        );
    } else {
        panic!("Expected VectorError::Storage, got error");
    }
    Ok(())
}

#[tokio::test]
async fn test_v1_legacy_migration_load_degraded_fallback() -> Result<()> {
    let temp_dir = tempfile::tempdir().map_err(|e| VectorError::Storage(e.to_string()))?;
    let path = temp_dir.path().join("legacy_v1.hnsw");

    let v1_header_bytes = [
        0x57, 0x53, 0x4E, 0x48, // magic (HNSW little endian: 0x484E5357)
        0x01, 0x00, // version = 1
        0x02, 0x00, 0x00, 0x00, // dimension = 2
        0x10, 0x00, 0x00, 0x00, // m = 16
        0x01, // metric = 1 (Euclidean)
        0x01, // quantized = 1
        0x00, 0x00, 0x80, 0xBF, // q_min = -1.0
        0x00, 0x00, 0x80, 0x3F, // q_max = 1.0
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // node_count = 1
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // entry_point = 0
        0x40, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // nodes_offset = 64
        0x59, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // connections_offset = 64 + 25 = 89
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // last_tx_id = 1
    ];

    let mut file_bytes = v1_header_bytes.to_vec();
    let record = NodeRecord {
        doc_id: 1,
        max_layer: 0,
        vector_offset: 89,
        connections_offset: 89,
    };
    file_bytes.extend_from_slice(&record.to_bytes());
    file_bytes.push(127);
    file_bytes.push(127);

    std::fs::write(&path, &file_bytes).map_err(|e| VectorError::Storage(e.to_string()))?;

    let config = crate::hnsw::HnswConfig {
        dimension: 2,
        m: 16,
        ef_construction: 64,
        ef_search: 16,
        distance_metric: crate::types::DistanceMetric::Euclidean,
        quantize: true,
        ..Default::default()
    };
    let index = crate::hnsw::HnswIndex::try_new(config)?;
    let res = index.load_mmap(&path);
    assert!(res.is_ok(), "Expected legacy v1 format to load gracefully");
    assert!(index.quantizer().is_some());

    Ok(())
}

#[tokio::test]
async fn test_v2_save_load_roundtrip_preserves_per_dim_calibration() -> Result<()> {
    #[cfg(feature = "ports-compat")]
    {
        use crate::hnsw::{HnswConfig, HnswIndex};
        use crate::types::{DocId, TxId};

        let temp_dir = tempfile::tempdir().map_err(|e| VectorError::Storage(e.to_string()))?;
        let path = temp_dir.path().join("v2_roundtrip.hnsw");

        let config = HnswConfig {
            dimension: 2,
            m: 16,
            ef_construction: 64,
            ef_search: 16,
            distance_metric: crate::types::DistanceMetric::Euclidean,
            quantize: true,
            ..Default::default()
        };
        let index = HnswIndex::try_new(config.clone())?;
        let tx = TxId::new(1);

        for i in 0..10u64 {
            let v0 = (i as f32) * 1.5 - 5.0;
            let v1 = (i as f32) * 0.5 + 2.0;
            index.insert(tx, DocId::from(i + 1), &[v0, v1]).await?;
        }
        index.commit(tx).await?;

        index.save(&path)?;

        let loaded_index = HnswIndex::try_new(config)?;
        loaded_index.load_mmap(&path)?;

        let q_orig = index.quantizer();
        let q_loaded = loaded_index.quantizer();
        assert!(q_loaded.is_some());
        if let (Some(orig), Some(loaded)) = (q_orig, q_loaded) {
            assert_eq!(orig.mins.len(), loaded.mins.len());
            for i in 0..orig.mins.len() {
                assert!(
                    (orig.mins[i] - loaded.mins[i]).abs() < 1e-5,
                    "Min mismatch at {}",
                    i
                );
                assert!(
                    (orig.maxes[i] - loaded.maxes[i]).abs() < 1e-5,
                    "Max mismatch at {}",
                    i
                );
            }
        }
    }

    Ok(())
}

#[test]
fn test_open_rejects_truncated_header() -> Result<()> {
    let temp_dir = tempfile::tempdir().map_err(|e| VectorError::Storage(e.to_string()))?;
    let path = temp_dir.path().join("truncated_header.hnsw");
    let header = HnswHeader::new(128, 16, 1, 0, -1.0, 1.0, 0, -1, 64, 64, 1);
    let bytes = header.to_bytes();
    std::fs::write(&path, &bytes[..20]).map_err(|e| VectorError::Storage(e.to_string()))?;

    let res = MmapIndex::open(&path);
    assert!(res.is_err(), "Expected error for truncated header");
    if let Err(VectorError::Storage(msg)) = res {
        assert!(
            msg.contains("too small"),
            "Unexpected error message: {}",
            msg
        );
    } else {
        panic!("Expected VectorError::Storage, got error");
    }
    Ok(())
}

#[test]
fn test_get_connections_rejects_out_of_bounds_offset() -> Result<()> {
    let temp_dir = tempfile::tempdir().map_err(|e| VectorError::Storage(e.to_string()))?;
    let path = temp_dir.path().join("oob_connections.hnsw");
    let conn_offset = HnswHeader::SIZE + NodeRecord::SIZE;
    let header = HnswHeader::new(
        128,
        16,
        1,
        0,
        -1.0,
        1.0,
        1,
        0,
        HnswHeader::SIZE as u64,
        conn_offset as u64,
        1,
    );
    let mut file_bytes = header.to_bytes().to_vec();
    let record = NodeRecord {
        doc_id: 1,
        max_layer: 0,
        vector_offset: (HnswHeader::SIZE + NodeRecord::SIZE) as u64,
        connections_offset: 9999,
    };
    file_bytes.extend_from_slice(&record.to_bytes());
    std::fs::write(&path, &file_bytes).map_err(|e| VectorError::Storage(e.to_string()))?;

    let mmap_index = MmapIndex::open(&path)?;
    let res = mmap_index.get_connections(&record, 0);
    assert!(
        res.is_err(),
        "Expected error for out-of-bounds connections_offset"
    );
    if let Err(VectorError::Storage(msg)) = res {
        assert!(
            msg.contains("out of bounds"),
            "Unexpected error message: {}",
            msg
        );
    } else {
        panic!("Expected VectorError::Storage, got error");
    }
    Ok(())
}

#[test]
fn test_get_vector_rejects_out_of_bounds_offset() -> Result<()> {
    let temp_dir = tempfile::tempdir().map_err(|e| VectorError::Storage(e.to_string()))?;
    let path = temp_dir.path().join("oob_vector.hnsw");
    let header = HnswHeader::new(
        128,
        16,
        1,
        0,
        -1.0,
        1.0,
        1,
        0,
        HnswHeader::SIZE as u64,
        (HnswHeader::SIZE + NodeRecord::SIZE) as u64,
        1,
    );
    let mut file_bytes = header.to_bytes().to_vec();
    let record = NodeRecord {
        doc_id: 1,
        max_layer: 0,
        vector_offset: (HnswHeader::SIZE + NodeRecord::SIZE) as u64,
        connections_offset: (HnswHeader::SIZE + NodeRecord::SIZE) as u64,
    };
    file_bytes.extend_from_slice(&record.to_bytes());
    std::fs::write(&path, &file_bytes).map_err(|e| VectorError::Storage(e.to_string()))?;

    let mmap_index = MmapIndex::open(&path)?;
    let mut bad_record = record;
    bad_record.vector_offset = 9999;
    let res = mmap_index.get_vector(&bad_record);
    assert!(
        res.is_err(),
        "Expected error for out-of-bounds vector_offset"
    );
    if let Err(VectorError::Storage(msg)) = res {
        assert!(
            msg.contains("out of bounds"),
            "Unexpected error message: {}",
            msg
        );
    } else {
        panic!("Expected VectorError::Storage, got error");
    }
    Ok(())
}

#[test]
fn test_get_connections_rejects_absurd_length_field() -> Result<()> {
    let temp_dir = tempfile::tempdir().map_err(|e| VectorError::Storage(e.to_string()))?;
    let path = temp_dir.path().join("absurd_length.hnsw");
    let conn_offset = HnswHeader::SIZE + NodeRecord::SIZE;
    let header = HnswHeader::new(
        128,
        16,
        1,
        0,
        -1.0,
        1.0,
        1,
        0,
        HnswHeader::SIZE as u64,
        conn_offset as u64,
        1,
    );
    let mut file_bytes = header.to_bytes().to_vec();
    let record = NodeRecord {
        doc_id: 1,
        max_layer: 0,
        vector_offset: (HnswHeader::SIZE + NodeRecord::SIZE) as u64,
        connections_offset: (HnswHeader::SIZE + NodeRecord::SIZE) as u64,
    };
    file_bytes.extend_from_slice(&record.to_bytes());
    file_bytes.push(1);
    file_bytes.extend_from_slice(&u32::MAX.to_le_bytes());
    std::fs::write(&path, &file_bytes).map_err(|e| VectorError::Storage(e.to_string()))?;

    let mmap_index = MmapIndex::open(&path)?;
    let res = mmap_index.get_connections(&record, 0);
    assert!(res.is_err(), "Expected error for absurd length field");
    if let Err(VectorError::Storage(msg)) = res {
        assert!(
            msg.contains("overflow") || msg.contains("out of bounds"),
            "Unexpected error message: {}",
            msg
        );
    } else {
        panic!("Expected VectorError::Storage, got error");
    }
    Ok(())
}
