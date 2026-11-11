// FILE-CONTEXT: Block-Max WAND (Weak-AND) BM25 Query Execution Engine (IP-10).
// ZWECK: Multi-Term Query-Traversierung mit Block-Maximalwert-Pruning und dynamic thresholding.
// INVARIANTEN: Zero-Panic Doctrine, Safe Optimization (Ergebnisse identisch zu Full-Scan), MVCC & Tombstone Isolation.

//! Block-Max WAND (Weak AND) Algorithmus für performante Multi-Term BM25-Suchen.
//!
//! # Ring-Zuordnung
//! Ring 0 (Kern).
//!
//! # Funktionsweise
//! WAND vermeidet das vollständige Berechnen von BM25-Scores für alle Dokumente,
//! indem anhand vorberechneter Obergrenzen (`term_max_score` und `block_max_scores`)
//! Terme und Dokumentblöcke dynamisch übersprungen werden, deren potentieller
//! Maximalscore den aktuellen Schwellenwert der Top-K-Treffer nicht übersteigen kann.

use crate::bm25::score_term;
use crate::posting_list::{DocIdRaw, Posting, PostingList, BLOCK_SIZE};
use crate::storage::PostingStore;
use crate::error::{Result, TextError};
use crate::types::{DocId, MAX_SEARCH_K};
use std::collections::{BTreeMap, BinaryHeap};
use std::sync::Arc;

// ANCHOR[contextra-text:WAND-001]
/// Interner BM25-Kandidat für die Top-K-Min-Heap-Verwaltung.
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

// ANCHOR[contextra-text:WAND-002]
/// Begrenzter Min-Heap zur effizienten Verwaltung der K besten Treffer.
pub struct BoundedTopK {
    heap: BinaryHeap<Bm25Candidate>,
    capacity: usize,
}

impl BoundedTopK {
    // ANCHOR[contextra-text:WAND-003]
    /// Erzeugt einen neuen `BoundedTopK`-Puffer mit maximaler Kapazität (`MAX_SEARCH_K`).
    pub fn new(capacity: usize) -> Self {
        let capacity = capacity.min(MAX_SEARCH_K);
        Self {
            heap: BinaryHeap::with_capacity(capacity.saturating_add(1)),
            capacity,
        }
    }

    // ANCHOR[contextra-text:WAND-004]
    /// Fügt einen neuen Kandidaten ein, wenn er besser als der schlechteste bisherige Treffer ist.
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

    // ANCHOR[contextra-text:WAND-005]
    /// Liefert den aktuellen Minimal-Score (Schwelle), den ein Kandidat übertreffen muss.
    pub fn min_threshold(&self) -> f32 {
        if self.heap.len() < self.capacity {
            0.0
        } else {
            self.heap.peek().map(|c| c.score).unwrap_or(0.0)
        }
    }

    // ANCHOR[contextra-text:WAND-006]
    /// Konvertiert den Heap in einen nach Relevanz absteigend sortierten Ergebnisvektor.
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
    fn current_doc_id(&self) -> Option<DocIdRaw> {
        self.current_posting().map(|p| p.doc_id)
    }

