use super::types::{Language, StagedStatsChange, TextIndexMetadata};
use crate::posting_list::{Posting, ResidentPostingIndex};
use crate::tokenizer::{DefaultTokenizer, GermanMorphTokenizer, Tokenizer};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use contextra_types::{DocId, ContextraError, Result, ScoredDocument, TxId, MAX_SEARCH_K};
use contextra_ports::{StorageEngine, TextIndex, TextIndexStats};
/// An inverted index tied to a specific collection namespace.
///
/// # Concurrency & Lock Hierarchy:
/// 1. `commit_lock` (`parking_lot::Mutex`): Ensures single-writer serialization during transactional stats commits.
/// 2. `staged_stats` (`parking_lot::Mutex`): Synchronous, short-lived spinlock guarding in-memory uncommitted transaction statistics.
///
/// **Rule**: `commit_lock` must ALWAYS be acquired BEFORE acquiring `staged_stats`. Never hold synchronous lock guards across `.await` points.
pub struct InvertedIndex<S: StorageEngine> {
    storage: Arc<S>,
    prefix: Vec<u8>,
    tokenizer: Arc<dyn Tokenizer>,
    pub(crate) total_docs: Arc<AtomicU64>,
    pub(crate) total_tokens: Arc<AtomicU64>,
    pub(crate) avg_doc_len_x1000: Arc<AtomicU64>, // Cached fixed-point (FIND-TXT-004)
    pub(crate) staged_stats: Arc<parking_lot::Mutex<HashMap<TxId, StagedStatsChange>>>,
    pub(crate) staged_terms: Arc<parking_lot::Mutex<HashMap<TxId, Vec<String>>>>,
    commit_lock: Arc<parking_lot::Mutex<()>>,
    pub(crate) resident_index: Arc<ResidentPostingIndex>,
}

impl<S: StorageEngine> Clone for InvertedIndex<S> {
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
            prefix: self.prefix.clone(),
            tokenizer: self.tokenizer.clone(),
            total_docs: self.total_docs.clone(),
            total_tokens: self.total_tokens.clone(),
            avg_doc_len_x1000: self.avg_doc_len_x1000.clone(),
            staged_stats: self.staged_stats.clone(),
            staged_terms: self.staged_terms.clone(),
            commit_lock: self.commit_lock.clone(),
            resident_index: self.resident_index.clone(),
        }
    }
}

impl<S: StorageEngine> InvertedIndex<S> {
    /// Maximum allowed document text size in bytes (10 MB).
    pub const MAX_TEXT_BYTES: usize = 10 * 1024 * 1024;

    /// Maximum allowed staged uncommitted transaction statistics changes (10,000).
    pub const MAX_STAGED_TRANSACTIONS: usize = 10_000;

    /// Creates a new InvertedIndex with explicit language configuration.
    ///
    /// # Language Selection
    /// Use `Language::from_iso("de")` for German,
    /// `Language::English` (default) for all other languages.
    ///
    /// **NOT** namespace-based — namespace names do NOT determine language.
    pub fn new_with_language(storage: Arc<S>, namespace: &str, language: Language) -> Self {
        let prefix = if namespace == "default" {
            b"__txt:default:".to_vec()
        } else {
            format!("__txt:{}:", namespace).into_bytes()
        };

        let tokenizer: Arc<dyn Tokenizer> = match language {
            Language::German => Arc::new(GermanMorphTokenizer::new()),
            Language::English | Language::Custom => Arc::new(DefaultTokenizer),
        };

        Self {
            storage,
            prefix,
            tokenizer,
            total_docs: Arc::new(AtomicU64::new(0)),
            total_tokens: Arc::new(AtomicU64::new(0)),
            avg_doc_len_x1000: Arc::new(AtomicU64::new(0)),
            staged_stats: Arc::new(parking_lot::Mutex::new(HashMap::new())),
            staged_terms: Arc::new(parking_lot::Mutex::new(HashMap::new())),
            commit_lock: Arc::new(parking_lot::Mutex::new(())),
            resident_index: Arc::new(ResidentPostingIndex::new()),
        }
    }

