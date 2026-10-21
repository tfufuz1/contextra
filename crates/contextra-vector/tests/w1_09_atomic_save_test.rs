use contextra_ports::VectorIndex;
use contextra_types::DocId;
use contextra_vector::hnsw::{HnswConfigBuilder, HnswIndex};
use std::path::Path;

#[tokio::test]
async fn w1_09_hnsw_save_crash_atomic() -> contextra_types::Result<()> {
    let temp_dir = tempfile::tempdir().unwrap();
    let index_path = temp_dir.path().join("index.hnsw");

    let config = HnswConfigBuilder::new(4).build().unwrap();
    let index = HnswIndex::try_new(config)?;

    let doc1 = DocId::new(1);
    index.insert(contextra_types::TxId::internal(), doc1, &[1.0, 0.0, 0.0, 0.0]).await?;
    index.commit(contextra_types::TxId::internal()).await?;

    // Initial save
    index.save(&index_path).await?;
    assert!(index_path.exists());

    // Second index with updated doc
    let index2 = HnswIndex::try_new(HnswConfigBuilder::new(4).build().unwrap())?;
    let doc2 = DocId::new(2);
    index2.insert(contextra_types::TxId::internal(), doc2, &[0.0, 1.0, 0.0, 0.0]).await?;
    index2.commit(contextra_types::TxId::internal()).await?;

    // Simulate crash before rename: write temp file, then simulate crash before atomic replace finishes
    let tmp_file = index_path.with_extension("hnsw.tmp");
    std::fs::write(&tmp_file, b"corrupted temp file data").unwrap();

    // Opening index_path should succeed and load index1 completely
    let loaded = HnswIndex::try_new(HnswConfigBuilder::new(4).build().unwrap())?;
    loaded.load_mmap(&index_path).await?;
    let stats = loaded.stats().await?;
    assert_eq!(stats.num_vectors, 1);

    // After full save, index2 is completely saved
    index2.save(&index_path).await?;
    let loaded2 = HnswIndex::try_new(HnswConfigBuilder::new(4).build().unwrap())?;
    loaded2.load_mmap(&index_path).await?;
    let stats2 = loaded2.stats().await?;
    assert_eq!(stats2.num_vectors, 1);

    Ok(())
}

#[tokio::test]
#[cfg(feature = "experimental-diskann")]
async fn w1_09_diskann_build_crash_atomic() -> contextra_types::Result<()> {
    use contextra_vector::diskann::{DiskAnnConfig, DiskAnnIndex};

    let temp_dir = tempfile::tempdir().unwrap();
    let index_path = temp_dir.path().join("diskann.idx");

    let config = DiskAnnConfig {
        index_path: index_path.clone(),
        dimension: 4,
        max_degree: 4,
        beam_width: 4,
        pending_flush_threshold: Some(10),
        ..DiskAnnConfig::default()
    };
    let index = DiskAnnIndex::try_new(config)?;

    let vecs: Vec<Vec<f32>> = (0..10).map(|i| vec![i as f32, 0.0, 0.0, 0.0]).collect();
    let ids: Vec<DocId> = (0..10).map(|i| DocId::new(100 + i as u64)).collect();
    index.build(&vecs, &ids).await?;

    let pending_doc = DocId::new(200);
    index.insert(contextra_types::TxId::internal(), pending_doc, &[10.0, 0.0, 0.0, 0.0]).await?;

    // Failpoint test: "after_rename"
    index.set_failpoint("after_rename");
    let res = index.persist_delta().await;
    assert!(res.is_err(), "persist_delta must fail when failpoint is hit");

    // Clear failpoint
    index.clear_failpoint();

    // After restart/reload, index is valid
    index.load().await?;

    Ok(())
}

#[test]
fn no_std_fs_in_async_fn_vector_text() {
    use std::fs;

    fn check_dir(dir: &Path, violations: &mut Vec<String>) {
        if !dir.exists() {
            return;
        }
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                check_dir(&path, violations);
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                let content = fs::read_to_string(&path).unwrap();
                let mut in_async_fn = false;
                let mut brace_depth = 0;
                let mut async_brace_depth = 0;

                for (line_num, line) in content.lines().enumerate() {
                    let trimmed = line.trim();
                    if trimmed.contains("async fn ") || trimmed.contains("async move") {
                        in_async_fn = true;
                        async_brace_depth = brace_depth;
                    }

                    if in_async_fn && trimmed.contains("std::fs::") {
                        // Exclude comments
                        if !trimmed.starts_with("//") && !trimmed.starts_with("/*") {
                            violations.push(format!(
                                "{}:{}: {}",
                                path.display(),
                                line_num + 1,
                                trimmed
                            ));
                        }
                    }

                    for c in line.chars() {
                        if c == '{' {
                            brace_depth += 1;
                        } else if c == '}' {
                            brace_depth -= 1;
                            if in_async_fn && brace_depth <= async_brace_depth {
                                in_async_fn = false;
                            }
                        }
                    }
                }
            }
        }
    }

    let mut violations = Vec::new();
    check_dir(Path::new("crates/contextra-vector/src"), &mut violations);
    check_dir(Path::new("crates/contextra-text/src"), &mut violations);

    assert!(
        violations.is_empty(),
        "Found blocking std::fs:: calls inside async fn in vector/text:\n{:#?}",
        violations
    );
}
