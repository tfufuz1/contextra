// FILE-CONTEXT: TextIndex Physical Purger & BM25 Corpus Statistics Recalculation (§12.7.4).
// ZWECK: Stellt den Purger für physical deletion per document and per collection bereit.
// INVARIANTEN: Atomar gegenüber Lesern; entfernt leere Term-Dictionary-Einträge; korrigiert BM25-Statistik.
// NICHT-OFFENSICHTLICH: Document purge purges postings, dl, fw, tbs, empties plb keys and updates atomic stats.
// HOTSPOTS: purge_document_internal, purge_collection_internal

use crate::inverted::index_struct::InvertedIndex;
use crate::inverted::scanner::ParsedTextTarget;
use crate::inverted::types::TextIndexMetadata;
use crate::posting_list::PostingList;
use contextra_ports::scanner::{PurgeStats, Purger, ScanTarget};
use contextra_ports::StorageEngine;
use contextra_types::{ContextraError, DocId, RequestContext, Result, TxId};
use std::collections::HashSet;
use std::sync::atomic::Ordering;
use std::sync::Arc;

/// Executes an async block safely inside synchronous purger methods.
fn block_on_async<F: std::future::Future + Send + 'static>(f: F) -> F::Output
where
    F::Output: Send + 'static,
{
    match tokio::runtime::Handle::try_current() {
        Ok(handle) => {
            if handle.runtime_flavor() == tokio::runtime::RuntimeFlavor::MultiThread {
                tokio::task::block_in_place(|| handle.block_on(f))
            } else {
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .expect("Failed to build temporary Tokio runtime");
                    rt.block_on(f)
                })
                .join()
                .expect("Thread panicked while executing block_on_async")
            }
        }
        Err(_) => {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to build temporary Tokio runtime");
            rt.block_on(f)
        }
    }
}

/// Physical Purger implementation for `InvertedIndex`.
pub struct TextPurger<S: StorageEngine + 'static> {
    index: Arc<InvertedIndex<S>>,
}