    /// Creates a new InvertedIndex with English tokenizer (default).
    pub fn new(storage: Arc<S>, namespace: &str) -> Self {
        Self::new_with_language(storage, namespace, Language::English)
    }

    /// Sets a custom tokenizer for the inverted index.
    pub fn with_tokenizer(mut self, tokenizer: Arc<dyn Tokenizer>) -> Self {
        self.tokenizer = tokenizer;
        self
    }

    /// Loads index statistics from storage into the cache.
    pub async fn load_stats(&self) -> Result<()> {
        let meta_key = self.key("meta:stats");
        if let Some(bytes) = self.storage.get(&meta_key).await? {
            let meta: TextIndexMetadata = bincode::deserialize(&bytes)
                .map_err(|e| ContextraError::Storage(format!("bincode: {}", e)))?;
            self.total_docs.store(meta.total_docs, Ordering::SeqCst);
            self.total_tokens.store(meta.total_tokens, Ordering::SeqCst);
            self.avg_doc_len_x1000
                .store(meta.avg_doc_len_x1000, Ordering::SeqCst);
        }
        Ok(())
    }

    pub(crate) fn key(&self, suffix: &str) -> Vec<u8> {
        let mut k = Vec::with_capacity(self.prefix.len() + suffix.len());
        k.extend_from_slice(&self.prefix);
        k.extend_from_slice(suffix.as_bytes());
        k
    }

    pub(crate) fn key_with_id(&self, type_prefix: &str, id: impl Into<u128>) -> Vec<u8> {
        let mut itoa_buf = itoa::Buffer::new();
        let id_str = itoa_buf.format(id.into());
        let mut k = Vec::with_capacity(self.prefix.len() + type_prefix.len() + id_str.len());
        k.extend_from_slice(&self.prefix);
        k.extend_from_slice(type_prefix.as_bytes());
        k.extend_from_slice(id_str.as_bytes());
        k
    }

    pub(crate) fn key_with_term_doc(&self, term: &str, doc_id: DocId) -> Vec<u8> {
        let mut itoa_buf = itoa::Buffer::new();
        let id_str = itoa_buf.format(doc_id.inner());
        let mut k = Vec::with_capacity(self.prefix.len() + 3 + term.len() + 1 + id_str.len());
        k.extend_from_slice(&self.prefix);
        k.extend_from_slice(b"pl:");
        k.extend_from_slice(term.as_bytes());
        k.push(b':');
        k.extend_from_slice(id_str.as_bytes());
        k
    }

    pub(crate) fn key_term_prefix(&self, term: &str) -> Vec<u8> {
        let mut k = Vec::with_capacity(self.prefix.len() + 3 + term.len() + 1);
        k.extend_from_slice(&self.prefix);
        k.extend_from_slice(b"pl:");
        k.extend_from_slice(term.as_bytes());
        k.push(b':');
        k
    }

    pub(crate) fn key_batch_posting_list(&self, term: &str) -> Vec<u8> {
        let mut k = Vec::with_capacity(self.prefix.len() + 4 + term.len());
        k.extend_from_slice(&self.prefix);
        k.extend_from_slice(b"plb:");
        k.extend_from_slice(term.as_bytes());
        k
    }

