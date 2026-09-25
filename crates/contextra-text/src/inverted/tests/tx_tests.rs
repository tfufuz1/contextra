use super::mock::MockStorage;
use crate::inverted::{BM25MorphIndex, InvertedIndex, Language, TextIndexMetadata};
use crate::tokenizer::{DefaultTokenizer, Tokenizer};
use contextra_ports::{StorageEngine, TextIndex};
use contextra_types::{ContextraError, DocId, Result, TxId};
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::Arc;

#[tokio::test]
async fn staged_insert_not_visible_before_commit() -> Result<()> {
    let storage = Arc::new(MockStorage::new());
    let index = InvertedIndex::new(storage.clone(), "staged_test");
    let tx = TxId::new(1);
    let doc_id = DocId::new(42);

    let seq_before = storage.last_seq_no().await?;

    index.upsert_document(tx, doc_id, "test content").await?;

    // Before commit (with snapshot isolation at previous committed sequence), search returns empty
    let results_before = index.search_bm25_at("test", 10, Some(seq_before)).await?;
    assert!(
        results_before.is_empty(),
        "Uncommitted upsert must not be visible at seq_before"
    );

    // After commit: readable
    index.commit(tx).await?;
    let results_after = index.search("test", 10).await?;
    assert!(
        !results_after.is_empty(),
        "Committed insert must be visible in search"
    );

    Ok(())
}

#[tokio::test]
async fn inverted_index_rollback_cleans_up() -> Result<()> {
    let storage = Arc::new(MockStorage::new());
    let index = InvertedIndex::new(storage.clone(), "rollback_test");

    let tx1 = TxId::new(1);
    let doc_id = DocId::new(100);

    index
        .insert(tx1, doc_id, "German morphological search engine")
        .await?;
    assert_eq!(index.search("morphological", 10).await?.len(), 1);

    index.rollback(tx1).await?;

    let results = index.search("morphological", 10).await?;
    assert!(
        results.is_empty(),
        "Rolled-back insert must not be visible in search results"
    );
    assert_eq!(
        index.len().await,
        0,
        "Index length must be 0 after rollback"
    );

    Ok(())
}

