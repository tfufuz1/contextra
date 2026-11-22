use contextra_ports::{BoxFuture, StorageEngine, StorageStats, TextIndex};
use contextra_text::morphology::{
    normalize_umlauts, GermanCompoundSplitter, MorphologicalTokenizer,
};
use contextra_text::tokenizer::{DefaultTokenizer, GermanMorphTokenizer, Tokenizer};
use contextra_text::{InvertedIndex, TextPurger};
use contextra_types::{ContextraError, DocId, Result, TxId};
use parking_lot::RwLock;
use proptest::prelude::*;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

struct SimpleMockStore {
    data: RwLock<HashMap<Vec<u8>, Vec<u8>>>,
    next_seq: AtomicU64,
}

impl SimpleMockStore {
    fn new() -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
            next_seq: AtomicU64::new(1),
        }
    }
}

impl StorageEngine for SimpleMockStore {
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
        Box::pin(async move { Ok(self.data.read().get(key).cloned().map(bytes::Bytes::from)) })
    }

    fn put<'a>(
        &'a self,
        _tx_id: TxId,
        key: &'a [u8],
        value: &'a [u8],
    ) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.data.write().insert(key.to_vec(), value.to_vec());
            Ok(())
        })
    }

    fn delete<'a>(&'a self, _tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.data.write().remove(key);
            Ok(())
        })
    }

    fn commit<'a>(&'a self, _tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn rollback<'a>(&'a self, _tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn rollback_to_tx<'a>(&'a self, _tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn get_at_seq<'a>(
        &'a self,
        key: &'a [u8],
        _seq: u64,
    ) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
        Box::pin(async move { self.get(key).await })
    }

    fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, Result<u64>> {
        Box::pin(async move { Ok(self.next_seq.load(Ordering::SeqCst)) })
    }

    fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> {
        Box::pin(async move { Ok(TxId::new(1)) })
    }

    fn flush<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn stats<'a>(&'a self) -> BoxFuture<'a, Result<StorageStats>> {
        Box::pin(async move {
            Ok(StorageStats {
                num_segments: 0,
                total_size_bytes: 0,
                memtable_size_bytes: 0,
            })
        })
    }

    fn pin_checkpoint<'a>(&'a self, _id: u64) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn unpin_checkpoint<'a>(&'a self, _id: u64) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn scan<'a>(
        &'a self,
        _start: std::ops::Bound<&'a [u8]>,
        _end: std::ops::Bound<&'a [u8]>,
        _: Option<usize>,
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move { Ok(Vec::new()) })
    }

    fn scan_prefix<'a>(
        &'a self,
        prefix: &'a [u8],
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move {
            let data = self.data.read();
            let mut res = Vec::new();
            for (k, v) in data.iter() {
                if k.starts_with(prefix) {
                    res.push((k.clone(), v.clone()));
                }
            }
            Ok(res)
        })
    }

    fn scan_prefix_at<'a>(
        &'a self,
        prefix: &'a [u8],
        _seq_no: u64,
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move { self.scan_prefix(prefix).await })
    }
}

// Fixed seed configuration for deterministic proptests
fn deterministic_config() -> proptest::test_runner::Config {
    proptest::test_runner::Config {
        cases: 100,
        source_file: Some("tests/index_purge_proptests.rs"),
        ..Default::default()
    }
}

proptest! {
    #![proptest_config(deterministic_config())]

    #[test]
    fn prop_compound_splitter_umlauts_and_sz_never_panic(s in "[a-zA-ZäöüÄÖÜß-]{1,50}") {
        let splitter = GermanCompoundSplitter::new();
        let norm = normalize_umlauts(&s.to_lowercase());
        let decomp = splitter.decompose(&norm);
        prop_assert!(!decomp.is_empty());
    }

    #[test]
    fn prop_compound_splitter_hyphenated_never_panic(s in "[a-z]{3,10}-[a-z]{3,10}") {
        let splitter = GermanCompoundSplitter::new();
        let decomp = splitter.decompose(&s);
        prop_assert!(!decomp.is_empty());
    }

    #[test]
    fn prop_tokenizer_case_and_normalization_determinism(s in "[a-zäöüßA-ZÄÖÜ]{1,30}") {
        let tok = GermanMorphTokenizer::default();
        let lower = s.to_lowercase();
        let norm = normalize_umlauts(&lower);

        let tokens_orig = tok.tokenize(&s);
        let tokens_lower = tok.tokenize(&lower);

        // Case insensitivity: tokenizing uppercase vs lowercase produces identical tokens
        prop_assert_eq!(tokens_orig.clone(), tokens_lower);

        // Decomposed normalized form is always included in German morph tokenizer output
        let norm_tokens = tok.tokenize(&norm);
        for nt in &norm_tokens {
            prop_assert!(tokens_orig.contains(nt));
        }
    }
}

