//! SD-05-TEXT-002 — Integration tests for BM25 Candidate Stream (`Bm25CandidateStream`).
//!
//! Verifies pull-stream batching, geometric reloading, deduplication, prefix ordering,
//! snapshot isolation, and laziness using `MockStorage`.

use contextra_types::{DocId, Result, TxId, MAX_SEARCH_K};
use contextra_ports::{BoxFuture, StorageEngine, TextIndex};
use contextra_text::{Bm25CandidateStream, InvertedIndex, DEFAULT_STREAM_BATCH_SIZE};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

type MockVersion = (u64, Option<Vec<u8>>);
type MockVersionMap = HashMap<Vec<u8>, Vec<MockVersion>>;

struct MockStorage {
    store: RwLock<MockVersionMap>,
    seq_counter: AtomicU64,
    scan_prefix_calls: AtomicU64,
}

impl MockStorage {
    fn new() -> Self {
        Self {
            store: RwLock::new(HashMap::new()),
            seq_counter: AtomicU64::new(1),
            scan_prefix_calls: AtomicU64::new(0),
        }
    }

    fn scan_prefix_count(&self) -> u64 {
        self.scan_prefix_calls.load(Ordering::Relaxed)
    }
}

impl StorageEngine for MockStorage {
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
        Box::pin(async move { self.get_at_seq(key, u64::MAX).await })
    }

    fn put<'a>(&'a self, _tx: TxId, key: &'a [u8], value: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            let seq = self.seq_counter.fetch_add(1, Ordering::SeqCst);
            self.store
                .write()
                .entry(key.to_vec())
                .or_default()
                .push((seq, Some(value.to_vec())));
            Ok(())
        })
    }

    fn delete<'a>(&'a self, _tx: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            let seq = self.seq_counter.fetch_add(1, Ordering::SeqCst);
            self.store
                .write()
                .entry(key.to_vec())
                .or_default()
                .push((seq, None));
            Ok(())
        })
    }

    fn commit<'a>(&'a self, _tx: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn rollback<'a>(&'a self, _tx: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn rollback_to_tx<'a>(&'a self, _tx: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn get_at_seq<'a>(
        &'a self,
        key: &'a [u8],
        seq: u64,
    ) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
        Box::pin(async move {
            let store = self.store.read();
            if let Some(versions) = store.get(key) {
                for (v_seq, val) in versions.iter().rev() {
                    if *v_seq <= seq {
                        return Ok(val.clone().map(bytes::Bytes::from));
                    }
                }
            }
            Ok(None)
        })
    }

    fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, Result<u64>> {
        Box::pin(async move {
            Ok(self
                .seq_counter
                .load(Ordering::SeqCst)
                .saturating_sub(1))
        })
    }

    fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> {
        Box::pin(async move { Ok(TxId::new(0)) })
    }

    fn flush<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn stats<'a>(&'a self) -> BoxFuture<'a, Result<contextra_ports::StorageStats>> {
        Box::pin(async move {
            Ok(contextra_ports::StorageStats {
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
        Box::pin(async move { self.scan_prefix_at(prefix, u64::MAX).await })
    }

    fn scan_prefix_at<'a>(
        &'a self,
        prefix: &'a [u8],
        seq_no: u64,
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move {
            self.scan_prefix_calls.fetch_add(1, Ordering::Relaxed);
            let store = self.store.read();
            let mut results = Vec::new();
            for (k, versions) in store.iter() {
                if k.starts_with(prefix) {
                    for (v_seq, val) in versions.iter().rev() {
                        if *v_seq <= seq_no {
                            if let Some(v) = val {
                                results.push((k.clone(), v.clone()));
                            }
                            break;
                        }
                    }
                }
            }
            Ok(results)
        })
    }
}

async fn create_test_index_with_docs(
    count: usize,
) -> Result<(Arc<InvertedIndex<MockStorage>>, Arc<MockStorage>)> {
    let storage = Arc::new(MockStorage::new());
    let index = Arc::new(InvertedIndex::new(storage.clone(), "stream_test"));

    let tx = TxId::new(1);
    for i in 1..=count {
        let doc_id = DocId::from(i as u64);
        let text = format!("rust database candidate search benchmark item {i}");
        index
            .upsert_document(tx, doc_id, &text)
            .await?;
    }
    index.commit(tx).await?;

    Ok((index, storage))
}

#[tokio::test]
async fn test_stream_matches_full_search_ordering() -> Result<()> {
    let (index, _) = create_test_index_with_docs(100).await?;
    let query = "rust search benchmark";

    let expected = index.search_bm25_at(query, 100, None).await?;
    let mut stream = Bm25CandidateStream::new(&index, query, None);

    let mut streamed = Vec::new();
    loop {
        let batch = stream.next_batch().await?;
        if batch.is_empty() {
            break;
        }
        streamed.extend(batch);
    }

    assert_eq!(streamed, expected);
    assert_eq!(stream.yielded(), expected.len());
    assert!(stream.is_exhausted());
    Ok(())
}

