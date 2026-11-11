//! Standalone-Beispiel für `contextra-text` (SPEC §6, Gate G-F).
//!
//! Zeigt die Initialisierung und Volltextsuche mit einem lokalen `PostingStore`
//! ohne Abhängigkeit von `contextra-types`, `contextra-ports` oder `contextra-db`.

use bytes::Bytes;
use contextra_ids::{DocId, TxId, TOMBSTONE_BIT};
use contextra_text::error::Result;
use contextra_text::inverted::InvertedIndex;
use contextra_text::storage::{BoxFuture, PostingStore};
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

struct SimpleStore {
    map: RwLock<BTreeMap<Vec<u8>, Vec<(Vec<u8>, u64)>>>,
    seq: AtomicU64,
}

impl SimpleStore {
    fn new() -> Self {
        Self {
            map: RwLock::new(BTreeMap::new()),
            seq: AtomicU64::new(1),
        }
    }
}

impl PostingStore for SimpleStore {
    fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<Bytes>>> {
        Box::pin(async move { self.get_at_seq(key, u64::MAX).await })
    }

    fn get_at_seq<'a>(&'a self, key: &'a [u8], seq: u64) -> BoxFuture<'a, Result<Option<Bytes>>> {
        Box::pin(async move {
            let r = self.map.read();
            if let Some(versions) = r.get(key) {
                for (v, s) in versions.iter().rev() {
                    let raw = s & !TOMBSTONE_BIT;
                    if raw <= seq {
                        if (s & TOMBSTONE_BIT) != 0 {
                            return Ok(None);
                        }
                        return Ok(Some(Bytes::from(v.clone())));
                    }
                }
            }
            Ok(None)
        })
    }

    fn put<'a>(&'a self, _tx: TxId, key: &'a [u8], value: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            let s = self.seq.fetch_add(1, Ordering::SeqCst);
            self.map
                .write()
                .entry(key.to_vec())
                .or_default()
                .push((value.to_vec(), s));
            Ok(())
        })
    }

    fn delete<'a>(&'a self, _tx: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            let s = self.seq.fetch_add(1, Ordering::SeqCst);
            self.map
                .write()
                .entry(key.to_vec())
                .or_default()
                .push((Vec::new(), s | TOMBSTONE_BIT));
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
        Box::pin(async move { Ok(self.seq.load(Ordering::SeqCst).saturating_sub(1)) })
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
            let r = self.map.read();
            let mut out = Vec::new();
            for (k, versions) in r.iter() {
                if k.starts_with(prefix) {
                    for (v, s) in versions.iter().rev() {
                        let raw = s & !TOMBSTONE_BIT;
                        if raw <= seq {
                            if (s & TOMBSTONE_BIT) == 0 {
                                out.push((k.clone(), v.clone()));
                            }
                            break;
                        }
                    }
                }
            }
            Ok(out)
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let store = Arc::new(SimpleStore::new());
    let index = InvertedIndex::new(store.clone(), "standalone_example");
    let tx = TxId::new(1);

    index
        .upsert_document(tx, DocId::new(10), "Contextra modular hybrid search")
        .await?;
    index.commit_stats(tx).await?;

    let hits = index.search_bm25("hybrid search", 5, None).await?;
    println!("Found hits: {:?}", hits);
    assert!(!hits.is_empty());
    Ok(())
}
