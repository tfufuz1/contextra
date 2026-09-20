// FILE-CONTEXT: Block-Max WAND (Weak-AND) BM25 Query Execution Engine (IP-10).
// ZWECK: Multi-Term Query-Traversierung mit Block-Maximalwert-Pruning und dynamic thresholding.
// INVARIANTEN: Zero-Panic Doctrine, Safe Optimization (Ergebnisse identisch zu Full-Scan), MVCC & Tombstone Isolation.

use crate::bm25::score_term;
use crate::posting_list::{Posting, PostingList, BLOCK_SIZE};
use memfuse_core::{DocId, MemFuseError, Result, StorageEngine, MAX_SEARCH_K};
use std::collections::{BinaryHeap, HashMap};
use std::sync::Arc;

struct Bm25Candidate {
    doc_id: DocId,
    score: f32,
}

impl PartialEq for Bm25Candidate {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == std::cmp::Ordering::Equal
    }
}

impl Eq for Bm25Candidate {}

impl Ord for Bm25Candidate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Min-heap for Top-K: worst candidate (lowest score, highest DocId) at peak
        other
            .score
            .partial_cmp(&self.score)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| self.doc_id.cmp(&other.doc_id))
    }
}

impl PartialOrd for Bm25Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub struct BoundedTopK {
    heap: BinaryHeap<Bm25Candidate>,
    capacity: usize,
}

impl BoundedTopK {
    pub fn new(capacity: usize) -> Self {
        let capacity = capacity.min(MAX_SEARCH_K);
        Self {
            heap: BinaryHeap::with_capacity(capacity.saturating_add(1)),
            capacity,
        }
    }

    pub fn push(&mut self, doc_id: DocId, score: f32) {
        if self.capacity == 0 {
            return;
        }
        let candidate = Bm25Candidate { doc_id, score };
        if self.heap.len() < self.capacity {
            self.heap.push(candidate);
        } else if let Some(worst) = self.heap.peek() {
            if candidate < *worst {
                self.heap.pop();
                self.heap.push(candidate);
            }
        }
    }

    pub fn min_threshold(&self) -> f32 {
        if self.heap.len() < self.capacity {
            0.0
        } else {
            self.heap.peek().map(|c| c.score).unwrap_or(0.0)
        }
    }

    pub fn into_sorted_vec(self) -> Vec<(DocId, f32)> {
        self.heap
            .into_sorted_vec()
            .into_iter()
            .map(|c| (c.doc_id, c.score))
            .collect()
    }
}

/// Represents an active query term cursor during WAND traversal.
struct TermCursor {
    term: String,
    posting_list: Arc<PostingList>,
    df: u32,
    cursor: usize,
    /// Precomputed global maximum term score upper bound.
    term_max_score: f32,
    /// Precomputed block maximum term score upper bounds.
    block_max_scores: Vec<f32>,
}

impl TermCursor {
    fn new(
        term: String,
        posting_list: Arc<PostingList>,
        df: u32,
        total_docs: u32,
        avg_doc_len: f32,
    ) -> Self {
        let blocks = posting_list.blocks();
        let mut block_max_scores = Vec::with_capacity(blocks.len());
        let mut term_max_score = 0.0f32;

        for block in blocks {
            let max_score =
                score_term(block.max_tf, block.min_doc_len, avg_doc_len, df, total_docs);
            if max_score > term_max_score {
                term_max_score = max_score;
            }
            block_max_scores.push(max_score);
        }

        Self {
            term,
            posting_list,
            df,
            cursor: 0,
            term_max_score,
            block_max_scores,
        }
    }

    #[inline]
    fn current_posting(&self) -> Option<&Posting> {
        self.posting_list.as_slice().get(self.cursor)
    }

    #[inline]
    fn current_doc_id(&self) -> Option<u64> {
        self.current_posting().map(|p| p.doc_id)
    }