    #[inline]
    fn block_max_score_at_doc_id(&self, doc_id: DocIdRaw) -> f32 {
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
    fn advance_to(&mut self, target_doc_id: DocIdRaw) {
        let postings = self.posting_list.as_slice();
        if self.cursor >= postings.len() {
            return;
        }

        // Skip blocks whose max_doc_id is strictly less than target_doc_id
        let blocks = self.posting_list.blocks();
        let mut block_idx = self.cursor / BLOCK_SIZE;

        while block_idx < blocks.len()
            && blocks
                .get(block_idx)
                .is_some_and(|b| b.max_doc_id < target_doc_id)
        {
            block_idx += 1;
        }

        let new_cursor_start = block_idx * BLOCK_SIZE;
        if new_cursor_start >= postings.len() {
            self.cursor = postings.len();
            return;
        }

        self.cursor = new_cursor_start.max(self.cursor);

        // Binary search within current block / remaining postings
        if let Some(slice) = postings.get(self.cursor..) {
            match slice.binary_search_by_key(&target_doc_id, |p| p.doc_id) {
                Ok(offset) => self.cursor += offset,
                Err(offset) => self.cursor += offset,
            }
        }
    }
}

// ANCHOR[contextra-text:WAND-007]
/// Ergebnis einer Block-Max-WAND-Suche inklusive Top-K-Treffern und Metadaten über evaluierte Dokumente.
#[derive(Debug)]
pub struct WandSearchResult {
    /// Gefundene Trefferdokumente mit Relevanzwert.
    pub results: Vec<(DocId, f32)>,
    /// Anzahl tatsächlich voll berechneter Dokumente (Pruning-Effektivitätsmetrik).
    pub evaluated_docs_count: usize,
}

// ANCHOR[contextra-text:WAND-008]
/// Führt den Block-Max WAND-Algorithmus über speicherresidenten Posting-Listen aus.
///
/// # Ablauf
/// 1. Snapshot-Sequenznummer auflösen und aktive Tombstones vorladen.
/// 2. Cursors für alle Abfrageterme initialisieren und obere Schranken vorberechnen.
/// 3. In einer Schleife:
///    a) Cursors nach aktueller DocId sortieren.
///    b) Pivot-Term auswählen, dessen kumulierter Max-Score die aktuelle Top-K-Schwelle übersteigt.
///    c) Falls Min-DocId == Pivot-DocId: Block-Max-Schranke prüfen, exakten BM25-Score berechnen und in Top-K aufnehmen.
///    d) Andernfalls Cursors zur Pivot-DocId vorspringen lassen (Skip).
// ADR-0015: Block-Max WAND query search signature requires storage, index, and query parameters.
#[allow(clippy::too_many_arguments)]
pub async fn block_max_wand_search<S: PostingStore>(
    storage: &S,
    terms: &[String],
    resident_index: &crate::posting_list::ResidentPostingIndex,
    prefix_helper: impl Fn(&str) -> Vec<u8>,
    tombstone_key_helper: impl Fn(DocId, &str) -> Vec<u8>,
    doc_len_key_helper: impl Fn(u128) -> Vec<u8>,
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
    let total_docs_u32 = u32::try_from(total_docs).unwrap_or(u32::MAX).max(1);

    let mut doc_len_cache: BTreeMap<DocId, Option<u32>> = BTreeMap::new();
    let mut tbs_cache: BTreeMap<Vec<u8>, bool> = BTreeMap::new();

    // Prefetch active tombstones globally at snapshot `seq` in a single prefix scan
    let tbs_global_prefix = tombstone_key_helper(DocId::new(0), "")
        .into_iter()
        .take_while(|&b| b != b'0')
        .collect::<Vec<u8>>();
    let active_tbs_entries = storage.scan_prefix_at(&tbs_global_prefix, seq).await?;
    for (tbs_key, _) in &active_tbs_entries {
        tbs_cache.insert(tbs_key.clone(), true);
    }

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
                    if let Some(prefix_sub) = prefix.get(..pos) {
                        batch_key.extend_from_slice(prefix_sub);
                    }
                    batch_key.extend_from_slice(b"plb:");
                    let remaining = prefix.get(pos.saturating_add(3)..).unwrap_or(&[]);
                    if remaining.ends_with(b":") {
                        if let Some(rem_sub) = remaining.get(..remaining.len().saturating_sub(1)) {
                            batch_key.extend_from_slice(rem_sub);
                        }
                    } else {
                        batch_key.extend_from_slice(remaining);
                    }
                } else {
                    batch_key = prefix.clone();
                }

                let loaded_list = if let Some(bytes) = storage.get_at_seq(&batch_key, seq).await? {
                    PostingList::decode_compact(&bytes).ok()
                } else {
                    None
                };

