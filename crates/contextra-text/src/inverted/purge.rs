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
    futures::executor::block_on(f)
}

/// Detailed purge execution report capturing purged units, failed units, bytes freed, and errors.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct PurgeReport {
    /// Number of units successfully purged.
    pub units_purged: u64,
    /// Number of units that failed during purge execution.
    pub units_failed: u64,
    /// Number of storage bytes freed.
    pub bytes_freed: u64,
    /// List of error descriptions encountered during partial failures.
    pub errors: Vec<String>,
}

impl PurgeReport {
    /// Creates a new empty `PurgeReport`.
    pub fn new() -> Self {
        Self {
            units_purged: 0,
            units_failed: 0,
            bytes_freed: 0,
            errors: Vec::new(),
        }
    }

    /// Returns `true` if no errors occurred during purge.
    pub fn is_success(&self) -> bool {
        self.units_failed == 0 && self.errors.is_empty()
    }

    /// Converts this report to `PurgeStats`, or returns an error if partial failure occurred.
    pub fn to_purge_stats(&self) -> Result<PurgeStats> {
        if self.is_success() {
            Ok(PurgeStats {
                purged_units: self.units_purged,
                bytes_freed: self.bytes_freed,
            })
        } else {
            Err(ContextraError::Text(format!(
                "Purge incomplete: {} units failed. Errors: {:?}",
                self.units_failed, self.errors
            )))
        }
    }
}

impl Default for PurgeReport {
    fn default() -> Self {
        Self::new()
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

    /// Purges a single document from the inverted index and recalculates corpus statistics,
    /// returning a detailed `PurgeReport`.
    pub fn purge_document_report(&self, tx: TxId, doc_id: DocId) -> PurgeReport {
        let _guard = self.index.commit_lock().lock();
        self.purge_document_internal_report(tx, doc_id)
    }

    /// Purges a single document from the inverted index and recalculates corpus statistics.
    pub fn purge_document(&self, tx: TxId, doc_id: DocId) -> Result<PurgeStats> {
        let report = self.purge_document_report(tx, doc_id);
        report.to_purge_stats()
    }

    /// Internal document purge logic executed under commit_lock.
    fn purge_document_internal_report(&self, tx: TxId, doc_id: DocId) -> PurgeReport {
        let mut report = PurgeReport::new();

        // 1. Read and delete document length key `dl:{doc_id}`
        let dl_key = self.index.key_with_id("dl:", doc_id.inner());
        let mut doc_len = 0u32;
        let storage = self.index.storage().clone();
        let dl_key_c = dl_key.clone();
        let dl_bytes_res = block_on_async(async move { storage.get(&dl_key_c).await });

        let dl_bytes_opt = match dl_bytes_res {
            Ok(opt) => opt,
            Err(e) => {
                report.units_failed += 1;
                report
                    .errors
                    .push(format!("Failed to read doc_len key: {e}"));
                None
            }
        };

        if let Some(ref dl_bytes) = dl_bytes_opt {
            if dl_bytes.len() == 4 {
                match (&dl_bytes[..]).try_into() {
                    Ok(arr) => doc_len = u32::from_le_bytes(arr),
                    Err(_) => {
                        report.units_failed += 1;
                        report
                            .errors
                            .push("Invalid doc_len byte representation".into());
                    }
                }
            }
            report.bytes_freed = report
                .bytes_freed
                .saturating_add((dl_key.len() + dl_bytes.len()) as u64);
            report.units_purged = report.units_purged.saturating_add(1);

            let storage = self.index.storage().clone();
            let dl_key_c = dl_key.clone();
            if let Err(e) = block_on_async(async move { storage.delete(tx, &dl_key_c).await }) {
                report.units_failed += 1;
                report
                    .errors
                    .push(format!("Failed to delete doc_len key: {e}"));
            }
        }

        // 2. Read forward index `fw:{doc_id}` to identify terms
        let fw_key = self.index.key_with_id("fw:", doc_id.inner());
        let mut terms_to_check: HashSet<String> = HashSet::new();

        let storage = self.index.storage().clone();
        let fw_key_c = fw_key.clone();
        let fw_bytes_res = block_on_async(async move { storage.get(&fw_key_c).await });

        let fw_bytes_opt = match fw_bytes_res {
            Ok(opt) => opt,
            Err(e) => {
                report.units_failed += 1;
                report
                    .errors
                    .push(format!("Failed to read forward index key: {e}"));
                None
            }
        };

        if let Some(ref fw_bytes) = fw_bytes_opt {
            match bincode::deserialize::<Vec<String>>(fw_bytes) {
                Ok(terms) => terms_to_check.extend(terms),
                Err(e) => {
                    report.units_failed += 1;
                    report.errors.push(format!(
                        "Corrupted forward index entry for doc {doc_id:?}: {e}"
                    ));
                }
            }
            report.bytes_freed = report
                .bytes_freed
                .saturating_add((fw_key.len() + fw_bytes.len()) as u64);
            report.units_purged = report.units_purged.saturating_add(1);

            let storage = self.index.storage().clone();
            let fw_key_c = fw_key.clone();
            if let Err(e) = block_on_async(async move { storage.delete(tx, &fw_key_c).await }) {
                report.units_failed += 1;
                report
                    .errors
                    .push(format!("Failed to delete forward index key: {e}"));
            }
        }

        // Fallback prefix scan for any extra individual posting or tombstone keys for this doc
        let doc_suffix = format!(":{}", doc_id.inner()).into_bytes();
        let doc_infix = format!(":{}:", doc_id.inner()).into_bytes();

        let storage = self.index.storage().clone();
        let prefix = self.index.prefix().to_vec();
        let raw_prefix_res = block_on_async(async move { storage.scan_prefix(&prefix).await });

        let raw_prefix_entries = match raw_prefix_res {
            Ok(entries) => entries,
            Err(e) => {
                report.units_failed += 1;
                report
                    .errors
                    .push(format!("Failed prefix scan during purge: {e}"));
                Vec::new()
            }
        };

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
                report.bytes_freed = report
                    .bytes_freed
                    .saturating_add((k.len() + v.len()) as u64);
                report.units_purged = report.units_purged.saturating_add(1);
                let storage = self.index.storage().clone();
                let k_c = k.clone();
                if let Err(e) = block_on_async(async move { storage.delete(tx, &k_c).await }) {
                    report.units_failed += 1;
                    report
                        .errors
                        .push(format!("Failed to delete prefix key: {e}"));
                }
            }
        }