impl<S: StorageEngine + 'static> TextPurger<S> {
    /// Creates a new `TextPurger` for the given index.
    pub fn new(index: Arc<InvertedIndex<S>>) -> Self {
        Self { index }
    }

    /// Purges a single document from the inverted index and recalculates corpus statistics.
    pub fn purge_document(&self, tx: TxId, doc_id: DocId) -> Result<PurgeStats> {
        let _guard = self.index.commit_lock().lock();
        self.purge_document_internal(tx, doc_id)
    }

    /// Internal document purge logic executed under commit_lock.
    fn purge_document_internal(&self, tx: TxId, doc_id: DocId) -> Result<PurgeStats> {
        let mut purged_units = 0u64;
        let mut bytes_freed = 0u64;

        // 1. Read and delete document length key `dl:{doc_id}`
        let dl_key = self.index.key_with_id("dl:", doc_id.inner());
        let mut doc_len = 0u32;
        let storage = self.index.storage().clone();
        let dl_key_c = dl_key.clone();
        let dl_bytes_opt = block_on_async(async move { storage.get(&dl_key_c).await })?;

        if let Some(ref dl_bytes) = dl_bytes_opt {
            if dl_bytes.len() == 4 {
                doc_len = u32::from_le_bytes(
                    (&dl_bytes[..])
                        .try_into()
                        .map_err(|_| ContextraError::Storage("Invalid doc_len bytes".into()))?,
                );
            }
            bytes_freed = bytes_freed.saturating_add((dl_key.len() + dl_bytes.len()) as u64);
            purged_units = purged_units.saturating_add(1);
            let storage = self.index.storage().clone();
            let dl_key_c = dl_key.clone();
            block_on_async(async move { storage.delete(tx, &dl_key_c).await })?;
        }

        // 2. Read forward index `fw:{doc_id}` to identify terms
        let fw_key = self.index.key_with_id("fw:", doc_id.inner());
        let mut terms_to_check: HashSet<String> = HashSet::new();

        let storage = self.index.storage().clone();
        let fw_key_c = fw_key.clone();
        let fw_bytes_opt = block_on_async(async move { storage.get(&fw_key_c).await })?;

        if let Some(ref fw_bytes) = fw_bytes_opt {
            if let Ok(terms) = bincode::deserialize::<Vec<String>>(fw_bytes) {
                terms_to_check.extend(terms);
            }
            bytes_freed = bytes_freed.saturating_add((fw_key.len() + fw_bytes.len()) as u64);
            purged_units = purged_units.saturating_add(1);
            let storage = self.index.storage().clone();
            let fw_key_c = fw_key.clone();
            block_on_async(async move { storage.delete(tx, &fw_key_c).await })?;
        }

        // Fallback prefix scan for any extra individual posting or tombstone keys for this doc
        let doc_suffix = format!(":{}", doc_id.inner()).into_bytes();
        let doc_infix = format!(":{}:", doc_id.inner()).into_bytes();

        let storage = self.index.storage().clone();
        let prefix = self.index.prefix().to_vec();
        let raw_prefix_entries = block_on_async(async move { storage.scan_prefix(&prefix).await })?;

        for (k, v) in &raw_prefix_entries {
            if k.ends_with(&doc_suffix) || k.windows(doc_infix.len()).any(|w| w == doc_infix) {
                if k.starts_with(&self.index.key("pl:")) {
                    let pl_prefix = self.index.key("pl:");
                    let suffix = &k[pl_prefix.len()..];
                    if let Some(colon_idx) = suffix.iter().position(|&b| b == b':') {
                        if let Ok(term) = std::str::from_utf8(&suffix[..colon_idx]) {
                            terms_to_check.insert(term.to_string());
                        }
                    }
                }
                bytes_freed = bytes_freed.saturating_add((k.len() + v.len()) as u64);
                purged_units = purged_units.saturating_add(1);
                let storage = self.index.storage().clone();
                let k_c = k.clone();
                block_on_async(async move { storage.delete(tx, &k_c).await })?;
            }
        }

        // 3. Purge document postings from terms and batch posting lists
        for term in &terms_to_check {
            // Load current posting list for term
            let plb_key = self.index.key_batch_posting_list(term);
            let storage = self.index.storage().clone();
            let plb_key_c = plb_key.clone();
            let plb_bytes_opt = block_on_async(async move { storage.get(&plb_key_c).await })?;

            let plist_opt = match plb_bytes_opt {
                Some(ref bytes) => PostingList::decode_compact(bytes).ok(),
                None => None,
            };

            let updated_plist = match plist_opt {
                Some(plist) => plist.remove(doc_id),
                None => PostingList::empty(),
            };

            if updated_plist.is_empty() {
                // Term is now empty: purge batch posting key and remove from dictionary
                if let Some(ref bytes) = plb_bytes_opt {
                    bytes_freed = bytes_freed.saturating_add((plb_key.len() + bytes.len()) as u64);
                    purged_units = purged_units.saturating_add(1);
                }
                let storage = self.index.storage().clone();
                let plb_key_c = plb_key.clone();
                block_on_async(async move { storage.delete(tx, &plb_key_c).await })?;
                self.index
                    .resident_index
                    .remove_posting_from_terms(std::slice::from_ref(term), doc_id);
                self.index
                    .resident_index
                    .remove_terms(std::slice::from_ref(term));
            } else {
                // Update batch posting key and resident index
                let encoded = updated_plist.encode_compact()?;
                let storage = self.index.storage().clone();
                let plb_key_c = plb_key.clone();
                block_on_async(async move { storage.put(tx, &plb_key_c, &encoded).await })?;
                self.index
                    .resident_index
                    .insert_list(term.clone(), updated_plist);
            }

            // Delete individual posting key `pl:{term}:{doc_id}` and tombstone key `tbs:{doc_id}:{term}`
            let pl_doc_key = self.index.key_with_term_doc(term, doc_id);
            let tbs_key = self.index.key_tombstone(doc_id, term);
            let storage = self.index.storage().clone();
            block_on_async(async move {
                let _ = storage.delete(tx, &pl_doc_key).await;
                let _ = storage.delete(tx, &tbs_key).await;
            });
        }

        // 4. Update BM25 corpus statistics atomically
        if dl_bytes_opt.is_some() {
            let current_docs = self.index.total_docs.load(Ordering::SeqCst);
            let current_tokens = self.index.total_tokens.load(Ordering::SeqCst);

            let new_docs = current_docs.saturating_sub(1);
            let new_tokens = current_tokens.saturating_sub(doc_len as u64);
            let new_avg_len = if new_docs > 0 {
                (new_tokens as f64 / new_docs as f64 * 1000.0) as u64
            } else {
                0
            };

            self.index.total_docs.store(new_docs, Ordering::SeqCst);
            self.index.total_tokens.store(new_tokens, Ordering::SeqCst);
            self.index
                .avg_doc_len_x1000
                .store(new_avg_len, Ordering::SeqCst);

            let meta = TextIndexMetadata {
                total_docs: new_docs,
                total_tokens: new_tokens,
                avg_doc_len_x1000: new_avg_len,
            };
            if let Ok(meta_bytes) = bincode::serialize(&meta) {
                let meta_key = self.index.key("meta:stats");
                let storage = self.index.storage().clone();
                let _ =
                    block_on_async(async move { storage.put(tx, &meta_key, &meta_bytes).await });
            }
        }

        // Commit storage transaction
        let storage = self.index.storage().clone();
        block_on_async(async move { storage.commit(tx).await })?;

        Ok(PurgeStats {
            purged_units,
            bytes_freed,
        })
    }

    /// Fast-path purge for an entire collection namespace (`__txt:{collection}:`).
    pub fn purge_collection(&self, tx: TxId, namespace: &str) -> Result<PurgeStats> {
        let _guard = self.index.commit_lock().lock();

        let prefix = if namespace == "default" {
            b"__txt:default:".to_vec()
        } else if namespace.starts_with("__txt:") {
            namespace.as_bytes().to_vec()
        } else {
            format!("__txt:{}:", namespace).into_bytes()
        };

        let storage = self.index.storage().clone();
        let prefix_c = prefix.clone();
        let raw_entries = block_on_async(async move { storage.scan_prefix(&prefix_c).await })?;

        let mut purged_units = 0u64;
        let mut bytes_freed = 0u64;

        for (k, v) in raw_entries {
            bytes_freed = bytes_freed.saturating_add((k.len() + v.len()) as u64);
            purged_units = purged_units.saturating_add(1);
            let storage = self.index.storage().clone();
            block_on_async(async move { storage.delete(tx, &k).await })?;
        }

        // Reset in-memory statistics and resident index
        self.index.total_docs.store(0, Ordering::SeqCst);
        self.index.total_tokens.store(0, Ordering::SeqCst);
        self.index.avg_doc_len_x1000.store(0, Ordering::SeqCst);
        self.index.staged_stats.lock().clear();
        self.index.staged_terms.lock().clear();
        self.index.resident_index.clear();

        let storage = self.index.storage().clone();
        block_on_async(async move { storage.commit(tx).await })?;

        Ok(PurgeStats {
            purged_units,
            bytes_freed,
        })
    }
}

impl<S: StorageEngine + 'static> Purger for TextPurger<S> {
    fn purge(&self, ctx: &RequestContext, target: &ScanTarget) -> Result<PurgeStats> {
        let parsed = ParsedTextTarget::parse(&target.target_id);

        match parsed {
            ParsedTextTarget::Document(doc_id) => self.purge_document(ctx.tx_id, doc_id),
            ParsedTextTarget::Collection(col_ns) => self.purge_collection(ctx.tx_id, &col_ns),
        }
    }
}

/// Factory function to construct a `Purger` for text index.
pub fn text_purger<S: StorageEngine + 'static>(index: Arc<InvertedIndex<S>>) -> Arc<dyn Purger> {
    Arc::new(TextPurger::new(index))
}