    /// Ensures that the posting list for a term is loaded into the resident index from storage.
    pub(crate) async fn ensure_term_loaded(
        &self,
        term: &str,
    ) -> Result<Arc<crate::posting_list::PostingList>> {
        if let Some(list) = self.resident_index.get(term) {
            return Ok(list);
        }

        let plb_key = self.key_batch_posting_list(term);
        if let Some(bytes) = self.storage.get(&plb_key).await? {
            if let Ok(plist) = crate::posting_list::PostingList::decode_compact(&bytes) {
                self.resident_index.insert_list(term.to_string(), plist);
                if let Some(l) = self.resident_index.get(term) {
                    return Ok(l);
                }
            }
        }

        // Fallback to legacy single-key prefix scan
        let prefix = self.key_term_prefix(term);
        let raw_entries = self.storage.scan_prefix(&prefix).await?;
        let mut postings = Vec::with_capacity(raw_entries.len());
        for (key, val_bytes) in raw_entries {
            let suffix_bytes = &key[prefix.len()..];
            if let Ok(suffix) = std::str::from_utf8(suffix_bytes) {
                #[cfg(not(feature = "docid-128"))]
                let parsed_doc_id = suffix.parse::<u64>().ok();
                #[cfg(feature = "docid-128")]
                let parsed_doc_id = suffix.parse::<u128>().ok();

                if let Some(doc_id_raw) = parsed_doc_id {
                    if val_bytes.len() == 4 {
                        let doc_id = DocId::new(doc_id_raw);
                        let tf =
                            u32::from_le_bytes((&val_bytes[..4]).try_into().map_err(|_| {
                                ContextraError::Storage("Invalid posting tf length".into())
                            })?);
                        let dl_key = self.key_with_id("dl:", doc_id.inner());
                        let doc_len = match self.storage.get(&dl_key).await? {
                            Some(dl_bytes) if dl_bytes.len() == 4 => {
                                u32::from_le_bytes((&dl_bytes[..]).try_into().map_err(|_| {
                                    ContextraError::Storage("Invalid doc_len length".into())
                                })?)
                            }
                            _ => 0,
                        };
                        postings.push(Posting::new(doc_id, tf, doc_len));
                    }
                }
            }
        }
        let plist = crate::posting_list::PostingList::new(postings);
        self.resident_index
            .insert_list(term.to_string(), plist.clone());
        Ok(Arc::new(plist))
    }

    /// Generates a tombstone storage key for document updating or deletion.
    ///
    /// # Tombstone Key & Value Schema
    /// - **Key Format**: `__txt:{namespace}:tbs:{doc_id}:{term}`
    /// - **Value Schema**: Empty byte slice (`&[]`).
    /// - **Storage Layer**: Written to the transactional LSM `StorageEngine` during `upsert_document()` (updates)
    ///   and `delete_document()` (deletions).
    /// - **MVCC & Snapshot Isolation**: Evaluated during `search_bm25_at()` via `storage.get_at_seq(&tbs_key, seq)`.
    ///   If a tombstone key exists at or before `seq`, the posting entry for `(term, doc_id)` is masked out.
    /// - **Compaction Strategy**: Tombstone keys persist in storage until resolved via `resolve_tombstones()`
    ///   or removed during background compaction when all snapshots prior to tombstone creation are no longer active.
    pub(crate) fn key_tombstone(&self, doc_id: DocId, term: &str) -> Vec<u8> {
        let mut itoa_buf = itoa::Buffer::new();
        let id_str = itoa_buf.format(doc_id.inner());
        let mut k = Vec::with_capacity(self.prefix.len() + 4 + id_str.len() + 1 + term.len());
        k.extend_from_slice(&self.prefix);
        k.extend_from_slice(b"tbs:");
        k.extend_from_slice(id_str.as_bytes());
        k.push(b':');
        k.extend_from_slice(term.as_bytes());
        k
    }