#[tokio::test]
async fn test_index_purge_search_never_returns_purged_doc() -> Result<()> {
    let storage = Arc::new(SimpleMockStore::new());
    let index = Arc::new(InvertedIndex::new(storage.clone(), "test_prop_purge"));

    let tx = TxId::new(1);
    let doc1 = DocId::from(1001u64);
    let doc2 = DocId::from(1002u64);
    let doc3 = DocId::from(1003u64);

    index
        .upsert_document(tx, doc1, "arbeitsvertrag fuer softwareentwicklung")
        .await?;
    index
        .upsert_document(tx, doc2, "qualitaetspruefung im krankenhaus")
        .await?;
    index
        .upsert_document(tx, doc3, "softwareentwicklung und qualitaetspruefung")
        .await?;
    index.commit(tx).await?;

    let hits_before = index.search("softwareentwicklung", 10).await?;
    assert_eq!(hits_before.len(), 2);

    let purger = TextPurger::new(index.clone());
    let report = purger.purge_document_report(tx, doc1);
    assert!(report.is_success());

    let hits_after = index.search("softwareentwicklung", 10).await?;
    assert_eq!(hits_after.len(), 1);
    assert_eq!(hits_after[0].doc_id, doc3);

    Ok(())
}

#[tokio::test]
async fn test_purge_search_non_retrieval_after_reload() -> Result<()> {
    let storage = Arc::new(SimpleMockStore::new());
    let index = Arc::new(InvertedIndex::new(storage.clone(), "test_reload_purge"));

    let tx = TxId::new(1);
    let doc1 = DocId::from(2001u64);
    index
        .upsert_document(tx, doc1, "datenschutzrichtlinie im unternehmen")
        .await?;
    index.commit(tx).await?;

    let purger = TextPurger::new(index.clone());
    let report = purger.purge_document_report(tx, doc1);
    assert!(report.is_success());

    // Reload: construct new InvertedIndex instance pointing to same storage
    let reloaded_index = Arc::new(InvertedIndex::new(storage.clone(), "test_reload_purge"));
    let hits_reloaded = reloaded_index.search("datenschutzrichtlinie", 10).await?;
    assert!(
        hits_reloaded.is_empty(),
        "Purged document must remain unretrievable after cold reload"
    );

    Ok(())
}

#[test]
fn test_compound_splitter_edge_case_exact_128_bytes() {
    let splitter = GermanCompoundSplitter::new();
    let exact_128 = "a".repeat(128);
    let parts = splitter.decompose(&exact_128);
    assert_eq!(parts.len(), 1);

    let over_128 = "a".repeat(129);
    let parts_over = splitter.decompose(&over_128);
    assert_eq!(parts_over.len(), 1);
}

#[test]
fn test_compound_splitter_edge_case_empty_and_short() {
    let splitter = GermanCompoundSplitter::new();
    assert_eq!(splitter.decompose(""), vec![""]);
    assert_eq!(splitter.decompose("a"), vec!["a"]);
    assert_eq!(splitter.decompose("ab"), vec!["ab"]);
}

#[test]
fn test_compound_splitter_edge_case_hyphenated_words() {
    let splitter = GermanCompoundSplitter::new();
    let parts = splitter.decompose("kmu-software");
    assert!(!parts.is_empty());
}