#[tokio::test]
async fn test_stream_batching_and_exact_sizes() -> Result<()> {
    let (index, _) = create_test_index_with_docs(100).await?;
    let mut stream = Bm25CandidateStream::new(&index, "database item", None);

    assert_eq!(DEFAULT_STREAM_BATCH_SIZE, 16);

    let b1 = stream.next_batch().await?;
    assert_eq!(b1.len(), 16);

    let b2 = stream.next_batch().await?;
    assert_eq!(b2.len(), 16);

    let mut total_yielded = b1.len() + b2.len();
    while let Ok(batch) = stream.next_batch().await {
        if batch.is_empty() {
            break;
        }
        total_yielded += batch.len();
    }

    assert_eq!(total_yielded, 100);
    assert_eq!(stream.yielded(), 100);
    assert!(stream.is_exhausted());

    let empty_batch = stream.next_batch().await?;
    assert!(empty_batch.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_stream_deduplication_across_reloads() -> Result<()> {
    let (index, _) = create_test_index_with_docs(60).await?;
    let mut stream =
        Bm25CandidateStream::new(&index, "candidate search", None).with_batch_size(10);

    let mut seen_ids = std::collections::HashSet::new();
    let mut total_count = 0;

    while let Some(doc_id) = stream.next_doc().await? {
        assert!(seen_ids.insert(doc_id), "Duplicate DocId yielded");
        total_count += 1;
    }

    assert_eq!(total_count, 60);
    assert_eq!(stream.yielded(), 60);
    assert!(stream.is_exhausted());
    Ok(())
}

#[tokio::test]
async fn test_stream_laziness_measurement() -> Result<()> {
    let (index, storage) = create_test_index_with_docs(120).await?;
    let query = "benchmark item";

    let scan_before = storage.scan_prefix_count();

    let mut stream = Bm25CandidateStream::new(&index, query, None);
    let first_batch = stream.next_batch().await?;
    assert_eq!(first_batch.len(), 16);

    let scan_after_b1 = storage.scan_prefix_count();
    let calls_for_batch_1 = scan_after_b1 - scan_before;

    while let Ok(batch) = stream.next_batch().await {
        if batch.is_empty() {
            break;
        }
    }

    let scan_after_full = storage.scan_prefix_count();
    let total_calls = scan_after_full - scan_before;

    assert!(
        calls_for_batch_1 < total_calls,
        "Fetching only first batch ({calls_for_batch_1}) must perform fewer scans than full stream ({total_calls})"
    );

    Ok(())
}

#[tokio::test]
async fn test_stream_depth_limit_max_search_k() -> Result<()> {
    let (index, _) = create_test_index_with_docs(1050).await?;
    let query = "candidate search";

    assert_eq!(Bm25CandidateStream::<MockStorage>::max_depth(), 1000);

    let mut stream = Bm25CandidateStream::new(&index, query, None).with_batch_size(200);

    let mut total_yielded = 0;
    while let Ok(batch) = stream.next_batch().await {
        if batch.is_empty() {
            break;
        }
        total_yielded += batch.len();
    }

    assert_eq!(
        total_yielded, MAX_SEARCH_K,
        "Stream must cap results at MAX_SEARCH_K (1,000)"
    );
    assert_eq!(stream.yielded(), MAX_SEARCH_K);
    assert!(stream.is_exhausted());

    Ok(())
}

#[tokio::test]
async fn test_stream_snapshot_isolation() -> Result<()> {
    let (index, storage) = create_test_index_with_docs(30).await?;
    let query = "database item";

    let snapshot_seq = storage.last_seq_no().await?;
    let mut stream = Bm25CandidateStream::new(&index, query, Some(snapshot_seq)).with_batch_size(10);

    let batch1 = stream.next_batch().await?;
    assert_eq!(batch1.len(), 10);

    // Insert new document matching query after stream start
    let tx2 = TxId::new(2);
    let new_doc = DocId::new(999);
    index
        .upsert_document(tx2, new_doc, "database item fresh insert")
        .await?;
    index.commit(tx2).await?;

    let mut remaining = Vec::new();
    while let Ok(batch) = stream.next_batch().await {
        if batch.is_empty() {
            break;
        }
        remaining.extend(batch);
    }

    let all_streamed_ids: std::collections::HashSet<_> = batch1
        .into_iter()
        .chain(remaining)
        .map(|(doc_id, _)| doc_id)
        .collect();

    assert!(
        !all_streamed_ids.contains(&new_doc),
        "Document inserted after snapshot_seq must not appear in stream"
    );
    assert_eq!(all_streamed_ids.len(), 30);

    Ok(())
}

#[tokio::test]
async fn test_stream_empty_query_and_no_hits() -> Result<()> {
    let (index, _) = create_test_index_with_docs(10).await?;

    let mut empty_stream = Bm25CandidateStream::new(&index, "", None);
    assert!(empty_stream.next_batch().await?.is_empty());
    assert!(empty_stream.is_exhausted());

    let mut no_hits_stream = Bm25CandidateStream::new(&index, "nonexistent_token_xyz", None);
    assert!(no_hits_stream.next_batch().await?.is_empty());
    assert!(no_hits_stream.is_exhausted());

    Ok(())
}