                let plist = match loaded_list {
                    Some(l) => l,
                    None => {
                        let raw_entries = storage.scan_prefix_at(&prefix, u64::MAX).await?;
                        let mut postings = Vec::with_capacity(raw_entries.len());
                        for (key, val_bytes) in raw_entries {
                            let suffix_bytes = key.get(prefix.len()..).unwrap_or(&[]);
                            if let Ok(suffix) = std::str::from_utf8(suffix_bytes) {
                                #[cfg(not(feature = "docid-128"))]
                                let parsed_doc_id = suffix.parse::<u64>().ok();
                                #[cfg(feature = "docid-128")]
                                let parsed_doc_id = suffix.parse::<u128>().ok();

                                if let Some(doc_id_raw) = parsed_doc_id {
                                    if val_bytes.len() == 4 {
                                        let doc_id = DocId::new(doc_id_raw as _);
                                        let tf_bytes = val_bytes.get(..4).ok_or_else(|| {
                                            TextError::Storage(
                                                "Invalid val_bytes length".into(),
                                            )
                                        })?;
                                        let tf = u32::from_le_bytes(tf_bytes.try_into().map_err(
                                            |_| {
                                                TextError::Storage(
                                                    "Invalid posting tf length".into(),
                                                )
                                            },
                                        )?);
                                        let dl_key = doc_len_key_helper(doc_id.inner().into());
                                        let doc_len = match storage.get(&dl_key).await? {
                                            Some(dl_bytes) if dl_bytes.len() == 4 => {
                                                u32::from_le_bytes(
                                                    (&dl_bytes[..]).try_into().map_err(|_| {
                                                        TextError::Storage(
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
            // Count active tombstones for `term` at snapshot `seq` from pre-cached global tombstones
            let mut term_tbs_count = 0u32;
            let term_suffix = format!(":{}", term);
            for (tbs_key, _) in &active_tbs_entries {
                if let Ok(key_str) = std::str::from_utf8(tbs_key) {
                    if key_str.ends_with(&term_suffix) {
                        // Extract doc_id from tombstone key: {prefix}tbs:{doc_id}:{term}
                        let suffix = tbs_key.get(tbs_global_prefix.len()..).unwrap_or(&[]);
                        if let Some(colon_pos) = suffix.iter().position(|&b| b == b':') {
                            if let Some(doc_bytes) = suffix.get(..colon_pos) {
                                if let Ok(doc_id_str) = std::str::from_utf8(doc_bytes) {
                                    #[cfg(not(feature = "docid-128"))]
                                    let parsed_doc_id = doc_id_str.parse::<u64>().ok();
                                    #[cfg(feature = "docid-128")]
                                    let parsed_doc_id = doc_id_str.parse::<u128>().ok();

                                    if let Some(doc_id_raw) = parsed_doc_id {
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
            }

            let effective_df = u32::try_from(list.len())
                .unwrap_or(u32::MAX)
                .saturating_sub(term_tbs_count);

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
        cursors.sort_by(|a, b| {
            a.current_doc_id()
                .unwrap_or(DocIdRaw::MAX)
                .cmp(&b.current_doc_id().unwrap_or(DocIdRaw::MAX))
                .then_with(|| a.term.cmp(&b.term))
        });

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
            let doc_id = DocId::new(pivot_doc_id as _);

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
                let dl_key = doc_len_key_helper(doc_id.inner().into());
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

                        let is_posting_valid = if !is_latest {
                            let prefix = prefix_helper(&c.term);
                            let mut key = Vec::with_capacity(prefix.len() + 20);
                            key.extend_from_slice(&prefix);
                            let mut itoa_buf = itoa::Buffer::new();
                            key.extend_from_slice(itoa_buf.format(doc_id.inner()).as_bytes());
                            storage.get_at_seq(&key, seq).await?.is_some()
                        } else {
                            true
                        };

                        if !is_tombstone && is_posting_valid {
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

