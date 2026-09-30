// FILE-CONTEXT
// ZWECK: Unit-Tests für HNSW Persistenz-Schicht.

use super::*;
use contextra_core::{ContextraError, Result};

#[test]
fn test_hnsw_header_roundtrip_and_errors() -> Result<()> {
    let header = HnswHeader::new(128, 16, 1, 0, -1.0, 1.0, 100, 5, 84, 256, 42);

    let bytes = header.to_bytes();
    assert_eq!(bytes.len(), HnswHeader::SIZE);
    let parsed = HnswHeader::try_from_bytes(&bytes)?;
    assert_eq!(header, parsed);

    let small_bytes = vec![0u8; 10];
    let err_small = HnswHeader::try_from_bytes(&small_bytes);
    assert!(matches!(err_small, Err(ContextraError::Storage(_))));

    let mut bad_magic_bytes = bytes;
    bad_magic_bytes[0..4].copy_from_slice(&0xDEADBEEFu32.to_le_bytes());
    let err_magic = HnswHeader::try_from_bytes(&bad_magic_bytes);
    assert!(matches!(err_magic, Err(ContextraError::Storage(_))));

    Ok(())
}

#[test]
fn test_node_record_roundtrip_and_errors() -> Result<()> {
    let record = NodeRecord {
        doc_id: 12345,
        max_layer: 3,
        vector_offset: 500,
        connections_offset: 1500,
    };

    let bytes = record.to_bytes();
    assert_eq!(bytes.len(), NodeRecord::SIZE);
    let parsed = NodeRecord::from_bytes(&bytes)?;
    assert_eq!(record.doc_id, parsed.doc_id);
    assert_eq!(record.max_layer, parsed.max_layer);
    assert_eq!(record.vector_offset, parsed.vector_offset);
    assert_eq!(record.connections_offset, parsed.connections_offset);

    let small_bytes = vec![0u8; 10];
    let err = NodeRecord::from_bytes(&small_bytes);
    assert!(matches!(err, Err(ContextraError::Storage(_))));

    Ok(())
}

#[tokio::test]
async fn test_mmap_index_boundary_checks() -> contextra_core::Result<()> {
    use crate::hnsw::{HnswConfig, HnswIndex};
    use contextra_core::{DistanceMetric, DocId, TxId, VectorIndex};

    let temp_dir = tempfile::tempdir().map_err(|e| ContextraError::Storage(e.to_string()))?;
    let path = temp_dir.path().join("bounds_test.hnsw");

    let config = HnswConfig {
        dimension: 4,
        m: 16,
        distance_metric: DistanceMetric::Euclidean,
        ..Default::default()
    };
    let index = HnswIndex::try_new(config.clone())?;
    let tx = TxId::new(1);
    index
        .insert(tx, DocId::new(1), &[1.0, 2.0, 3.0, 4.0])
        .await?;
    index.commit(tx).await?;
    index.save(&path).await?;

    let mmap_index = MmapIndex::open(&path)?;

    assert!(mmap_index.get_node_record(99999).is_err());

    let rec = mmap_index.get_node_record(0)?;
    let connections = mmap_index.get_connections(&rec, 100)?;
    assert!(connections.is_empty());

    let vec_bytes = mmap_index.get_vector(&rec)?;
    assert_eq!(vec_bytes.len(), 4 * 4);

    Ok(())
}