#[tokio::test]
async fn test_text_search_snapshot_isolation() -> Result<()> {
    let storage = Arc::new(MockStorage::new());
    let index = InvertedIndex::new(storage.clone(), "isolation");

    // 1. Initial State: Document 1
    let tx1 = TxId::new(1);
    let d1 = DocId::new(1);
    index.upsert_document(tx1, d1, "rust programming").await?;
    storage.commit(tx1).await?;
    let seq1 = storage.last_seq_no().await?;

    // 2. Second State: Document 2 (should not be visible at seq1)
    let tx2 = TxId::new(2);
    let d2 = DocId::new(2);
    index.upsert_document(tx2, d2, "rust compiler").await?;
    storage.commit(tx2).await?;

    // 3. Verify Isolation
    // At seq1, only d1 should exist
    let results_at_seq1 = index.search_at("rust", 10, seq1).await?;
    assert_eq!(results_at_seq1.len(), 1);
    assert_eq!(results_at_seq1[0].doc_id, d1);

    // At latest (None or higher seq), both should exist
    let results_latest = index.search("rust", 10).await?;
    assert_eq!(results_latest.len(), 2);

    Ok(())
}
#[tokio::test]
async fn test_avgdl_accuracy_incremental_updates() -> Result<()> {
    let storage = Arc::new(MockStorage::new());
    let index = InvertedIndex::new(storage.clone(), "test_avgdl");

    // Step 1: Insert 10 documents of varying length
    // We use distinct words so DefaultTokenizer doesn't filter them out as stopwords.
    let docs = [
        "alpha",                                                                        // 1 token
        "beta gamma",                                                                   // 2 tokens
        "delta epsilon zeta",                                                           // 3 tokens
        "eta theta iota kappa",                                                         // 4 tokens
        "lambda mu nu xi omicron",                                                      // 5 tokens
        "pi rho sigma tau upsilon phi",                                                 // 6 tokens
        "chi psi omega apple banana cherry",                                            // 7 tokens
        "date elderberry fig grape hazelnut kiwi lemon",                                // 8 tokens
        "mango nectarine orange papaya quince raspberry strawberry tangerine",          // 9 tokens
        "ugli vanilla walnut ximenia yuzu ziziphus apricot blueberry cranberry durian", // 10 tokens
    ];

    let mut current_doc_lengths = HashMap::new();

    for (i, text) in docs.iter().enumerate() {
        let doc_id = DocId::new((i + 1) as u64);
        let tx = TxId::new((i + 1) as u64);
        let tokens = DefaultTokenizer.tokenize(text);
        current_doc_lengths.insert(doc_id, tokens.len() as u64);

        index.upsert_document(tx, doc_id, text).await?;
        index.commit_stats(tx).await?;
        storage.commit(tx).await?;
    }

    assert_eq!(index.total_docs.load(Ordering::SeqCst), 10);

    // Step 2: Delete 5 documents (DocId 1..=5)
    for i in 1..=5 {
        let doc_id = DocId::new(i as u64);
        let tx = TxId::new(100 + i as u64);
        current_doc_lengths.remove(&doc_id);

        index.delete_document(tx, doc_id).await?;
        index.commit_stats(tx).await?;
        storage.commit(tx).await?;
    }

    assert_eq!(index.total_docs.load(Ordering::SeqCst), 5);

    // Step 3: Insert 5 more documents (DocId 11..=15)
    let new_docs = [
        "ant bee cat dog",                               // 4 tokens
        "elephant fox giraffe hippo iguana",             // 5 tokens
        "jaguar koala lion monkey newt owl",             // 6 tokens
        "panda quail rabbit snake tiger urchin vulture", // 7 tokens
        "whale wolf yak zebra ant bee cat dog elephant", // 9 tokens
    ];

    for (i, text) in new_docs.iter().enumerate() {
        let doc_id = DocId::new((11 + i) as u64);
        let tx = TxId::new(200 + i as u64);
        let tokens = DefaultTokenizer.tokenize(text);
        current_doc_lengths.insert(doc_id, tokens.len() as u64);

        index.upsert_document(tx, doc_id, text).await?;
        index.commit_stats(tx).await?;
        storage.commit(tx).await?;
    }

    // Verify state
    let total_docs = index.total_docs.load(Ordering::SeqCst);
    let total_tokens = index.total_tokens.load(Ordering::SeqCst);
    assert_eq!(total_docs, 10, "Should have 10 active documents");

    let expected_total_tokens: u64 = current_doc_lengths.values().sum();
    assert_eq!(
        total_tokens, expected_total_tokens,
        "Total tokens must match active documents"
    );

    let expected_avgdl = expected_total_tokens as f64 / total_docs as f64;
    let actual_avgdl = index.avg_doc_len_x1000.load(Ordering::SeqCst) as f64 / 1000.0;

    assert!(
        (actual_avgdl - expected_avgdl).abs() < 0.001,
        "avgdl ({}) must equal mean length of current 10 documents ({})",
        actual_avgdl,
        expected_avgdl
    );

    Ok(())
}

#[tokio::test]
async fn test_idf_recalculation_with_rare_term_after_delete() -> Result<()> {
    let storage = Arc::new(MockStorage::new());
    let index = InvertedIndex::new(storage.clone(), "idf_rare_test");

    // Corpus: doc1 ("common rare"), doc2 ("common rare"), doc3..doc10 ("common filler")
    // term "rare" starts with N=10, df=2
    let tx = TxId::new(1);
    let d1 = DocId::new(1);
    let d2 = DocId::new(2);
    index.insert(tx, d1, "common rare").await?;
    index.insert(tx, d2, "common rare").await?;
    for i in 3..=10 {
        index.insert(tx, DocId::new(i), "common filler").await?;
    }
    index.commit(tx).await?;

    // Score for "rare" in d2 when N=10, df=2
    let search_before = index.search_bm25("rare", 10, None).await?;
    let score_before = search_before
        .iter()
        .find(|(id, _)| *id == d2)
        .ok_or_else(|| ContextraError::InvalidInput("d2 not found in search_before".into()))?
        .1;

    // Delete d1 -> N=9, df=1 for "rare"
    let tx_del = TxId::new(2);
    index.delete(tx_del, d1).await?;
    index.commit(tx_del).await?;

    // Score for "rare" in d2 when N=9, df=1
    let search_after = index.search_bm25("rare", 10, None).await?;
    let score_after = search_after
        .iter()
        .find(|(id, _)| *id == d2)
        .ok_or_else(|| ContextraError::InvalidInput("d2 not found in search_after".into()))?
        .1;

    // With N=10, df=2: idf_arg = (10 - 2 + 0.5)/(2 + 0.5) = 8.5 / 2.5 = 3.4 -> ln(3.4) ~= 1.2237
    // With N=9, df=1: idf_arg = (9 - 1 + 0.5)/(1 + 0.5) = 8.5 / 1.5 = 5.6667 -> ln(5.6667) ~= 1.7346
    // So IDF and score_after must be significantly higher than score_before
    assert!(
            score_after > score_before,
            "Deleting doc containing 'rare' decreased df from 2 to 1, so BM25 score for 'rare' must increase (after: {}, before: {})",
            score_after,
            score_before
        );

    Ok(())
}