        // 3. Purge document postings from terms and batch posting lists
        for term in &terms_to_check {
            // Load current posting list for term
            let plb_key = self.index.key_batch_posting_list(term);
            let storage = self.index.storage().clone();
            let plb_key_c = plb_key.clone();
            let plb_bytes_res = block_on_async(async move { storage.get(&plb_key_c).await });

            let plb_bytes_opt = match plb_bytes_res {
                Ok(opt) => opt,
                Err(e) => {
                    report.units_failed += 1;
                    report.errors.push(format!(
                        "Failed to read batch posting key for '{term}': {e}"
                    ));
                    None
                }
            };

            let plist_opt = match plb_bytes_opt {
                Some(ref bytes) => match PostingList::decode_compact(bytes) {
                    Ok(plist) => Some(plist),
                    Err(e) => {
                        report.units_failed += 1;
                        report.errors.push(format!(
                            "Corrupted batch posting list for term '{term}': {e}"
                        ));
                        None
                    }
                },
                None => None,
            };

            let updated_plist = match plist_opt {
                Some(plist) => plist.remove(doc_id),
                None => PostingList::empty(),
            };

            if updated_plist.is_empty() {
                // Term is now empty: purge batch posting key and remove from dictionary
                if let Some(ref bytes) = plb_bytes_opt {
                    report.bytes_freed = report
                        .bytes_freed
                        .saturating_add((plb_key.len() + bytes.len()) as u64);
                    report.units_purged = report.units_purged.saturating_add(1);
                }
                let storage = self.index.storage().clone();
                let plb_key_c = plb_key.clone();
                if let Err(e) = block_on_async(async move { storage.delete(tx, &plb_key_c).await })
                {
                    report.units_failed += 1;
                    report
                        .errors
                        .push(format!("Failed to delete empty batch posting key: {e}"));
                }
                self.index
                    .resident_index
                    .remove_posting_from_terms(std::slice::from_ref(term), doc_id);
                self.index
                    .resident_index
                    .remove_terms(std::slice::from_ref(term));
            } else {
                // Update batch posting key and resident index
                match updated_plist.encode_compact() {
                    Ok(encoded) => {
                        let storage = self.index.storage().clone();
                        let plb_key_c = plb_key.clone();
                        if let Err(e) =
                            block_on_async(
                                async move { storage.put(tx, &plb_key_c, &encoded).await },
                            )
                        {
                            report.units_failed += 1;
                            report
                                .errors
                                .push(format!("Failed to write updated batch posting list: {e}"));
                        } else {
                            self.index
                                .resident_index
                                .insert_list(term.clone(), updated_plist);
                        }
                    }
                    Err(e) => {
                        report.units_failed += 1;
                        report.errors.push(format!(
                            "Failed to encode updated posting list for term '{term}': {e}"
                        ));
                    }
                }
            }

            // Delete individual posting key `pl:{term}:{doc_id}` and tombstone key `tbs:{doc_id}:{term}`
            let pl_doc_key = self.index.key_with_term_doc(term, doc_id);
            let tbs_key = self.index.key_tombstone(doc_id, term);
            let storage = self.index.storage().clone();
            if let Err(e) = block_on_async(async move {
                let _ = storage.delete(tx, &pl_doc_key).await;
                let _ = storage.delete(tx, &tbs_key).await;
                Ok::<(), ContextraError>(())
            }) {
                report.units_failed += 1;
                report.errors.push(format!(
                    "Failed to delete individual posting or tombstone key: {e}"
                ));
            }
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
        if let Err(e) = block_on_async(async move { storage.commit(tx).await }) {
            report.units_failed += 1;
            report
                .errors
                .push(format!("Failed to commit purge transaction: {e}"));
        }

        report
    }

