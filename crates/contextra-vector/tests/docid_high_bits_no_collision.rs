#![forbid(unsafe_code)]

use contextra_core::{DistanceMetric, DocId, TxId, VectorIndex};
use contextra_vector::hnsw::{HnswConfigBuilder, HnswIndex};
use std::collections::HashSet;

/// Independent Brute-Force Oracle for Vector Search Verification.
/// Erwartungswerte stammen aus dieser unabhängigen Referenzfunktion, nie aus den Produktionsformeln.
fn brute_force_search_oracle(
    dataset: &[(DocId, Vec<f32>)],
    active_ids: &HashSet<DocId>,
    query: &[f32],
    k: usize,
) -> Vec<DocId> {
    let mut scored: Vec<(DocId, f32)> = dataset
        .iter()
        .filter(|(id, _)| active_ids.contains(id))
        .map(|(id, vec)| {
            // Cosine distance / Negative Dot product for normalized vectors
            let dot: f32 = query.iter().zip(vec.iter()).map(|(&a, &b)| a * b).sum();
            let distance = 1.0 - dot;
            (*id, distance)
        })
        .collect();

    scored.sort_by(|a, b| a.1.total_cmp(&b.1).then_with(|| a.0.cmp(&b.0)));
    scored.into_iter().take(k).map(|(id, _)| id).collect()
}

#[tokio::test]
async fn test_hnsw_docid_high_bits_isolation_and_tombstones() -> contextra_core::Result<()> {
    let dim = 4;
    let config = HnswConfigBuilder::new(dim)
        .m(16)
        .ef_construction(64)
        .ef_search(32)
        .distance_metric(DistanceMetric::Cosine)
        .build()?;

    let index = HnswIndex::try_new(config)?;

    // SCHRITT 2: Zwei DocIds, die sich NUR in den oberen 64 Bit unterscheiden (falls docid-128 aktiviert ist).
    #[cfg(feature = "docid-128")]
    let id1 = DocId::from(1_u128);
    #[cfg(feature = "docid-128")]
    let id2 = DocId::from(1_u128 | (1_u128 << 64));

    #[cfg(not(feature = "docid-128"))]
    let id1 = DocId::from(1_u64);
    #[cfg(not(feature = "docid-128"))]
    let id2 = DocId::from(1001_u64);

    let id_max = DocId::MAX;
    let id_min = DocId::MIN; // DocId(0)

    let v1 = vec![1.0, 0.0, 0.0, 0.0];
    let v2 = vec![0.0, 1.0, 0.0, 0.0];
    let v_max = vec![0.0, 0.0, 1.0, 0.0];
    let v_min = vec![0.0, 0.0, 0.0, 1.0];

    let dataset = vec![
        (id1, v1.clone()),
        (id2, v2.clone()),
        (id_max, v_max.clone()),
        (id_min, v_min.clone()),
    ];

    let mut active_ids: HashSet<DocId> = dataset.iter().map(|(id, _)| *id).collect();

    // 1. Insert all 4 distinct DocIds
    let tx1 = TxId::new(1);
    for (id, v) in &dataset {
        index.insert(tx1, *id, v).await?;
    }
    index.commit(tx1).await?;

    // 2. Verify all DocIds are present in index map
    let all_map_ids = index.all_doc_ids_from_map();
    assert_eq!(
        all_map_ids.len(),
        4,
        "Index must retain all 4 distinct DocIds without collision"
    );
    assert!(all_map_ids.contains(&id1), "id1 must be present");
    assert!(all_map_ids.contains(&id2), "id2 must be present");
    assert!(all_map_ids.contains(&id_max), "DocId::MAX must be present");
    assert!(
        all_map_ids.contains(&id_min),
        "DocId::MIN (0) must be present"
    );

    // 3. Compare top-1 search results against independent Brute-Force Oracle
    for (id, query_vec) in &dataset {
        let expected_top = brute_force_search_oracle(&dataset, &active_ids, query_vec, 1);
        let res = index.search(query_vec, 1).await?;
        assert_eq!(res.len(), 1);
        assert_eq!(
            res[0].doc_id, expected_top[0],
            "Search for DocId {:?} must return expected oracle hit {:?}",
            id, expected_top[0]
        );
    }

    // 4. Delete id1 only
    let tx2 = TxId::new(2);
    index.delete(tx2, id1).await?;
    index.commit(tx2).await?;
    active_ids.remove(&id1);

    // 5. Verify id1 is deleted while id2, id_max, id_min remain intact
    let remaining_ids = index.all_doc_ids_from_map();
    assert_eq!(
        remaining_ids.len(),
        3,
        "Deleting id1 must leave exactly 3 items"
    );
    assert!(!remaining_ids.contains(&id1), "id1 must be deleted");
    assert!(
        remaining_ids.contains(&id2),
        "id2 must still be present and unaffected"
    );
    assert!(
        remaining_ids.contains(&id_max),
        "DocId::MAX must still be present"
    );
    assert!(
        remaining_ids.contains(&id_min),
        "DocId::MIN must still be present"
    );

    // 6. Query for v1 vector after id1 deletion - oracle check
    let expected_oracle_after_del = brute_force_search_oracle(&dataset, &active_ids, &v1, 1);
    let res_v1_after_del = index.search(&v1, 1).await?;
    assert_eq!(res_v1_after_del.len(), 1);
    assert_ne!(
        res_v1_after_del[0].doc_id, id1,
        "Deleted id1 must never be returned in search results"
    );
    assert_eq!(
        res_v1_after_del[0].doc_id, expected_oracle_after_del[0],
        "Top hit after deletion must match oracle"
    );

    Ok(())
}

