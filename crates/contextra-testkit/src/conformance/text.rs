use super::failure::{ConformanceFailure, ConformanceResult};
use contextra_ports::TextIndex;
use contextra_types::{DocId, TxId};
use std::future::Future;

/// Runs the complete conformance test suite for a [`TextIndex`] implementation.
///
/// # Contract Requirements Tested
/// 1. Document indexing & search retrieval.
/// 2. Deletion: once deleted and committed, document is never retrievable again.
/// 3. German Umlaute and ß case normalization matching (e.g. "Grüße" / "Köln").
/// 4. Empty query handling without errors or crashes.
pub async fn run_text_index_suite<F, Fut, T>(factory: F) -> ConformanceResult
where
    F: Fn() -> Fut,
    Fut: Future<Output = T>,
    T: TextIndex,
{
    let suite = "TextIndex";

    // Test 1: Indexing & search
    {
        let index = factory().await;
        let tx = TxId::new(1);
        let doc_text = "Rust async concurrency and high performance storage";
        if let Err(e) = index.insert(tx, DocId(10), doc_text).await {
            return Err(ConformanceFailure::new(
                suite,
                "indexing_and_search",
                format!("insert failed: {e:?}"),
            ));
        }
        let _ = index.commit(tx).await;

        let search_res = match index.search("concurrency", 5).await {
            Ok(r) => r,
            Err(e) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "indexing_and_search",
                    format!("search failed: {e:?}"),
                ))
            }
        };

        let first_doc = search_res.first().map(|d| d.doc_id);
        if first_doc != Some(DocId(10)) {
            return Err(ConformanceFailure::new(
                suite,
                "indexing_and_search",
                format!("expected DocId(10), got {search_res:?}"),
            ));
        }
    }

    // Test 2: Deletion => never retrievable again
    {
        let index = factory().await;
        let tx1 = TxId::new(10);
        let _ = index
            .insert(tx1, DocId(1), "secret confidential document")
            .await;
        let _ = index.commit(tx1).await;

        let tx2 = TxId::new(11);
        if let Err(e) = index.delete(tx2, DocId(1)).await {
            return Err(ConformanceFailure::new(
                suite,
                "delete_never_retrievable",
                format!("delete failed: {e:?}"),
            ));
        }
        let _ = index.commit(tx2).await;

        let search_res = match index.search("confidential", 5).await {
            Ok(r) => r,
            Err(e) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "delete_never_retrievable",
                    format!("search after delete failed: {e:?}"),
                ))
            }
        };
        if search_res.iter().any(|d| d.doc_id == DocId(1)) {
            return Err(ConformanceFailure::new(
                suite,
                "delete_never_retrievable",
                format!("deleted document DocId(1) still found in search: {search_res:?}"),
            ));
        }
    }

    // Test 3: Umlaute / ß normalization
    {
        let index = factory().await;
        let tx = TxId::new(20);
        let _ = index
            .insert(tx, DocId(100), "Herzliche Grüße aus Köln am Rhein")
            .await;
        let _ = index.commit(tx).await;

        let search_umlaute = match index.search("Köln", 5).await {
            Ok(r) => r,
            Err(e) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "umlaute_ß_normalization",
                    format!("search Köln failed: {e:?}"),
                ))
            }
        };
        if search_umlaute.is_empty() {
            return Err(ConformanceFailure::new(
                suite,
                "umlaute_ß_normalization",
                "search for 'Köln' returned empty",
            ));
        }

        let search_ss = match index.search("grüsse", 5).await {
            Ok(r) => r,
            Err(e) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "umlaute_ß_normalization",
                    format!("search grüsse failed: {e:?}"),
                ))
            }
        };
        if search_ss.is_empty() {
            return Err(ConformanceFailure::new(
                suite,
                "umlaute_ß_normalization",
                "search for 'grüsse' matching 'Grüße' returned empty",
            ));
        }
    }

    // Test 4: Empty query handling
    {
        let index = factory().await;
        let tx = TxId::new(30);
        let _ = index.insert(tx, DocId(1), "Sample text content").await;
        let _ = index.commit(tx).await;

        let empty_search = match index.search("", 10).await {
            Ok(r) => r,
            Err(e) => {
                return Err(ConformanceFailure::new(
                    suite,
                    "empty_query",
                    format!("search with empty string query failed: {e:?}"),
                ))
            }
        };
        let _ = empty_search;
    }

    Ok(())
}