    #[inline]
    fn block_max_score_at_doc_id(&self, doc_id: u64) -> f32 {
        let postings = self.posting_list.as_slice();
        if self.cursor >= postings.len() {
            return 0.0;
        }
        // If current cursor doc_id matches target doc_id, return current block max score
        if let Some(p) = postings.get(self.cursor) {
            if p.doc_id == doc_id {
                let block_idx = self.cursor / BLOCK_SIZE;
                return self
                    .block_max_scores
                    .get(block_idx)
                    .copied()
                    .unwrap_or(self.term_max_score);
            }
        }
        // Binary search for doc_id block index
        let blocks = self.posting_list.blocks();
        match blocks.binary_search_by_key(&doc_id, |b| b.max_doc_id) {
            Ok(idx) => self
                .block_max_scores
                .get(idx)
                .copied()
                .unwrap_or(self.term_max_score),
            Err(idx) => {
                if idx < blocks.len() {
                    self.block_max_scores
                        .get(idx)
                        .copied()
                        .unwrap_or(self.term_max_score)
                } else {
                    0.0
                }
            }
        }
    }

    /// Advances cursor to the first posting with `doc_id >= target_doc_id`.
    fn advance_to(&mut self, target_doc_id: u64) {
        let postings = self.posting_list.as_slice();
        if self.cursor >= postings.len() {
            return;
        }

        // Skip blocks whose max_doc_id is strictly less than target_doc_id
        let blocks = self.posting_list.blocks();
        let mut block_idx = self.cursor / BLOCK_SIZE;

        while block_idx < blocks.len() && blocks[block_idx].max_doc_id < target_doc_id {
            block_idx += 1;
        }

        let new_cursor_start = block_idx * BLOCK_SIZE;
        if new_cursor_start >= postings.len() {
            self.cursor = postings.len();
            return;
        }

        self.cursor = new_cursor_start.max(self.cursor);

        // Binary search within current block / remaining postings
        let slice = &postings[self.cursor..];
        match slice.binary_search_by_key(&target_doc_id, |p| p.doc_id) {
            Ok(offset) => self.cursor += offset,
            Err(offset) => self.cursor += offset,
        }
    }
}

/// Result from Block-Max WAND search containing top-k items and execution metrics.
#[derive(Debug)]
pub struct WandSearchResult {
    pub results: Vec<(DocId, f32)>,
    pub evaluated_docs_count: usize,
}

