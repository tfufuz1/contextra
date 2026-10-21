use contextra_ports::TextIndex;
use contextra_text::inverted::InvertedIndex;
use contextra_types::{DocId, TxId};
use std::sync::Arc;

#[path = "../src/inverted/tests/mock.rs"]
mod mock;

use mock::MockStorage;

#[tokio::test]
async fn bm25_ranking_deterministic_across_runs() -> contextra_types::Result<()> {
    let storage = Arc::new(MockStorage::new());
    let index = InvertedIndex::new(storage.clone(), "det_ns");

    let tx = TxId::internal();
    let docs = vec![
        (DocId::new(1), "rust programming language performance search engine"),
        (DocId::new(2), "rust language systems programming performance"),
        (DocId::new(3), "rust language search performance ranking engine"),
        (DocId::new(4), "rust programming performance language ranking"),
        (DocId::new(5), "search engine ranking performance language rust"),
    ];

    for (doc_id, text) in docs {
        index.insert(tx, doc_id, text).await?;
    }
    index.commit(tx).await?;

    let mut first_ids_and_scores = Vec::new();
    for i in 0..100 {
        let results = index.search("rust language performance", 5).await?;
        let ids_and_scores: Vec<(DocId, f32)> = results.iter().map(|r| (r.doc_id, r.score)).collect();
        if i == 0 {
            first_ids_and_scores = ids_and_scores;
        } else {
            assert_eq!(
                ids_and_scores, first_ids_and_scores,
                "Run {} yielded non-deterministic search results",
                i
            );
        }
    }

    assert!(!first_ids_and_scores.is_empty());
    Ok(())
}
