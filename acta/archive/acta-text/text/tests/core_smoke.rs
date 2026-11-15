//! Rauchtest für den autarken Kern ohne `workspace` oder Ports-Abhängigkeiten (Gate G-A).
// ANCHOR[acta-text:CORE-SMOKE-001]

use bytes::Bytes;
use acta_ids::{DocId, TxId, TOMBSTONE_BIT};
use acta_text::error::Result;
use acta_text::inverted::InvertedIndex;
use acta_text::posting_list::ResidentPostingIndex;
use acta_text::storage::{BoxFuture, PostingStore};
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

struct InMemoryPostingStore {
    data: RwLock<BTreeMap<Vec<u8>, Vec<(Vec<u8>, u64)>>>,
    next_seq: AtomicU64,
}

impl InMemoryPostingStore {
    fn new() -> Self {
        Self {
            data: RwLock::new(BTreeMap::new()),
            next_seq: AtomicU64::new(1),
        }
    }
}

impl PostingStore for InMemoryPostingStore {
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<Bytes>>> {
        Box::pin(async move { self.get_at_seq(key, u64::MAX).await })
    }

    fn get_at_seq<'a>(&'a self, key: &'a [u8], seq: u64) -> BoxFuture<'a, Result<Option<Bytes>>> {
        Box::pin(async move {
            let guard = self.data.read();
            if let Some(versions) = guard.get(key) {
                for (val, v_seq) in versions.iter().rev() {
                    let raw_seq = v_seq & !TOMBSTONE_BIT;
                    if raw_seq <= seq {
                        if (v_seq & TOMBSTONE_BIT) != 0 {
                            return Ok(None);
                        }
                        return Ok(Some(Bytes::from(val.clone())));
                    }
                }
            }
            Ok(None)
        })
    }

    fn put<'a>(&'a self, _tx: TxId, key: &'a [u8], value: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            let seq = self.next_seq.fetch_add(1, Ordering::SeqCst);
            self.data
                .write()
                .entry(key.to_vec())
                .or_default()
                .push((value.to_vec(), seq));
            Ok(())
        })
    }

    fn delete<'a>(&'a self, _tx: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            let seq = self.next_seq.fetch_add(1, Ordering::SeqCst);
            self.data
                .write()
                .entry(key.to_vec())
                .or_default()
                .push((Vec::new(), seq | TOMBSTONE_BIT));
            Ok(())
        })
    }

    fn commit<'a>(&'a self, _tx: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn rollback<'a>(&'a self, _tx: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn rollback_to_tx<'a>(&'a self, _tx_id: TxId) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move { Ok(()) })
    }

    fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<TxId>> {
        Box::pin(async move { Ok(TxId::new(1)) })
    }

    fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, Result<u64>> {
        Box::pin(async move { Ok(self.next_seq.load(Ordering::SeqCst).saturating_sub(1)) })
    }

    fn scan_prefix<'a>(&'a self, prefix: &'a [u8]) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move { self.scan_prefix_at(prefix, u64::MAX).await })
    }

    fn scan_prefix_at<'a>(
        &'a self,
        prefix: &'a [u8],
        seq: u64,
    ) -> BoxFuture<'a, Result<Vec<(Vec<u8>, Vec<u8>)>>> {
        Box::pin(async move {
            let guard = self.data.read();
            let mut res = Vec::new();
            for (k, versions) in guard.iter() {
                if k.starts_with(prefix) {
                    for (val, v_seq) in versions.iter().rev() {
                        let raw_seq = v_seq & !TOMBSTONE_BIT;
                        if raw_seq <= seq {
                            if (v_seq & TOMBSTONE_BIT) == 0 {
                                res.push((k.clone(), val.clone()));
                            }
                            break;
                        }
                    }
                }
            }
            Ok(res)
        })
    }
}

#[tokio::test]
async fn core_smoke() -> Result<()> {
    let store = Arc::new(InMemoryPostingStore::new());
    let index = InvertedIndex::new(store.clone(), "smoke_test");
    let tx = TxId::new(1);

    // 1. Dokumente einfügen
    index
        .upsert_document(tx, DocId::new(1), "rust high performance text search engine")
        .await?;
    index
        .upsert_document(tx, DocId::new(2), "rust systems programming language")
        .await?;
    index
        .upsert_document(tx, DocId::new(3), "python machine learning and data science")
        .await?;

    index.commit_stats(tx).await?;
    store.commit(tx).await?;

    // 2. BM25-Suche ausführen (ruft intern block_max_wand_search auf)
    let results = index.search_bm25("rust search", 10, None).await?;
    assert!(!results.is_empty());
    assert_eq!(results[0].0, DocId::new(1));

    // 3. Explizite WAND-Traversal-Funktion direkt aufrufen
    let resident = ResidentPostingIndex::new();
    let wand_res = acta_text::wand::block_max_wand_search(
        store.as_ref(),
        &["rust".to_string(), "engine".to_string()],
        &resident,
        |term| {
            let mut k = b"__txt:smoke_test:pl:".to_vec();
            k.extend_from_slice(term.as_bytes());
            k.push(b':');
            k
        },
        |doc_id, term| {
            format!("__txt:smoke_test:tbs:{}:{}", doc_id.inner(), term).into_bytes()
        },
        |doc_id_inner| {
            format!("__txt:smoke_test:dl:{}", doc_id_inner).into_bytes()
        },
        5,
        None,
        3,
        1.0,
    )
    .await?;

    assert_eq!(wand_res.results.len(), 2);
    assert_eq!(wand_res.results[0].0, DocId::new(1));

    Ok(())
}
