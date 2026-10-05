use contextra_ports::{BoxFuture, StorageEngine, StorageStats, TextIndex};
use contextra_text::morphology::MorphologicalTokenizer;
use contextra_text::{GermanCompoundSplitter, InvertedIndex, Posting, ResidentPostingIndex};
use contextra_types::{DocId, Result, TxId};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::Mutex;

struct MockStorage {
    data: Mutex<HashMap<Vec<u8>, Vec<u8>>>,
}

impl MockStorage {
    fn new() -> Self {
        Self {
            data: Mutex::new(HashMap::new()),
        }
    }
}

impl StorageEngine for MockStorage {
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
        Box::pin(async move {
            Ok(self
                .data
                .lock()
                .get(key)
                .cloned()
                .map(bytes::Bytes::from))
        })
    }

    fn put<'a>(
        &'a self,
        _tx_id: TxId,
        key: &'a [u8],
        value: &'a [u8],
    ) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.data
                .lock()
                .insert(key.to_vec(), value.to_vec());
            Ok(())
        })
    }

    fn delete<'a>(&'a self, _tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            self.data.lock().remove(key);
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
        Box::pin(async move { Ok(1) })
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
            let data = self.data.lock();
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

#[test]
fn test_compound_splitter_new_with_all_domains_and_extend_vocabulary() {
    let mut splitter = GermanCompoundSplitter::new_with_all_domains();

    // Verify domain compound decomposition loaded via new_with_all_domains
    let decomp = splitter.decompose("bundesverfassungsgerichtsentscheidung");
    assert!(
        decomp.len() >= 2,
        "new_with_all_domains should load legal terms for decomposition, got: {:?}",
        decomp
    );

    // Dynamically extend vocabulary with a custom KMU / domain word
    let custom_words = vec!["specializedkmu", "customdomainterm"];
    splitter.extend_vocabulary(custom_words);

    let custom_decomp = splitter.decompose("specializedkmucustomdomainterm");
    assert!(
        custom_decomp.contains(&"specializedkmu") && custom_decomp.contains(&"customdomainterm"),
        "extend_vocabulary should add terms to trie enabling decomposition, got: {:?}",
        custom_decomp
    );
}

#[test]
fn test_resident_posting_index_remove_terms_direct_and_delegation() {
    let index = ResidentPostingIndex::new();
    let doc1 = DocId::from(101u64);
    let doc2 = DocId::from(102u64);
    let term = "contextra".to_string();

    index.upsert_posting(&term, Posting::new(doc1, 2, 20));
    index.upsert_posting(&term, Posting::new(doc2, 1, 15));

    let list_before = index.get(&term).expect("term contextra should exist");
    assert_eq!(list_before.len(), 2);

    // Test remove_terms directly
    index.remove_terms(doc1, &[term.clone()]);

    let list_after_doc1 = index.get(&term).expect("term contextra should remain for doc2");
    assert_eq!(list_after_doc1.len(), 1);
    assert_eq!(list_after_doc1.as_slice()[0].doc_id(), doc2);

    // Test remove_posting_from_terms delegation to remove_terms
    index.remove_posting_from_terms(&[term.clone()], doc2);
    assert!(
        index.get(&term).is_none(),
        "removing all documents should prune term entry from resident index"
    );
}

#[tokio::test]
async fn test_inverted_index_deletion_uses_remove_terms() -> Result<()> {
    let storage = Arc::new(MockStorage::new());
    let index = InvertedIndex::new(storage, "j31_test");
    let tx = TxId::new(1);
    let doc_id = DocId::from(200u64);

    index.insert(tx, doc_id, "contextra search engine").await?;
    index.commit(tx).await?;

    let search_res = index.search("contextra", 10).await?;
    assert_eq!(search_res.len(), 1);

    // Delete document which triggers remove_posting_from_terms -> remove_terms
    index.delete(tx, doc_id).await?;
    index.commit(tx).await?;

    let search_after_del = index.search("contextra", 10).await?;
    assert_eq!(search_after_del.len(), 0);

    Ok(())
}