    #[tracing::instrument(skip(self, text))]
    pub async fn upsert_document(&self, tx: TxId, doc_id: DocId, text: &str) -> Result<()> {
        if text.len() > Self::MAX_TEXT_BYTES {
            return Err(ContextraError::InvalidInput(format!(
                "Text length {} exceeds maximum allowed size {}",
                text.len(),
                Self::MAX_TEXT_BYTES
            )));
        }

        let tokens = self.tokenizer.tokenize(text);
        let new_len = tokens.len() as u32;

        let mut tfs = HashMap::with_capacity(tokens.len());
        for t in tokens {
            *tfs.entry(t).or_insert(0u32) += 1;
        }

        let dl_key = self.key_with_id("dl:", doc_id.inner());
        let fw_key = self.key_with_id("fw:", doc_id.inner());
        let mut old_len = 0u32;
        let mut is_update = false;

        if let Some(bytes) = self.storage.get(&dl_key).await? {
            if bytes.len() == 4 {
                old_len = u32::from_le_bytes(
                    (&bytes[..])
                        .try_into()
                        .map_err(|_| ContextraError::Storage("Invalid doc_len length".into()))?,
                );
                is_update = true;

                if let Some(fw_bytes) = self.storage.get(&fw_key).await? {
                    let old_terms: Vec<String> = bincode::deserialize(&fw_bytes)
                        .map_err(|e| ContextraError::Storage(format!("bincode: {}", e)))?;
                    for term in old_terms {
                        let tbs_key = self.key_tombstone(doc_id, &term);
                        self.storage.put(tx, &tbs_key, &[]).await?;
                    }
                }
            }
        }

        self.storage
            .put(tx, &dl_key, &new_len.to_le_bytes())
            .await?;

        let mut tfs_vec: Vec<(String, u32)> = tfs.into_iter().collect();
        tfs_vec.sort_by(|a, b| a.0.cmp(&b.0));

        let unique_terms: Vec<&str> = tfs_vec.iter().map(|(k, _)| k.as_str()).collect();
        let fw_bytes = bincode::serialize(&unique_terms)
            .map_err(|e| ContextraError::Storage(format!("bincode: {}", e)))?;
        self.storage.put(tx, &fw_key, &fw_bytes).await?;

        // Stage stats changes (Option B: Atomics as Source of Truth, no direct read-modify-write)
        let mut change = StagedStatsChange::default();
        if is_update {
            change.tokens_delta = (new_len as i64) - (old_len as i64);
        } else {
            change.docs_delta = 1;
            change.tokens_delta = new_len as i64;
        }
        self.stage_stats_change(tx, change)?;

        for (term, tf) in &tfs_vec {
            self.ensure_term_loaded(term).await?;
            let pl_doc_key = self.key_with_term_doc(term, doc_id);
            self.storage.put(tx, &pl_doc_key, &tf.to_le_bytes()).await?;
            self.resident_index
                .upsert_posting(term, Posting::new(doc_id, *tf, new_len));

            if let Some(plist) = self.resident_index.get(term) {
                let batch_bytes = plist.encode_compact()?;
                let plb_key = self.key_batch_posting_list(term);
                self.storage.put(tx, &plb_key, &batch_bytes).await?;
            }
        }

        Ok(())
    }