#[tokio::test]
async fn test_text_index_search_at_snapshot_isolation() -> Result<()> {
    let storage = Arc::new(MockStorage::new());
    let index = InvertedIndex::new(storage.clone(), "text_search_at_test");

    // tx1: insert doc 1 "hello world"
    let tx1 = TxId::new(1);
    let doc1 = DocId::new(1);
    index.insert(tx1, doc1, "hello world").await?;
    index.commit(tx1).await?;
    let seq1 = storage.last_seq_no().await?;

    // tx2: insert doc 2 "hello rust"
    let tx2 = TxId::new(2);
    let doc2 = DocId::new(2);
    index.insert(tx2, doc2, "hello rust").await?;
    index.commit(tx2).await?;
    let seq2 = storage.last_seq_no().await?;

    // search_at seq1: should return doc1, but NOT doc2
    let res_seq1 = index.search_at("hello", 10, seq1).await?;
    assert_eq!(res_seq1.len(), 1);
    assert_eq!(res_seq1[0].doc_id, doc1);

    // search_at seq2: should return doc1 and doc2
    let res_seq2 = index.search_at("hello", 10, seq2).await?;
    assert_eq!(res_seq2.len(), 2);

    Ok(())
}

#[tokio::test]
async fn test_upsert_document_oversized_text_returns_error() -> Result<()> {
    let storage = Arc::new(MockStorage::new());
    let index = InvertedIndex::new(storage.clone(), "oversized_test");
    let tx = TxId::new(1);
    let doc_id = DocId::new(1);

    // Construct oversized string (> 10 MiB)
    let oversized = "a".repeat(InvertedIndex::<MockStorage>::MAX_TEXT_BYTES + 1);
    let err = index
        .upsert_document(tx, doc_id, &oversized)
        .await
        .unwrap_err();

    match err {
        ContextraError::InvalidInput(msg) => {
            assert!(msg.contains("exceeds maximum allowed size"));
        }
        _ => panic!("Expected ContextraError::InvalidInput, got {:?}", err),
    }

    Ok(())
}

#[test]
fn text_index_metadata_bincode_roundtrip() {
    let meta = TextIndexMetadata {
        total_docs: 42,
        total_tokens: 1337,
        avg_doc_len_x1000: 31833,
    };
    let bytes = bincode::serialize(&meta).expect("serialization succeeds"); // unwrap allowed
    let deserialized: TextIndexMetadata =
        bincode::deserialize(&bytes).expect("deserialization succeeds"); // unwrap allowed
    assert_eq!(meta.total_docs, deserialized.total_docs);
    assert_eq!(meta.total_tokens, deserialized.total_tokens);
    assert_eq!(meta.avg_doc_len_x1000, deserialized.avg_doc_len_x1000);
}

#[test]
fn language_from_iso_case_iso_codes_and_fallback() {
    assert_eq!(Language::from_iso("de"), Language::German);
    assert_eq!(Language::from_iso("DE"), Language::German);
    assert_eq!(Language::from_iso("de-DE"), Language::German);

    assert_eq!(Language::from_iso("en"), Language::English);
    assert_eq!(Language::from_iso("EN"), Language::English);
    assert_eq!(Language::from_iso("en-US"), Language::English);

    // Unknown ISO falls back to English
    assert_eq!(Language::from_iso("fr"), Language::English);
    assert_eq!(Language::from_iso("es"), Language::English);
    assert_eq!(Language::from_iso(""), Language::English);
}

#[tokio::test]
async fn bm25_morph_index_case_full_lifecycle() -> Result<()> {
    let storage = Arc::new(MockStorage::new());
    let morph_index = BM25MorphIndex::new(
        storage,
        "morph_ns",
        Arc::new(crate::morphology::PassthroughTokenizer::new("de")),
    );

    assert_eq!(morph_index.tokenizer().language(), "de");

    let tx = TxId::new(1);
    let doc_id = DocId::new(10);
    morph_index
        .insert(tx, doc_id, "test morphological index")
        .await?;
    morph_index.commit(tx).await?;

    assert_eq!(morph_index.len().await, 1);
    let results = morph_index.search("morphological", 5).await?;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].doc_id, doc_id);

    let stats = morph_index.stats().await?;
    assert_eq!(stats.num_documents, 1);

    morph_index.delete(tx, doc_id).await?;
    morph_index.commit(tx).await?;
    assert_eq!(morph_index.len().await, 0);

    morph_index.rollback(tx).await?;
    morph_index.rollback_to_tx(tx).await?;
    assert_eq!(morph_index.last_tx_id().await?, TxId(0));

    Ok(())
}