/// Executes Block-Max WAND traversal over resident posting lists.
#[allow(clippy::too_many_arguments)]
pub async fn block_max_wand_search<S: StorageEngine>(
    storage: &S,
    terms: &[String],
    resident_index: &crate::posting_list::ResidentPostingIndex,
    prefix_helper: impl Fn(&str) -> Vec<u8>,
    tombstone_key_helper: impl Fn(DocId, &str) -> Vec<u8>,
    doc_len_key_helper: impl Fn(u64) -> Vec<u8>,
    k: usize,
    max_seq: Option<u64>,
    total_docs: u64,
    avg_doc_len: f32,
) -> Result<WandSearchResult> {
    if terms.is_empty() || k == 0 {
        return Ok(WandSearchResult {
            results: Vec::new(),
            evaluated_docs_count: 0,
        });
    }

    let seq = if let Some(s) = max_seq {
        s
    } else {
        storage.last_seq_no().await?
    };

    let is_latest = max_seq.is_none() || max_seq == Some(u64::MAX);
    let total_docs_u32 = (total_docs as u32).max(1);

    let mut doc_len_cache: HashMap<DocId, Option<u32>> = HashMap::new();
    let mut tbs_cache: HashMap<Vec<u8>, bool> = HashMap::new();

    // Retrieve posting lists for all query terms
    let mut cursors: Vec<TermCursor> = Vec::with_capacity(terms.len());
    for term in terms {
        let list = match resident_index.get(term) {
            Some(l) => l,
            None => {
                let prefix = prefix_helper(term);

                // Construct batch posting list key by replacing "pl:" with "plb:" and stripping trailing ":"
                let mut batch_key = Vec::with_capacity(prefix.len() + 1);
                if let Some(pos) = prefix.windows(3).position(|w| w == b"pl:") {
                    batch_key.extend_from_slice(&prefix[..pos]);
                    batch_key.extend_from_slice(b"plb:");
                    let remaining = &prefix[pos + 3..];
                    if remaining.ends_with(b":") {
                        batch_key.extend_from_slice(&remaining[..remaining.len() - 1]);
                    } else {
                        batch_key.extend_from_slice(remaining);
                    }
                } else {
                    batch_key = prefix.clone();
                }

                let loaded_list = if let Some(bytes) = storage.get_at_seq(&batch_key, seq).await? {
                    bincode::deserialize::<PostingList>(&bytes).ok()
                } else {
                    None
                };

                let plist = match loaded_list {
                    Some(l) => l,
                    None => {
                        let raw_entries = storage.scan_prefix_at(&prefix, u64::MAX).await?;
                        let mut postings = Vec::with_capacity(raw_entries.len());
                        for (key, val_bytes) in raw_entries {
                            let suffix_bytes = &key[prefix.len()..];
                            if let Ok(suffix) = std::str::from_utf8(suffix_bytes) {
                                if let Ok(doc_id_raw) = suffix.parse::<u64>() {
                                    if val_bytes.len() == 4 {
                                        let doc_id = DocId::new(doc_id_raw);
                                        let tf = u32::from_le_bytes(
                                            (&val_bytes[..4]).try_into().map_err(|_| {
                                                MemFuseError::Storage(
                                                    "Invalid posting tf length".into(),
                                                )
                                            })?,
                                        );
                                        let dl_key = doc_len_key_helper(doc_id.inner());
                                        let doc_len = match storage.get(&dl_key).await? {
                                            Some(dl_bytes) if dl_bytes.len() == 4 => {
                                                u32::from_le_bytes(
                                                    (&dl_bytes[..]).try_into().map_err(|_| {
                                                        MemFuseError::Storage(
                                                            "Invalid doc_len length".into(),
                                                        )
                                                    })?,
                                                )
                                            }
                                            _ => 0,
                                        };
                                        postings.push(Posting::new(doc_id, tf, doc_len));
                                    }
                                }
                            }
                        }
                        PostingList::new(postings)
                    }
                };

                resident_index.insert_list(term.clone(), plist);
                match resident_index.get(term) {
                    Some(l) => l,
                    None => Arc::new(PostingList::empty()),
                }
            }
        };

        if !list.is_empty() {
            // Count active tombstones for `term` at snapshot `seq` using a fast prefix scan
            // Notice tombstone key format in inverted.rs is `__txt:{ns}:tbs:{doc_id}:{term}`.
            // To scan tombstones for term, we scan prefix `__txt:{ns}:tbs:` and filter by `:{term}` suffix.
            let tbs_global_prefix = tombstone_key_helper(DocId::new(0), "")
                .into_iter()
                .take_while(|&b| b != b'0')
                .collect::<Vec<u8>>();
            let active_tbs_entries = storage.scan_prefix_at(&tbs_global_prefix, seq).await?;
            let mut term_tbs_count = 0u32;
            let term_suffix = format!(":{}", term);
            for (tbs_key, _) in active_tbs_entries {
                if let Ok(key_str) = std::str::from_utf8(&tbs_key) {
                    if key_str.ends_with(&term_suffix) {
                        tbs_cache.insert(tbs_key.clone(), true);
                        // Extract doc_id from tombstone key: {prefix}tbs:{doc_id}:{term}
                        let suffix = &tbs_key[tbs_global_prefix.len()..];
                        if let Some(colon_pos) = suffix.iter().position(|&b| b == b':') {
                            if let Ok(doc_id_str) = std::str::from_utf8(&suffix[..colon_pos]) {
                                if let Ok(doc_id_raw) = doc_id_str.parse::<u64>() {
                                    if list
                                        .as_slice()
                                        .binary_search_by_key(&doc_id_raw, |p| p.doc_id)
                                        .is_ok()
                                    {
                                        term_tbs_count += 1;
                                    }
                                }
                            }
                        }
                    }
                }
            }

            let effective_df = (list.len() as u32).saturating_sub(term_tbs_count);

            if effective_df > 0 {
                cursors.push(TermCursor::new(
                    term.clone(),
                    list,
                    effective_df,
                    total_docs_u32,
                    avg_doc_len,
                ));
            }
        }
    }

    if cursors.is_empty() {
        return Ok(WandSearchResult {
            results: Vec::new(),
            evaluated_docs_count: 0,
        });
    }

    let mut top_k = BoundedTopK::new(k);
    let mut evaluated_docs_count = 0usize;

    loop {
        // Remove exhausted cursors
        cursors.retain(|c| c.current_doc_id().is_some());
        if cursors.is_empty() {
            break;
        }

        // Sort active cursors by current_doc_id ascending
        cursors.sort_by_key(|c| c.current_doc_id().unwrap_or(u64::MAX));

        let threshold = top_k.min_threshold();

        // 1. Pivot selection using conservative global term_max_score
        let mut accum_max_score = 0.0f32;
        let mut pivot_idx = None;

        for (idx, c) in cursors.iter().enumerate() {
            accum_max_score += c.term_max_score;
            if accum_max_score > threshold {
                pivot_idx = Some(idx);
                break;
            }
        }

        let pivot_idx = match pivot_idx {
            Some(idx) => idx,
            None => {
                // Total upper bound of all terms cannot reach threshold -> done!
                break;
            }
        };

        let pivot_doc_id = match cursors.get(pivot_idx).and_then(|c| c.current_doc_id()) {
            Some(id) => id,
            None => break,
        };

        let min_doc_id = match cursors.first().and_then(|c| c.current_doc_id()) {
            Some(id) => id,
            None => break,
        };

        if min_doc_id == pivot_doc_id {
            let doc_id = DocId::new(pivot_doc_id);

            // Check if block max scores at pivot_doc_id still exceed threshold
            let block_max_sum: f32 = cursors
                .iter()
                .filter(|c| c.current_doc_id() == Some(pivot_doc_id))
                .map(|c| c.block_max_score_at_doc_id(pivot_doc_id))
                .sum();

            if top_k.capacity > 0
                && top_k.heap.len() >= top_k.capacity
                && block_max_sum <= threshold
            {
                // Block upper bound cannot beat top_k min_threshold -> advance leading cursors
                for c in cursors.iter_mut() {
                    if c.current_doc_id() == Some(pivot_doc_id) {
                        c.advance_to(pivot_doc_id + 1);
                    }
                }
                continue;
            }

            // MVCC snapshot check for historical queries
            if !is_latest {
                let dl_key = doc_len_key_helper(doc_id.inner());
                let is_active = match doc_len_cache.get(&doc_id) {
                    Some(opt_len) => opt_len.is_some(),
                    None => match storage.get_at_seq(&dl_key, seq).await? {
                        Some(_) => {
                            let len = cursors
                                .first()
                                .and_then(|c| c.current_posting())
                                .map(|p| p.doc_len);
                            doc_len_cache.insert(doc_id, len);
                            true
                        }
                        None => {
                            doc_len_cache.insert(doc_id, None);
                            false
                        }
                    },
                };
                if !is_active {
                    for c in cursors.iter_mut() {
                        if c.current_doc_id() == Some(pivot_doc_id) {
                            c.advance_to(pivot_doc_id + 1);
                        }
                    }
                    continue;
                }
            }

            // Calculate exact BM25 score across all terms matching pivot_doc_id
            let mut total_score = 0.0f32;
            let mut has_valid_term = false;

            for c in cursors.iter_mut() {
                if c.current_doc_id() == Some(pivot_doc_id) {
                    if let Some(posting) = c.current_posting() {
                        let tbs_key = tombstone_key_helper(doc_id, &c.term);
                        let is_tombstone = match tbs_cache.get(&tbs_key) {
                            Some(&ex) => ex,
                            None => {
                                let ex = storage.get_at_seq(&tbs_key, seq).await?.is_some();
                                tbs_cache.insert(tbs_key.clone(), ex);
                                ex
                            }
                        };

                        if !is_tombstone {
                            let score = score_term(
                                posting.tf,
                                posting.doc_len,
                                avg_doc_len,
                                c.df,
                                total_docs_u32,
                            );
                            total_score += score;
                            has_valid_term = true;
                        }
                    }

                    c.advance_to(pivot_doc_id + 1);
                }
            }

            if has_valid_term {
                evaluated_docs_count += 1;
                top_k.push(doc_id, total_score);
            }
        } else {
            // min_doc_id < pivot_doc_id: Advance leading terms towards pivot_doc_id
            for c in cursors.iter_mut() {
                if let Some(cur_id) = c.current_doc_id() {
                    if cur_id < pivot_doc_id {
                        c.advance_to(pivot_doc_id);
                    } else {
                        break;
                    }
                }
            }
        }
    }

    Ok(WandSearchResult {
        results: top_k.into_sorted_vec(),
        evaluated_docs_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inverted::InvertedIndex;
    use memfuse_core::BoxFuture;
    use parking_lot::RwLock;

    type MockStoreMap = RwLock<HashMap<Vec<u8>, Vec<(Vec<u8>, u64)>>>;

    struct MockStorage {
        store: MockStoreMap,
        staged: RwLock<HashMap<memfuse_core::TxId, Vec<Vec<u8>>>>,
        next_seq: std::sync::atomic::AtomicU64,
    }

    impl MockStorage {
        fn new() -> Self {
            Self {
                store: MockStoreMap::new(HashMap::new()),
                staged: RwLock::new(HashMap::new()),
                next_seq: std::sync::atomic::AtomicU64::new(1),
            }
        }
    }

    impl StorageEngine for MockStorage {
        fn get<'a>(&'a self, key: &'a [u8]) -> BoxFuture<'a, Result<Option<bytes::Bytes>>> {
            Box::pin(async move { self.get_at_seq(key, u64::MAX).await })
        }
        fn put<'a>(
            &'a self,
            tx_id: memfuse_core::TxId,
            key: &'a [u8],
            value: &'a [u8],
        ) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move {
                let seq = self
                    .next_seq
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                self.store
                    .write()
                    .entry(key.to_vec())
                    .or_default()
                    .push((value.to_vec(), seq));
                self.staged
                    .write()
                    .entry(tx_id)
                    .or_default()
                    .push(key.to_vec());
                Ok(())
            })
        }
        fn delete<'a>(
            &'a self,
            tx_id: memfuse_core::TxId,
            key: &'a [u8],
        ) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move {
                let seq = self
                    .next_seq
                    .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                let mut w = self.store.write();
                if let Some(versions) = w.get_mut(key) {
                    versions.push((Vec::new(), seq | memfuse_core::TOMBSTONE_BIT));
                } else {
                    w.insert(
                        key.to_vec(),
                        vec![(Vec::new(), seq | memfuse_core::TOMBSTONE_BIT)],
                    );
                }
                self.staged
                    .write()
                    .entry(tx_id)
                    .or_default()
                    .push(key.to_vec());
                Ok(())
            })
        }
        fn commit<'a>(&'a self, tx_id: memfuse_core::TxId) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move {
                self.staged.write().remove(&tx_id);
                Ok(())
            })
        }
        fn rollback<'a>(&'a self, tx_id: memfuse_core::TxId) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move {
                let keys = self.staged.write().remove(&tx_id).unwrap_or_default();
                let mut store = self.store.write();
                for k in keys {
                    if let Some(versions) = store.get_mut(&k) {
                        versions.pop();
                        if versions.is_empty() {
                            store.remove(&k);
                        }
                    }
                }
                Ok(())
            })
        }
        fn rollback_to_tx<'a>(&'a self, _tx_id: memfuse_core::TxId) -> BoxFuture<'a, Result<()>> {
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
                    for (val, v_seq) in versions.iter().rev() {
                        let raw_seq = v_seq & !memfuse_core::TOMBSTONE_BIT;
                        if raw_seq <= seq {
                            if (v_seq & memfuse_core::TOMBSTONE_BIT) != 0 {
                                return Ok(None);
                            }
                            return Ok(Some(bytes::Bytes::from(val.clone())));
                        }
                    }
                }
                Ok(None)
            })
        }
        fn last_seq_no<'a>(&'a self) -> BoxFuture<'a, Result<u64>> {
            Box::pin(async move {
                Ok(self
                    .next_seq
                    .load(std::sync::atomic::Ordering::SeqCst)
                    .saturating_sub(1))
            })
        }
        fn last_tx_id<'a>(&'a self) -> BoxFuture<'a, Result<memfuse_core::TxId>> {
            Box::pin(async move { Ok(memfuse_core::TxId::new(0)) })
        }
        fn flush<'a>(&'a self) -> BoxFuture<'a, Result<()>> {
            Box::pin(async move { Ok(()) })
        }
        fn stats<'a>(&'a self) -> BoxFuture<'a, Result<memfuse_core::StorageStats>> {
            Box::pin(async move {
                Ok(memfuse_core::StorageStats {
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
                let store = self.store.read();
                let mut results = Vec::new();
                for (k, versions) in store.iter() {
                    if k.starts_with(prefix) {
                        for (val, v_seq) in versions.iter().rev() {
                            let raw_seq = v_seq & !memfuse_core::TOMBSTONE_BIT;
                            if raw_seq <= seq_no {
                                if (v_seq & memfuse_core::TOMBSTONE_BIT) == 0 {
                                    results.push((k.clone(), val.clone()));
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

    #[tokio::test]
    async fn test_block_max_wand_equivalence_and_pruning_performance() -> Result<()> {
        let storage = Arc::new(MockStorage::new());
        let index = InvertedIndex::new(storage.clone(), "wand_test");
        let tx = memfuse_core::TxId::new(1);

        // Index 500 documents:
        // "common" occurs in all 500 documents.
        // "rare" occurs in only 5 documents (DocId 100, 200, 300, 400, 500) with high frequency.
        for i in 1..=500 {
            let doc_id = DocId::new(i as u64);
            let text = if i % 100 == 0 {
                format!("common rare rare rare rare rare doc {}", i)
            } else {
                format!("common doc {}", i)
            };
            index.upsert_document(tx, doc_id, &text).await?;
        }
        index.commit_stats(tx).await?;
        storage.commit(tx).await?;

        // 1. Equivalence Test: WAND search results vs full scan expectations
        let k = 5;
        let query_results = index.search_bm25("common rare", k, None).await?;
        assert_eq!(query_results.len(), k);

        // The top 5 documents should be doc_ids 100, 200, 300, 400, 500 because they match "rare" 5x
        let top_doc_ids: Vec<u64> = query_results.iter().map(|(id, _)| id.inner()).collect();
        assert!(top_doc_ids.contains(&100));
        assert!(top_doc_ids.contains(&200));
        assert!(top_doc_ids.contains(&300));
        assert!(top_doc_ids.contains(&400));
        assert!(top_doc_ids.contains(&500));

        // 2. Pruning Performance Test: Verify evaluated_docs_count is significantly less than 500
        let n = index.total_docs.load(std::sync::atomic::Ordering::Acquire);
        let avg_len_x1000 = index
            .avg_doc_len_x1000
            .load(std::sync::atomic::Ordering::Acquire);
        let avg_doc_len = avg_len_x1000 as f32 / 1000.0;
        let terms = vec!["common".to_string(), "rare".to_string()];

        let wand_metrics = block_max_wand_search(
            storage.as_ref(),
            &terms,
            &index.resident_index,
            |term| index.key_term_prefix(term),
            |doc_id, term| index.key_tombstone(doc_id, term),
            |doc_id_inner| index.key_with_id("dl:", doc_id_inner),
            k,
            None,
            n,
            avg_doc_len,
        )
        .await?;

        assert_eq!(wand_metrics.results, query_results);

        // Without WAND, full scan evaluates all 500 documents.
        // With Block-Max WAND, dynamic thresholding skips low-scoring blocks of "common".
        assert!(
            wand_metrics.evaluated_docs_count < 500,
            "Block-Max WAND evaluated {} docs, which should be significantly less than total 500",
            wand_metrics.evaluated_docs_count
        );

        Ok(())
    }
}
