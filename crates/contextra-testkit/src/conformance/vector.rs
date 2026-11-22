use super::failure::{ConformanceFailure, ConformanceResult};
use contextra_ports::VectorIndex;
use contextra_types::{DocId, TxId};
use std::future::Future;

/// Runs the complete conformance test suite for a [`VectorIndex`] implementation.
///
/// # Contract Requirements Tested
/// 1. Insert & Search distance ordering: items closer to query score higher and appear first.
/// 2. Deletion & Tombstone consistency: deleted docs do not appear in `search` or `all_doc_ids`.
/// 3. Upsert: re-inserting same `DocId` updates its vector without duplicating `len`.
/// 4. Dimension mismatch rejection: returning an error if vector length differs from index dimension.
/// 5. NaN rejection: returning an error if embedding contains `f32::NAN`.
/// 6. Search `k` larger than index size: returns all available items without out-of-bounds error.
/// 7. Support for full range of `DocId`s.
pub async fn run_vector_index_suite<F, Fut, V>(factory: F) -> ConformanceResult
where
    F: Fn() -> Fut,
    Fut: Future<Output = V>,
    V: VectorIndex,
{
    let suite = "VectorIndex";

    // Test 1: Distance ordering in search
    {
        let index = factory().await;
        let tx = TxId::new(1);
        let query = [1.0f32, 0.0, 0.0];
        let v_close = [0.9f32, 0.1, 0.0];
        let v_far = [0.0f32, 1.0, 0.0];

        if let Err(e) = index.insert(tx, DocId(10), &v_far).await {
            return Err(ConformanceFailure::new(
                suite,
                "distance_ordering",
                format!("insert v_far failed: {e:?}"),
            ));
        }
        if let Err(e) = index.insert(tx, DocId(20), &v_close).await {
            return Err(ConformanceFailure::new(
                suite,
                "distance_ordering",
                format!("insert v_close failed: {e:?}"),
            ));
        }
        let _ = index.commit(tx).await;

        let results = match index.search(&query, 2).await {
            Ok(r) => r,
            Err(e) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "distance_ordering",
                    format!("search failed: {e:?}"),
                ))
            }
        };

        if results.len() != 2 {
            return Err(ConformanceFailure::new(
                suite,
                "distance_ordering",
                format!("expected 2 search results, got {}", results.len()),
            ));
        }
        let first_doc = results.first().map(|d| d.doc_id);
        if first_doc != Some(DocId(20)) {
            return Err(ConformanceFailure::new(
                suite,
                "distance_ordering",
                format!("closer doc 20 should be first, got {first_doc:?}"),
            ));
        }
    }

    // Test 2: Delete & Tombstone consistency
    {
        let index = factory().await;
        let tx1 = TxId::new(10);
        let _ = index.insert(tx1, DocId(1), &[1.0, 0.0, 0.0]).await;
        let _ = index.insert(tx1, DocId(2), &[0.0, 1.0, 0.0]).await;
        let _ = index.commit(tx1).await;

        let tx2 = TxId::new(11);
        if let Err(e) = index.delete(tx2, DocId(1)).await {
            return Err(ConformanceFailure::new(
                suite,
                "delete_tombstone",
                format!("delete failed: {e:?}"),
            ));
        }
        let _ = index.commit(tx2).await;

        let active_ids = match index.all_doc_ids().await {
            Ok(ids) => ids,
            Err(e) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "delete_tombstone",
                    format!("all_doc_ids failed: {e:?}"),
                ))
            }
        };
        if active_ids.contains(&DocId(1)) {
            return Err(ConformanceFailure::new(
                suite,
                "delete_tombstone",
                format!("deleted DocId(1) still in all_doc_ids: {active_ids:?}"),
            ));
        }

        let search_res = match index.search(&[1.0, 0.0, 0.0], 10).await {
            Ok(r) => r,
            Err(e) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "delete_tombstone",
                    format!("search after delete failed: {e:?}"),
                ))
            }
        };
        if search_res.iter().any(|d| d.doc_id == DocId(1)) {
            return Err(ConformanceFailure::new(
                suite,
                "delete_tombstone",
                format!("deleted DocId(1) still returned in search: {search_res:?}"),
            ));
        }
    }

    // Test 3: Upsert
    {
        let index = factory().await;
        let tx1 = TxId::new(20);
        let _ = index.insert(tx1, DocId(100), &[1.0, 0.0, 0.0]).await;
        let _ = index.commit(tx1).await;

        let tx2 = TxId::new(21);
        let _ = index.insert(tx2, DocId(100), &[0.0, 1.0, 0.0]).await;
        let _ = index.commit(tx2).await;

        if index.len().await != 1 {
            return Err(ConformanceFailure::new(
                suite,
                "upsert",
                format!("len after upsert expected 1, got {}", index.len().await),
            ));
        }

        let search_near_new = match index.search(&[0.0, 1.0, 0.0], 1).await {
            Ok(r) => r,
            Err(e) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "upsert",
                    format!("search failed: {e:?}"),
                ))
            }
        };
        let first_near = search_near_new.first().map(|d| d.doc_id);
        if first_near != Some(DocId(100)) {
            return Err(ConformanceFailure::new(
                suite,
                "upsert",
                format!("upserted DocId(100) not found near new vector: {search_near_new:?}"),
            ));
        }
    }

    // Test 4: Dimension mismatch
    {
        let index = factory().await;
        let tx = TxId::new(30);
        let _ = index.insert(tx, DocId(1), &[1.0, 0.0, 0.0]).await;
        let _ = index.commit(tx).await;

        let res_insert = index.insert(tx, DocId(2), &[1.0, 0.0]).await;
        if res_insert.is_ok() {
            return Err(ConformanceFailure::new(
                suite,
                "dimension_mismatch",
                "inserting wrong dimension vector should fail",
            ));
        }

        let res_search = index.search(&[1.0, 0.0], 1).await;
        if res_search.is_ok() {
            return Err(ConformanceFailure::new(
                suite,
                "dimension_mismatch",
                "searching with wrong dimension vector should fail",
            ));
        }
    }

    // Test 5: NaN rejection
    {
        let index = factory().await;
        let tx = TxId::new(40);
        let nan_vec = [f32::NAN, 0.0, 0.0];

        let res_insert = index.insert(tx, DocId(1), &nan_vec).await;
        if res_insert.is_ok() {
            return Err(ConformanceFailure::new(
                suite,
                "nan_rejection",
                "inserting NaN vector should return error",
            ));
        }

        let res_search = index.search(&nan_vec, 1).await;
        if res_search.is_ok() {
            return Err(ConformanceFailure::new(
                suite,
                "nan_rejection",
                "searching NaN vector should return error",
            ));
        }
    }

    // Test 6: k > size
    {
        let index = factory().await;
        let tx = TxId::new(50);
        let _ = index.insert(tx, DocId(1), &[1.0, 0.0, 0.0]).await;
        let _ = index.commit(tx).await;

        let results = match index.search(&[1.0, 0.0, 0.0], 100).await {
            Ok(r) => r,
            Err(e) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "top_k_larger_than_size",
                    format!("search failed: {e:?}"),
                ))
            }
        };
        if results.len() != 1 {
            return Err(ConformanceFailure::new(
                suite,
                "top_k_larger_than_size",
                format!(
                    "expected 1 result when size=1 and k=100, got {}",
                    results.len()
                ),
            ));
        }
    }

    // Test 7: DocId edge cases
    {
        let index = factory().await;
        let tx = TxId::new(60);
        let high_doc_id = DocId(u64::MAX - 1);
        let _ = index.insert(tx, high_doc_id, &[0.5, 0.5, 0.0]).await;
        let _ = index.commit(tx).await;

        let search_res = match index.search(&[0.5, 0.5, 0.0], 1).await {
            Ok(r) => r,
            Err(e) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "doc_id_handling",
                    format!("search failed: {e:?}"),
                ))
            }
        };
        let first_high = search_res.first().map(|d| d.doc_id);
        if first_high != Some(high_doc_id) {
            return Err(ConformanceFailure::new(
                suite,
                "doc_id_handling",
                format!("expected high DocId {high_doc_id:?}, got {search_res:?}"),
            ));
        }
    }

    Ok(())
}