#[tokio::test]
async fn test_mmap_open_async() -> contextra_core::Result<()> {
    use crate::hnsw::{HnswConfig, HnswIndex};
    use contextra_core::{DistanceMetric, DocId, TxId, VectorIndex};

    let temp_dir = tempfile::tempdir().map_err(|e| ContextraError::Storage(e.to_string()))?;
    let path = temp_dir.path().join("test_async.hnsw");

    let config = HnswConfig {
        dimension: 4,
        m: 16,
        ef_construction: 200,
        ef_search: 64,
        distance_metric: DistanceMetric::Euclidean,
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

    index.save(&path).await?;

    let mmap_index = HnswIndex::try_new(config)?;
    mmap_index.load_mmap(&path).await?;

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
    brute_force.sort_by(|a, b| a.1.total_cmp(&b.1));

    let expected_doc_ids: Vec<DocId> = brute_force.iter().take(5).map(|(id, _)| *id).collect();
    let returned_doc_ids: Vec<DocId> = search_results.iter().map(|r| r.doc_id).collect();

    assert_eq!(returned_doc_ids, expected_doc_ids);
    Ok(())
}

#[test]
fn test_mmap_index_error_paths() {
    let res = MmapIndex::open("non_existent_path_contextra_test.hnsw");
    assert!(res.is_err());

    let short_bytes = vec![0u8; 10];
    assert!(HnswHeader::try_from_bytes(&short_bytes).is_err());

    let invalid_magic = vec![0u8; HnswHeader::SIZE];
    assert!(HnswHeader::try_from_bytes(&invalid_magic).is_err());

    let short_node = vec![0u8; 10];
    assert!(NodeRecord::from_bytes(&short_node).is_err());
}

#[test]
fn test_open_rejects_invalid_magic() -> Result<()> {
    let temp_dir = tempfile::tempdir().map_err(|e| ContextraError::Storage(e.to_string()))?;
    let path = temp_dir.path().join("invalid_magic.hnsw");
    let header = HnswHeader::new(128, 16, 1, 0, -1.0, 1.0, 0, -1, 64, 64, 1);
    let mut bytes = header.to_bytes().to_vec();
    bytes[0..4].copy_from_slice(&0xDEADBEEFu32.to_le_bytes());
    std::fs::write(&path, &bytes).map_err(|e| ContextraError::Storage(e.to_string()))?;

    let res = MmapIndex::open(&path);
    assert!(res.is_err(), "Expected error for invalid magic");
    if let Err(ContextraError::Storage(msg)) = res {
        assert!(
            msg.contains("bad magic"),
            "Unexpected error message: {}",
            msg
        );
    } else {
        panic!("Expected Storage error");
    }
    Ok(())
}

#[test]
fn test_open_rejects_unsupported_version() -> Result<()> {
    let temp_dir = tempfile::tempdir().map_err(|e| ContextraError::Storage(e.to_string()))?;
    let path = temp_dir.path().join("unsupported_version.hnsw");
    let header = HnswHeader::new(128, 16, 1, 0, -1.0, 1.0, 0, -1, 64, 64, 1);
    let mut bytes = header.to_bytes().to_vec();
    let unsupported_version = 99u16;
    bytes[4..6].copy_from_slice(&unsupported_version.to_le_bytes());
    std::fs::write(&path, &bytes).map_err(|e| ContextraError::Storage(e.to_string()))?;

    let res = MmapIndex::open(&path);
    assert!(res.is_err(), "Expected error for unsupported version");
    if let Err(ContextraError::Storage(msg)) = res {
        assert!(
            msg.contains("Unsupported HNSW version"),
            "Unexpected error message: {}",
            msg
        );
    } else {
        panic!("Expected Storage error");
    }
    Ok(())
}

#[tokio::test]
async fn test_v1_legacy_migration_load_degraded_fallback() -> Result<()> {
    use crate::hnsw::{HnswConfig, HnswIndex};

    let temp_dir = tempfile::tempdir().map_err(|e| ContextraError::Storage(e.to_string()))?;
    let path = temp_dir.path().join("legacy_v1.hnsw");

    let v1_header_bytes = [
        0x57, 0x53, 0x4E, 0x48, // magic "HNSW"
        0x01, 0x00, // version 1
        0x02, 0x00, 0x00, 0x00, // dimension 2
        0x10, 0x00, 0x00, 0x00, // m 16
        0x01, // metric Euclidean
        0x01, // quantized = 1
        0x00, 0x00, 0x00, 0x00, // q_min = 0.0
        0x00, 0x00, 0x80, 0x3F, // q_max = 1.0
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // node_count = 1
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // entry_point = 0
        64, 0, 0, 0, 0, 0, 0, 0, // nodes_offset = 64
        89, 0, 0, 0, 0, 0, 0, 0, // connections_offset = 89
        0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // last_tx_id = 1
    ];

    let mut file_bytes = v1_header_bytes.to_vec();
    let record = NodeRecord {
        doc_id: 1,
        max_layer: 0,
        vector_offset: 89 + 1 + 4 + 4,
        connections_offset: 89,
    };
    file_bytes.extend_from_slice(&record.to_bytes());
    file_bytes.push(1);
    file_bytes.extend_from_slice(&1u32.to_le_bytes());
    file_bytes.extend_from_slice(&0u32.to_le_bytes());
    file_bytes.push(127);
    file_bytes.push(127);

    std::fs::write(&path, &file_bytes).map_err(|e| ContextraError::Storage(e.to_string()))?;

    let config = HnswConfig {
        dimension: 2,
        quantize: true,
        ..Default::default()
    };
    let index = HnswIndex::try_new(config)?;
    index.load_mmap(&path).await?;

    let q = index
        .quantizer()
        .expect("Quantizer must be populated on mmap load");
    assert_eq!(q.mins(), &[0.0, 0.0]);
    assert_eq!(q.maxes(), &[1.0, 1.0]);

    Ok(())
}

#[tokio::test]
async fn test_v2_save_load_roundtrip_preserves_per_dim_calibration() -> Result<()> {
    use crate::hnsw::{HnswConfig, HnswIndex};
    use contextra_core::{DocId, TxId, VectorIndex};

    let temp_dir = tempfile::tempdir().map_err(|e| ContextraError::Storage(e.to_string()))?;
    let path = temp_dir.path().join("v2_roundtrip.hnsw");

    let config = HnswConfig {
        dimension: 2,
        quantize: true,
        ..Default::default()
    };
    let index = HnswIndex::try_new(config.clone())?;
    let tx = TxId::new(1);

    for i in 0..60u64 {
        let v0 = (i as f32) / 59.0;
        let v1 = (i as f32) * 1000.0 / 59.0;
        index.insert(tx, DocId::from(i + 1), &[v0, v1]).await?;
    }
    index.commit(tx).await?;

    index.save(&path).await?;

    let loaded_index = HnswIndex::try_new(config)?;
    loaded_index.load_mmap(&path).await?;

    let q = loaded_index
        .quantizer()
        .expect("Quantizer must be restored");
    assert!(
        (q.mins()[0] - 0.0).abs() < 1e-2,
        "mins[0] was {}",
        q.mins()[0]
    );
    assert!(
        (q.mins()[1] - 0.0).abs() < 1e-2,
        "mins[1] was {}",
        q.mins()[1]
    );
    assert!(
        (q.maxes()[0] - 1.0).abs() < 1e-2,
        "maxes[0] was {}",
        q.maxes()[0]
    );
    assert!(
        (q.maxes()[1] - 1000.0).abs() < 10.0,
        "maxes[1] was {}",
        q.maxes()[1]
    );

    Ok(())
}

#[test]
fn test_open_rejects_truncated_header() -> Result<()> {
    let temp_dir = tempfile::tempdir().map_err(|e| ContextraError::Storage(e.to_string()))?;
    let path = temp_dir.path().join("truncated_header.hnsw");
    let header = HnswHeader::new(128, 16, 1, 0, -1.0, 1.0, 0, -1, 64, 64, 1);
    let bytes = header.to_bytes();
    std::fs::write(&path, &bytes[..20]).map_err(|e| ContextraError::Storage(e.to_string()))?;

    let res = MmapIndex::open(&path);
    assert!(res.is_err(), "Expected error for truncated header");
    if let Err(ContextraError::Storage(msg)) = res {
        assert!(
            msg.contains("too small"),
            "Unexpected error message: {}",
            msg
        );
    } else {
        panic!("Expected Storage error");
    }
    Ok(())
}

#[test]
fn test_get_connections_rejects_out_of_bounds_offset() -> Result<()> {
    let temp_dir = tempfile::tempdir().map_err(|e| ContextraError::Storage(e.to_string()))?;
    let path = temp_dir.path().join("oob_connections.hnsw");
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
        (HnswHeader::SIZE + 25) as u64,
        1,
    );
    let mut file_bytes = header.to_bytes().to_vec();
    let record = NodeRecord {
        doc_id: 1,
        max_layer: 1,
        vector_offset: (HnswHeader::SIZE + 25) as u64,
        connections_offset: 9999,
    };
    file_bytes.extend_from_slice(&record.to_bytes());
    std::fs::write(&path, &file_bytes).map_err(|e| ContextraError::Storage(e.to_string()))?;

    let mmap_index = MmapIndex::open(&path)?;
    let res = mmap_index.get_connections(&record, 0);
    assert!(
        res.is_err(),
        "Expected error for out-of-bounds connections_offset"
    );
    if let Err(ContextraError::Storage(msg)) = res {
        assert!(
            msg.contains("out of bounds"),
            "Unexpected error message: {}",
            msg
        );
    } else {
        panic!("Expected Storage error");
    }
    Ok(())
}