    /// Fast-path purge for an entire collection namespace (`__txt:{collection}:`), returning a `PurgeReport`.
    pub fn purge_collection_report(&self, tx: TxId, namespace: &str) -> PurgeReport {
        let _guard = self.index.commit_lock().lock();

        let mut report = PurgeReport::new();

        let prefix = if namespace == "default" {
            b"__txt:default:".to_vec()
        } else if namespace.starts_with("__txt:") {
            namespace.as_bytes().to_vec()
        } else {
            format!("__txt:{}:", namespace).into_bytes()
        };

        let storage = self.index.storage().clone();
        let prefix_c = prefix.clone();
        let raw_entries_res = block_on_async(async move { storage.scan_prefix(&prefix_c).await });

        let raw_entries = match raw_entries_res {
            Ok(entries) => entries,
            Err(e) => {
                report.units_failed += 1;
                report.errors.push(format!(
                    "Failed to scan prefix for collection purge '{namespace}': {e}"
                ));
                Vec::new()
            }
        };

        for (k, v) in raw_entries {
            report.bytes_freed = report
                .bytes_freed
                .saturating_add((k.len() + v.len()) as u64);
            report.units_purged = report.units_purged.saturating_add(1);
            let storage = self.index.storage().clone();
            if let Err(e) = block_on_async(async move { storage.delete(tx, &k).await }) {
                report.units_failed += 1;
                report
                    .errors
                    .push(format!("Failed to delete key during collection purge: {e}"));
            }
        }

        // Reset in-memory statistics and resident index
        self.index.total_docs.store(0, Ordering::SeqCst);
        self.index.total_tokens.store(0, Ordering::SeqCst);
        self.index.avg_doc_len_x1000.store(0, Ordering::SeqCst);
        self.index.staged_stats.lock().clear();
        self.index.staged_terms.lock().clear();
        self.index.resident_index.clear();

        let storage = self.index.storage().clone();
        if let Err(e) = block_on_async(async move { storage.commit(tx).await }) {
            report.units_failed += 1;
            report.errors.push(format!(
                "Failed to commit collection purge transaction: {e}"
            ));
        }

        report
    }

    /// Fast-path purge for an entire collection namespace (`__txt:{collection}:`).
    pub fn purge_collection(&self, tx: TxId, namespace: &str) -> Result<PurgeStats> {
        let report = self.purge_collection_report(tx, namespace);
        report.to_purge_stats()
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