#[test]
fn test_compound_splitter_edge_case_german_umlauts_and_sz() {
    let splitter = GermanCompoundSplitter::new();
    assert_eq!(normalize_umlauts("GROSSSTADTLEBEN"), "grossstadtleben");
    let parts = splitter.decompose("grossstadtleben");
    assert!(!parts.is_empty());
}

#[test]
fn test_tokenizer_try_new_fallible_and_deprecated_new() {
    let default_tok_res = DefaultTokenizer::try_new();
    assert!(default_tok_res.is_ok());

    #[allow(deprecated)]
    let default_tok = DefaultTokenizer::new();
    assert!(!default_tok.tokenize("hello world").is_empty());

    let german_tok_res = GermanMorphTokenizer::try_new();
    assert!(german_tok_res.is_ok());

    #[allow(deprecated)]
    let german_tok = GermanMorphTokenizer::new();
    assert!(!german_tok.tokenize("datenschutzrichtlinie").is_empty());
}

#[test]
fn test_purge_report_default_and_new_equivalence() {
    use contextra_text::PurgeReport;
    let r1 = PurgeReport::new();
    let r2 = PurgeReport::default();
    assert_eq!(r1, r2);
    assert!(r1.is_success());
    assert_eq!(r1.units_purged, 0);
    assert_eq!(r1.units_failed, 0);
    assert_eq!(r1.bytes_freed, 0);
    assert!(r1.errors.is_empty());
}

#[test]
fn test_purge_report_partial_failure_error_formatting() {
    use contextra_text::PurgeReport;
    let mut r = PurgeReport::new();
    r.units_purged = 5;
    r.units_failed = 2;
    r.errors.push("Failed to delete key plb:test".into());

    assert!(!r.is_success());
    let stats_res = r.to_purge_stats();
    assert!(stats_res.is_err());
    if let Err(ContextraError::Text(msg)) = stats_res {
        assert!(msg.contains("Purge incomplete: 2 units failed"));
        assert!(msg.contains("plb:test"));
    } else {
        panic!("Expected ContextraError::Text");
    }
}

#[test]
fn test_compile_protected_regex_valid_pattern() {
    use contextra_text::tokenizer::compile_protected_regex;
    let re_res = compile_protected_regex(r"[a-z]+@[a-z]+\.com");
    assert!(re_res.is_ok());
    let re = re_res.unwrap();
    assert!(re.is_match("user@test.com"));
}

#[test]
fn test_german_compound_splitter_with_min_length_boundaries() {
    let splitter = GermanCompoundSplitter::with_min_length(2);
    assert_eq!(splitter.min_component_len(), 2);

    let splitter_4 = GermanCompoundSplitter::with_min_length(4);
    assert_eq!(splitter_4.min_component_len(), 4);
}

#[test]
fn test_german_compound_splitter_extend_vocabulary() {
    let mut splitter = GermanCompoundSplitter::with_min_length(3);
    splitter.extend_vocabulary(["customstem", "customroot"]);
    let decomp = splitter.decompose("customstemcustomroot");
    assert_eq!(decomp, vec!["customstem", "customroot"]);
}

#[test]
fn test_german_compound_splitter_language_code() {
    let splitter = GermanCompoundSplitter::new();
    assert_eq!(splitter.language(), "de");
}

#[tokio::test]
async fn test_collection_purge_report_and_reloaded_search() -> Result<()> {
    let storage = Arc::new(SimpleMockStore::new());
    let index = Arc::new(InvertedIndex::new(storage.clone(), "col_purge_test"));

    let tx = TxId::new(1);
    index
        .upsert_document(tx, DocId::from(1u64), "collection purge document one")
        .await?;
    index
        .upsert_document(tx, DocId::from(2u64), "collection purge document two")
        .await?;
    index.commit(tx).await?;

    let purger = TextPurger::new(index.clone());
    let report = purger.purge_collection_report(tx, "col_purge_test");
    assert!(report.is_success());
    assert!(report.units_purged > 0);

    let reloaded = Arc::new(InvertedIndex::new(storage.clone(), "col_purge_test"));
    assert!(reloaded.search("collection", 10).await?.is_empty());

    Ok(())
}
