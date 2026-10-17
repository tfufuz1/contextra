// FILE-CONTEXT: Ring-3 Strangulation Facade Wiring Entry Point for contextra-text.
// ZWECK: Stellt den kanonischen Verkabelungs-Einstiegstyp TextWiring für contextra/src/builder.rs bereit.

use crate::inverted::{InvertedIndex, Language};
use contextra_ports::StorageEngine;
use contextra_types::Result;
use std::sync::Arc;

/// Facade wiring entry point for initializing a transactional text index.
pub struct TextWiring<S: StorageEngine> {
    pub index: InvertedIndex<S>,
}

impl<S: StorageEngine> TextWiring<S> {
    /// Builds a new [`TextWiring`] instance wrapping [`InvertedIndex`].
    pub fn build(storage: Arc<S>, namespace: &str, language: Language) -> Result<Self> {
        let index = InvertedIndex::new_with_language(storage, namespace, language);
        Ok(Self { index })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use contextra_ports::BoxFuture;
    use contextra_ports::StorageStats;
    use contextra_types::TxId;
    use std::collections::HashMap;
    use std::sync::Mutex;

    struct DummyStorage {
        data: Mutex<HashMap<Vec<u8>, Vec<u8>>>,
    }

    impl StorageEngine for DummyStorage {
        fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
            Box::pin(async move {
                Ok(self
                    .data
                    .lock()
                    .unwrap()
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
                    .unwrap()
                    .insert(key.to_vec(), value.to_vec());
                Ok(())
            })
        }

        fn delete<'a>(&'a self, _tx_id: TxId, key: &'a [u8]) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move {
                self.data.lock().unwrap().remove(key);
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
                let data = self.data.lock().unwrap();
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
    fn test_text_wiring_build() {
        let storage = Arc::new(DummyStorage {
            data: Mutex::new(HashMap::new()),
        });
        let wiring = TextWiring::build(storage, "wiring_ns", Language::German);
        assert!(wiring.is_ok());
    }
}