#[cfg(feature = "experimental-diskann")]
#[tokio::test]
async fn test_diskann_docid_high_bits_isolation() -> contextra_core::Result<()> {
    use contextra_vector::diskann::{DiskAnnConfig, DiskAnnIndex};
    use tempfile::tempdir;

    let dir = tempdir().map_err(contextra_core::ContextraError::Io)?;
    let index_path = dir.path().join("high_bits_diskann.dann");

    let config = DiskAnnConfig {
        index_path,
        dimension: 4,
        max_degree: 8,
        beam_width: 16,
        ..Default::default()
    };

    let index = DiskAnnIndex::try_new(config)?;

    #[cfg(feature = "docid-128")]
    let id1 = DocId::from(1_u128);
    #[cfg(feature = "docid-128")]
    let id2 = DocId::from(1_u128 | (1_u128 << 64));

    #[cfg(not(feature = "docid-128"))]
    let id1 = DocId::from(1_u64);
    #[cfg(not(feature = "docid-128"))]
    let id2 = DocId::from(1001_u64);

    let id_max = DocId::MAX;
    let id_min = DocId::MIN;

    let v1 = vec![1.0, 0.0, 0.0, 0.0];
    let v2 = vec![0.0, 1.0, 0.0, 0.0];
    let v_max = vec![0.0, 0.0, 1.0, 0.0];
    let v_min = vec![0.0, 0.0, 0.0, 1.0];

    index
        .build(
            &[v1.clone(), v2.clone(), v_max.clone(), v_min.clone()],
            &[id1, id2, id_max, id_min],
        )
        .await?;

    let all_doc_ids = index.all_doc_ids().await?;
    assert_eq!(all_doc_ids.len(), 4);
    assert!(all_doc_ids.contains(&id1));
    assert!(all_doc_ids.contains(&id2));
    assert!(all_doc_ids.contains(&id_max));
    assert!(all_doc_ids.contains(&id_min));

    // Delete id1 only
    index.delete(TxId::new(1), id1).await?;

    let remaining_doc_ids = index.all_doc_ids().await?;
    assert_eq!(remaining_doc_ids.len(), 3);
    assert!(!remaining_doc_ids.contains(&id1));
    assert!(remaining_doc_ids.contains(&id2));

    Ok(())
}

/// Gegenprobe (Mutation Test):
/// Modelliert das fehlerhafte Verhalten (as u64 Trunkierung von DocIds mit identischen unteren 64 Bit).
/// Dieser Test zeigt, dass der Test das Problem zuverlässig aufdeckt (ROT wird), wenn eine as u64 Kürzung vorliegt.
#[test]
#[cfg(feature = "docid-128")]
fn test_mutation_counter_proof_truncated_u64_causes_collision() {
    use ahash::AHashMap;

    let id1 = DocId::from(1_u128);
    let id2 = DocId::from(1_u128 | (1_u128 << 64));

    // Mutated implementation: Casts DocId to u64
    let mut truncated_map: AHashMap<u64, &str> = AHashMap::new();
    truncated_map.insert(id1.inner() as u64, "doc1");
    truncated_map.insert(id2.inner() as u64, "doc2");

    // Failure detection: id2 overwrote id1 because lower 64 bits are identical!
    let contains_id1 = truncated_map.contains_key(&(id1.inner() as u64))
        && truncated_map.get(&(id1.inner() as u64)) == Some(&"doc1");
    assert!(
        !contains_id1,
        "MUTATION PROOF: Truncating DocId to u64 causes collision and erases id1"
    );
}