    ///
    /// For every document with a `tbs:{doc_id}` marker this method reads the
    /// *current* forward index and removes posting-list entries for terms that
    /// are no longer present.  Call this during background compaction or
    /// whenever write throughput is low.
    ///
    /// Returns the number of tombstones that were resolved.
    pub async fn resolve_tombstones(&self, tx: TxId) -> Result<u64> {
        let tbs_prefix = {
            let mut k = Vec::with_capacity(self.prefix.len() + 4);
            k.extend_from_slice(&self.prefix);
            k.extend_from_slice(b"tbs:");
            k
        };

        let tombstones = self.storage.scan_prefix(&tbs_prefix).await?;
        let mut resolved = 0u64;

        for (tbs_key, _) in tombstones {
            // Format: {prefix}tbs:{doc_id}:{term}
            let suffix = &tbs_key[tbs_prefix.len()..];
            let parts: Vec<&[u8]> = suffix.split(|&b| b == b':').collect();
            if parts.len() < 2 {
                continue;
            }

            #[cfg(not(feature = "docid-128"))]
            let doc_id_raw_opt = std::str::from_utf8(parts[0])
                .ok()
                .and_then(|s| s.parse::<u64>().ok())
                .map(DocId::new);
            #[cfg(feature = "docid-128")]
            let doc_id_raw_opt = std::str::from_utf8(parts[0])
                .ok()
                .and_then(|s| s.parse::<u128>().ok())
                .map(|v| DocId::new(v));
            let doc_id = match doc_id_raw_opt {
                Some(v) => v,
                None => continue,
            };
            let term = String::from_utf8_lossy(parts[1]).to_string();

            // Read the *current* forward index to know which terms are live.
            // DECISION-REF: Consistent with load_stats() (L92) — bincode errors must propagate,
            // not be silently swallowed. Silent failure would cause phantom BM25 term deletion.
            // AI-TAG[SPEC-DRIFT][MAJOR] RESOLVED: AGT-TXT-001 — replaced unwrap_or_default() with map_err (TS:2026-08-25T00:00:00Z)
            let fw_key = self.key_with_id("fw:", doc_id.inner());
            let is_live = if let Some(fw_bytes) = self.storage.get(&fw_key).await? {
                let live_terms: Vec<String> = bincode::deserialize::<Vec<String>>(&fw_bytes)
                    .map_err(|e| {
                        ContextraError::Storage(format!(
                            "forward-index corrupt for doc {}: {}",
                            doc_id.inner(),
                            e
                        ))
                    })?;
                live_terms.contains(&term)
            } else {
                false // Document deleted — tombstone cleanup is safe
            };

            if !is_live {
                self.ensure_term_loaded(&term).await?;
                let pl_key = self.key_with_term_doc(&term, doc_id);
                self.storage.delete(tx, &pl_key).await?;
                self.resident_index
                    .remove_posting_from_terms(std::slice::from_ref(&term), doc_id);

                let plb_key = self.key_batch_posting_list(&term);
                if let Some(plist) = self.resident_index.get(&term) {
                    let batch_bytes = plist.encode_compact()?;
                    self.storage.put(tx, &plb_key, &batch_bytes).await?;
                } else {
                    self.storage.delete(tx, &plb_key).await?;
                }
            }
            self.storage.delete(tx, &tbs_key).await?;
            resolved += 1;
        }

        Ok(resolved)
    }

    /// Deletes a document from the index.
    pub async fn delete_document(&self, tx: TxId, doc_id: DocId) -> Result<()> {
        let dl_key = self.key_with_id("dl:", doc_id.inner());
        let fw_key = self.key_with_id("fw:", doc_id.inner());

        let mut doc_len = 0u32;
        if let Some(bytes) = self.storage.get(&dl_key).await? {
            if bytes.len() == 4 {
                doc_len = u32::from_le_bytes(
                    (&bytes[..])
                        .try_into()
                        .map_err(|_| ContextraError::Storage("Invalid doc_len length".into()))?,
                );
            }
        } else {
            return Ok(());
        }

        self.storage.delete(tx, &dl_key).await?;

        // Remove from posting lists using forward index and write tombstone markers
        if let Some(fw_bytes) = self.storage.get(&fw_key).await? {
            if let Ok(old_terms) = bincode::deserialize::<Vec<String>>(&fw_bytes) {
                for term in &old_terms {
                    self.ensure_term_loaded(term).await?;
                    let pl_doc_key = self.key_with_term_doc(term, doc_id);
                    self.storage.delete(tx, &pl_doc_key).await?;
                    let tbs_key = self.key_tombstone(doc_id, term);
                    self.storage.put(tx, &tbs_key, &[]).await?;

                    self.resident_index
                        .remove_posting_from_terms(std::slice::from_ref(term), doc_id);

                    let plb_key = self.key_batch_posting_list(term);
                    if let Some(plist) = self.resident_index.get(term) {
                        let batch_bytes = plist.encode_compact()?;
                        self.storage.put(tx, &plb_key, &batch_bytes).await?;
                    } else {
                        self.storage.delete(tx, &plb_key).await?;
                    }
                }
            }
        }
        self.storage.delete(tx, &fw_key).await?;

        // Stage stats changes
        let change = StagedStatsChange {
            docs_delta: -1,
            tokens_delta: -(doc_len as i64),
        };
        self.stage_stats_change(tx, change)?;

        Ok(())
    }