#[tokio::test]
async fn inverted_index_case_empty_and_unicode_inputs() -> Result<()> {
    let storage = Arc::new(MockStorage::new());
    let mut index = InvertedIndex::new(storage, "edge_ns");
    index = index.with_tokenizer(Arc::new(crate::tokenizer::DefaultTokenizer));

    let tx = TxId::new(1);
    let doc_id = DocId::new(5);

    // Insert empty string (doc is registered in doc_count, but 0 tokens indexed)
    index.insert(tx, doc_id, "").await?;
    index.commit(tx).await?;
    assert_eq!(index.len().await, 1);

    let doc_id2 = DocId::new(6);
    // Insert unicode text
    index
        .insert(tx, doc_id2, "Ärger über Ölpreise in Düsseldorf")
        .await?;
    index.commit(tx).await?;
    assert_eq!(index.len().await, 2);

    // Search empty query
    let empty_res = index.search("", 10).await?;
    assert!(empty_res.is_empty());

    // Search k=0
    let k0_res = index.search("ölpreise", 0).await?;
    assert!(k0_res.is_empty());

    // Search unicode query
    let unicode_res = index.search("ölpreise", 10).await?;
    assert_eq!(unicode_res.len(), 1);
    assert_eq!(unicode_res[0].doc_id, doc_id2);

    // Search oversized query returns error
    let oversized_q = "q".repeat(InvertedIndex::<MockStorage>::MAX_TEXT_BYTES + 1);
    let err = index.search(&oversized_q, 10).await.unwrap_err(); // unwrap allowed
    assert!(matches!(err, ContextraError::InvalidInput(_)));

    Ok(())
}

#[tokio::test]
async fn test_staged_stats_limit_exceeded() -> Result<()> {
    let storage = Arc::new(MockStorage::new());
    let index = InvertedIndex::new(storage, "limit_ns");

    // Fill staged_stats up to MAX_STAGED_TRANSACTIONS
    for i in 1..=InvertedIndex::<MockStorage>::MAX_STAGED_TRANSACTIONS {
        let tx = TxId::new(i as u64);
        let doc_id = DocId::new(i as u64);
        index.upsert_document(tx, doc_id, "test document").await?;
    }

    // The (MAX_STAGED_TRANSACTIONS + 1)-th transaction should fail with InvalidInput
    let overflow_tx = TxId::new((InvertedIndex::<MockStorage>::MAX_STAGED_TRANSACTIONS + 1) as u64);
    let overflow_doc = DocId::new(999_999);
    let err = index
        .upsert_document(overflow_tx, overflow_doc, "overflow text")
        .await
        .unwrap_err(); // unwrap allowed
    assert!(matches!(err, ContextraError::InvalidInput(_)));

    Ok(())
}

#[tokio::test]
async fn test_load_stats_persistence() -> Result<()> {
    let storage = Arc::new(MockStorage::new());
    let index1 = InvertedIndex::new(storage.clone(), "persisted_stats");

    let tx = TxId::new(1);
    index1
        .upsert_document(tx, DocId::new(1), "rust memory safe search engine")
        .await?;
    index1.commit(tx).await?;

    // Instantiate a second InvertedIndex over the same storage and reload stats
    let index2 = InvertedIndex::new(storage.clone(), "persisted_stats");
    assert_eq!(index2.total_docs.load(Ordering::SeqCst), 0);
    assert_eq!(index2.total_tokens.load(Ordering::SeqCst), 0);

    index2.load_stats().await?;

    assert_eq!(index2.total_docs.load(Ordering::SeqCst), 1);
    assert!(index2.total_tokens.load(Ordering::SeqCst) > 0);
    assert!(index2.avg_doc_len_x1000.load(Ordering::SeqCst) > 0);

    Ok(())
}

#[tokio::test]
async fn test_inverted_index_clone_sharing() -> Result<()> {
    let storage = Arc::new(MockStorage::new());
    let index1 = InvertedIndex::new(storage, "clone_sharing");
    let index2 = index1.clone();

    let tx = TxId::new(1);
    index1
        .upsert_document(tx, DocId::new(10), "cloned index sharing atomic state")
        .await?;
    index1.commit(tx).await?;

    // State should be reflected in index2 because atomics & locks are shared via Arcs
    assert_eq!(index2.total_docs.load(Ordering::SeqCst), 1);
    assert!(index2.total_tokens.load(Ordering::SeqCst) > 0);

    let results = index2.search("sharing", 10).await?;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].doc_id, DocId::new(10));

    Ok(())
}
