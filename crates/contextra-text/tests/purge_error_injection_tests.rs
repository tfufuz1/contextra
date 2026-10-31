//! Testsuite `purge_error_injection_tests.rs` für contextra-text.
//!
//! # Ring-Zuordnung
//! Ring LAB / Testbereich.

use contextra_ports::{BoxFuture, StorageEngine, StorageStats, TextIndex};
use contextra_text::{InvertedIndex, TextPurger};
use contextra_types::{ContextraError, DocId, Result, TxId};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

struct FaultyStorage {
    data: RwLock<HashMap<Vec<u8>, Vec<u8>>>,
    fail_on_delete: RwLock<bool>,
    next_seq: AtomicU64,
}

impl FaultyStorage {
    fn new() -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
            fail_on_delete: RwLock::new(false),
            next_seq: AtomicU64::new(1),
        }
    }

    fn inject_corrupt_key(&self, key: &[u8], corrupt_bytes: &[u8]) {
        self.data
            .write()
            .insert(key.to_vec(), corrupt_bytes.to_vec());
    }

    fn set_fail_on_delete(&self, fail: bool) {
        *self.fail_on_delete.write() = fail;
    }
}

impl StorageEngine for FaultyStorage {
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
            if *self.fail_on_delete.read() {
                return Err(ContextraError::Storage(format!(
                    "Injected storage delete error for key {:?}",
                    String::from_utf8_lossy(key)
                )));
            }
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

// ANCHOR[contextra-text:TEST-185]
// Testfall `test_purge_report_success_on_clean_document`: Prüft das Verhalten und Invarianten der Textverarbeitung.
#[tokio::test]
async fn test_purge_report_success_on_clean_document() -> Result<()> {
    let storage = Arc::new(FaultyStorage::new());
    let index = Arc::new(InvertedIndex::new(storage.clone(), "test_purge"));

    let tx = TxId::new(1);
    let doc_id = DocId::from(100u64);

    index
        .upsert_document(tx, doc_id, "rust search engine")
        .await?;
    index.commit(tx).await?;

    let purger = TextPurger::new(index);
    let report = purger.purge_document_report(tx, doc_id);

    assert!(report.is_success());
    assert_eq!(report.units_failed, 0);
    assert!(report.errors.is_empty());

    let stats = report.to_purge_stats()?;
    assert!(stats.purged_units > 0);

    Ok(())
}

// ANCHOR[contextra-text:TEST-186]
// Testfall `test_purge_report_catches_corrupted_posting_list_bytes`: Prüft das Verhalten und Invarianten der Textverarbeitung.
#[tokio::test]
async fn test_purge_report_catches_corrupted_posting_list_bytes() -> Result<()> {
    let storage = Arc::new(FaultyStorage::new());
    let index = Arc::new(InvertedIndex::new(storage.clone(), "test_purge_corrupt"));

    let tx = TxId::new(1);
    let doc_id = DocId::from(200u64);

    index
        .upsert_document(tx, doc_id, "corrupted term test")
        .await?;
    index.commit(tx).await?;

    // Inject corrupted bytes into batch posting list `plb:corrupted`
    let plb_key = format!("__txt:{}:plb:corrupted", "test_purge_corrupt").into_bytes();
    storage.inject_corrupt_key(&plb_key, b"PL\x02\x00\xff\xff\xff\xff");

    let purger = TextPurger::new(index);
    let report = purger.purge_document_report(tx, doc_id);

    assert!(!report.is_success());
    assert!(report.units_failed > 0);
    assert!(report
        .errors
        .iter()
        .any(|e| e.contains("Corrupted batch posting list")));

    // Verifies that to_purge_stats() fails closed and NEVER returns success!
    let stats_res = report.to_purge_stats();
    assert!(stats_res.is_err());
    if let Err(ContextraError::Text(msg)) = stats_res {
        assert!(msg.contains("Purge incomplete"));
    } else {
        panic!("Expected ContextraError::Text");
    }

    Ok(())
}

// ANCHOR[contextra-text:TEST-187]
// Testfall `test_purge_report_catches_storage_deletion_failure`: Prüft das Verhalten und Invarianten der Textverarbeitung.
#[tokio::test]
async fn test_purge_report_catches_storage_deletion_failure() -> Result<()> {
    let storage = Arc::new(FaultyStorage::new());
    let index = Arc::new(InvertedIndex::new(
        storage.clone(),
        "test_purge_storage_err",
    ));

    let tx = TxId::new(1);
    let doc_id = DocId::from(300u64);

    index
        .upsert_document(tx, doc_id, "failing storage delete")
        .await?;
    index.commit(tx).await?;

    storage.set_fail_on_delete(true);

    let purger = TextPurger::new(index);
    let report = purger.purge_document_report(tx, doc_id);

    assert!(!report.is_success());
    assert!(report.units_failed > 0);
    assert!(report
        .errors
        .iter()
        .any(|e| e.contains("Injected storage delete error")));

    assert!(purger.purge_document(tx, doc_id).is_err());

    Ok(())
}