#[test]
fn test_get_vector_rejects_out_of_bounds_offset() -> Result<()> {
    let temp_dir = tempfile::tempdir().map_err(|e| ContextraError::Storage(e.to_string()))?;
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
        max_layer: 1,
        vector_offset: (HnswHeader::SIZE + NodeRecord::SIZE) as u64,
        connections_offset: (HnswHeader::SIZE + NodeRecord::SIZE) as u64,
    };
    file_bytes.extend_from_slice(&record.to_bytes());
    std::fs::write(&path, &file_bytes).map_err(|e| ContextraError::Storage(e.to_string()))?;

    let mmap_index = MmapIndex::open(&path)?;
    let mut bad_record = record;
    bad_record.vector_offset = 9999;
    let res = mmap_index.get_vector(&bad_record);
    assert!(
        res.is_err(),
        "Expected error for out-of-bounds vector_offset"
    );
    if let Err(ContextraError::Storage(msg)) = res {
        assert!(
            msg.contains("out of bounds"),
            "Unexpected error message: {}",
            msg
        );
    } else {
        panic!("Expected Storage error");
    }
    Ok(())
}

#[test]
fn test_get_connections_rejects_absurd_length_field() -> Result<()> {
    let temp_dir = tempfile::tempdir().map_err(|e| ContextraError::Storage(e.to_string()))?;
    let path = temp_dir.path().join("absurd_length.hnsw");
    let conn_offset = HnswHeader::SIZE + NodeRecord::SIZE;
    let header = HnswHeader::new(
        4,
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
        max_layer: 1,
        vector_offset: conn_offset as u64,
        connections_offset: conn_offset as u64,
    };
    file_bytes.extend_from_slice(&record.to_bytes());
    file_bytes.push(1);
    file_bytes.extend_from_slice(&u32::MAX.to_le_bytes());
    std::fs::write(&path, &file_bytes).map_err(|e| ContextraError::Storage(e.to_string()))?;

    let mmap_index = MmapIndex::open(&path)?;
    let res = mmap_index.get_connections(&record, 0);
    assert!(res.is_err(), "Expected error for absurd length field");
    if let Err(ContextraError::Storage(msg)) = res {
        assert!(
            msg.contains("overflow") || msg.contains("out of bounds"),
            "Unexpected error message: {}",
            msg
        );
    } else {
        panic!("Expected Storage error");
    }
    Ok(())
}