    fn stage_stats_change(&self, tx: TxId, change: StagedStatsChange) -> Result<()> {
        let mut guard = self.staged_stats.lock();
        if guard.len() >= Self::MAX_STAGED_TRANSACTIONS && !guard.contains_key(&tx) {
            return Err(ContextraError::InvalidInput(format!(
                "Staged stats limit ({}) exceeded for uncommitted transactions",
                Self::MAX_STAGED_TRANSACTIONS
            )));
        }
        let entry = guard.entry(tx).or_default();
        entry.docs_delta += change.docs_delta;
        entry.tokens_delta += change.tokens_delta;
        Ok(())
    }

    #[allow(dead_code)]
    pub(crate) async fn commit_stats(&self, tx: TxId) -> Result<()> {
        let change = {
            let mut guard = self.staged_stats.lock();
            guard.remove(&tx).unwrap_or_default()
        };

        if change.docs_delta != 0 || change.tokens_delta != 0 {
            if change.docs_delta > 0 {
                self.total_docs
                    .fetch_add(change.docs_delta as u64, Ordering::SeqCst);
            } else if change.docs_delta < 0 {
                self.total_docs
                    .fetch_sub(change.docs_delta.unsigned_abs(), Ordering::SeqCst);
            }

            if change.tokens_delta > 0 {
                self.total_tokens
                    .fetch_add(change.tokens_delta as u64, Ordering::SeqCst);
            } else if change.tokens_delta < 0 {
                self.total_tokens
                    .fetch_sub(change.tokens_delta.unsigned_abs(), Ordering::SeqCst);
            }

            let docs = self.total_docs.load(Ordering::SeqCst);
            let tokens = self.total_tokens.load(Ordering::SeqCst);
            let avg_len = if docs > 0 {
                (tokens as f64 / docs as f64 * 1000.0) as u64
            } else {
                0
            };
            self.avg_doc_len_x1000.store(avg_len, Ordering::SeqCst);
        }

        let meta_bytes = {
            let _guard = self.commit_lock.lock();
            let docs = self.total_docs.load(Ordering::SeqCst);
            let tokens = self.total_tokens.load(Ordering::SeqCst);
            let avg_len = self.avg_doc_len_x1000.load(Ordering::SeqCst);

            let meta = TextIndexMetadata {
                total_docs: docs,
                total_tokens: tokens,
                avg_doc_len_x1000: avg_len,
            };
            bincode::serialize(&meta)
                .map_err(|e| ContextraError::Storage(format!("bincode: {}", e)))?
        };
        let meta_key = self.key("meta:stats");

        self.storage.put(tx, &meta_key, &meta_bytes).await
    }

    #[allow(dead_code)]
    pub(crate) async fn rollback_stats(&self, tx: TxId) -> Result<()> {
        let mut guard = self.staged_stats.lock();
        guard.remove(&tx);
        self.resident_index.clear();
        Ok(())
    }

    /// Searches the inverted index using BM25.
    #[tracing::instrument(skip(self, query))]
    pub async fn search_bm25(
        &self,
        query: &str,
        k: usize,
        max_seq: Option<u64>,
    ) -> Result<Vec<(DocId, f32)>> {
        self.search_bm25_at(query, k, max_seq).await
    }

    /// Searches the inverted index using BM25 at a specific snapshot version.
    #[tracing::instrument(skip(self, query))]
    pub async fn search_bm25_at(
        &self,
        query: &str,
        k: usize,
        max_seq: Option<u64>,
    ) -> Result<Vec<(DocId, f32)>> {
        if query.is_empty() || k == 0 {
            return Ok(Vec::new());
        }

        let k = k.min(MAX_SEARCH_K);

        if query.len() > Self::MAX_TEXT_BYTES {
            return Err(ContextraError::InvalidInput(format!(
                "Query length {} exceeds maximum allowed size {}",
                query.len(),
                Self::MAX_TEXT_BYTES
            )));
        }

        let tokens = self.tokenizer.tokenize(query);
        if tokens.is_empty() {
            return Ok(Vec::new());
        }

        let n = self.total_docs.load(Ordering::Acquire);
        let cached_avg_len_x1000 = self.avg_doc_len_x1000.load(Ordering::Acquire);

        let avg_doc_len = if n > 0 {
            cached_avg_len_x1000 as f32 / 1000.0
        } else {
            0.0
        };

        let wand_res = crate::wand::block_max_wand_search(
            self.storage.as_ref(),
            &tokens,
            &self.resident_index,
            |term| self.key_term_prefix(term),
            |doc_id, term| self.key_tombstone(doc_id, term),
            |doc_id_inner| self.key_with_id("dl:", doc_id_inner),
            k,
            max_seq,
            n,
            avg_doc_len,
        )
        .await?;

        Ok(wand_res.results)
    }
}

impl<S: StorageEngine> TextIndex for InvertedIndex<S> {
    async fn search(&self, query: &str, k: usize) -> Result<Vec<ScoredDocument>> {
        let results = self.search_bm25(query, k, None).await?;
        Ok(results
            .into_iter()
            .map(|(doc_id, score)| ScoredDocument { doc_id, score })
            .collect())
    }

    /// Searches the inverted index at a specific MVCC sequence number snapshot.
    ///
    /// Snapshot isolation is preserved by delegating to `search_bm25_at()`, which uses
    /// `scan_prefix_at()` and `get_at_seq()` on the underlying `StorageEngine`.
    async fn search_at(&self, query: &str, k: usize, seq_no: u64) -> Result<Vec<ScoredDocument>> {
        let results = self.search_bm25(query, k, Some(seq_no)).await?;
        Ok(results
            .into_iter()
            .map(|(doc_id, score)| ScoredDocument { doc_id, score })
            .collect())
    }

    async fn insert(&self, tx: TxId, id: DocId, text: &str) -> Result<()> {
        self.upsert_document(tx, id, text).await
    }

    async fn delete(&self, tx: TxId, id: DocId) -> Result<()> {
        self.delete_document(tx, id).await
    }

    async fn commit(&self, tx: TxId) -> Result<()> {
        self.commit_stats(tx).await?;
        self.storage.commit(tx).await
    }

    async fn rollback(&self, tx: TxId) -> Result<()> {
        self.rollback_stats(tx).await?;
        self.storage.rollback(tx).await
    }

    async fn rollback_to_tx(&self, tx_id: TxId) -> Result<()> {
        self.storage.rollback_to_tx(tx_id).await
    }

    async fn last_tx_id(&self) -> Result<TxId> {
        self.storage.last_tx_id().await
    }

    async fn len(&self) -> usize {
        self.total_docs.load(Ordering::Acquire) as usize
    }

    async fn stats(&self) -> Result<TextIndexStats> {
        let meta_key = self.key("meta:stats");
        let meta = if let Some(bytes) = self.storage.get(&meta_key).await? {
            bincode::deserialize::<TextIndexMetadata>(&bytes)
                .map_err(|e| ContextraError::Storage(format!("bincode: {}", e)))?
        } else {
            TextIndexMetadata::default()
        };

        Ok(TextIndexStats {
            num_documents: meta.total_docs as usize,
            num_tokens: meta.total_tokens as usize,
            memory_usage_bytes: 0,
        })
    }
}
