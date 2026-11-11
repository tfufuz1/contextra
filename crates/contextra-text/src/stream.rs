// FILE-CONTEXT: On-demand candidate streaming for BM25 search.
// ZWECK: Liefert BM25-Kandidaten in Batches à 16 (oder konfigurierbar) via geometrisches Nachladen.
// INVARIANTEN: #![forbid(unsafe_code)], zero panic/unwrap, Snapshot-Isolation via max_seq, max_depth = MAX_SEARCH_K.
// STAND: TS:2026-09-14T00:00:00Z

//! Asynchrones, bedarfsgesteuertes Streaming von BM25-Suchkandidaten.
//!
//! # Ring-Zuordnung
//! Ring 0 (Kern).
//!
//! # Funktionsweise
//! Stellt mit [`Bm25CandidateStream`] einen Pull-basierten Iterator bereit, der
//! Trefferdokumente paketweise (standardmäßig in 16er-Schritten) aus dem invertierten
//! Index nachlädt. Dadurch muss bei großen Ergebnismengen nicht sofort die maximale
//! Trefferzahl (`MAX_SEARCH_K`) ermittelt werden.

use crate::error::Result;
use crate::storage::PostingStore;
use crate::types::MAX_SEARCH_K;
use contextra_ids::DocId;
use std::collections::{BTreeSet, VecDeque};

use crate::inverted::InvertedIndex;

// ANCHOR[contextra-text:STREAM-001]
/// Standard-Batch-Größe für das Nachladen von Kandidaten (16 Dokumente).
pub const DEFAULT_STREAM_BATCH_SIZE: usize = 16;

// ANCHOR[contextra-text:STREAM-002]
/// Asynchroner Pull-Stream zum bedarfsgerechten Nachladen von BM25-Kandidaten.
pub struct Bm25CandidateStream<'a, S: PostingStore> {
    index: &'a InvertedIndex<S>,
    query: String,
    max_seq: Option<u64>,
    batch_size: usize,
    buffer: VecDeque<(DocId, f32)>,
    seen: BTreeSet<DocId>,
    fetched_k: usize,
    yielded: usize,
    exhausted: bool,
}

impl<'a, S: PostingStore> Bm25CandidateStream<'a, S> {
    // ANCHOR[contextra-text:STREAM-003]
    /// Erzeugt einen neuen Kandidaten-Stream für die übergebene Abfrage und Index-Snapshot.
    pub fn new(
        index: &'a InvertedIndex<S>,
        query: impl Into<String>,
        max_seq: Option<u64>,
    ) -> Self {
        Self {
            index,
            query: query.into(),
            max_seq,
            batch_size: DEFAULT_STREAM_BATCH_SIZE,
            buffer: VecDeque::new(),
            seen: BTreeSet::new(),
            fetched_k: 0,
            yielded: 0,
            exhausted: false,
        }
    }

    // ANCHOR[contextra-text:STREAM-004]
    /// Konfiguriert eine benutzerdefinierte Batch-Größe (mindestens 1).
    pub fn with_batch_size(mut self, n: usize) -> Self {
        self.batch_size = n.max(1);
        self
    }

    // ANCHOR[contextra-text:STREAM-005]
    /// Holt den nächsten Batch von Kandidaten-Dokumenten (bis zu `batch_size`).
    pub async fn next_batch(&mut self) -> Result<Vec<(DocId, f32)>> {
        while self.buffer.len() < self.batch_size && !self.exhausted {
            self.refill().await?;
        }

        let drain_count = self.batch_size.min(self.buffer.len());
        let mut batch = Vec::with_capacity(drain_count);
        for _ in 0..drain_count {
            if let Some(item) = self.buffer.pop_front() {
                batch.push(item);
            }
        }
        self.yielded = self.yielded.saturating_add(batch.len());
        Ok(batch)
    }

    // ANCHOR[contextra-text:STREAM-006]
    /// Holt die nächste einzelne Dokument-ID aus dem Stream.
    pub async fn next_doc(&mut self) -> Result<Option<DocId>> {
        if self.buffer.is_empty() && !self.exhausted {
            self.refill().await?;
        }

        if let Some((doc_id, _score)) = self.buffer.pop_front() {
            self.yielded = self.yielded.saturating_add(1);
            Ok(Some(doc_id))
        } else {
            Ok(None)
        }
    }

    // ANCHOR[contextra-text:STREAM-007]
    /// Liefert die Gesamtzahl aller bisher an den Aufrufer herausgegebenen Treffer.
    pub fn yielded(&self) -> usize {
        self.yielded
    }

    // ANCHOR[contextra-text:STREAM-008]
    /// Prüft, ob der Stream vollständig erschöpft ist (keine gepufferten oder nachladbaren Elemente mehr).
    pub fn is_exhausted(&self) -> bool {
        self.exhausted && self.buffer.is_empty()
    }

    // ANCHOR[contextra-text:STREAM-009]
    /// Liefert die maximale Suchtiefe für Kandidaten ([`MAX_SEARCH_K`]).
    pub const fn max_depth() -> usize {
        MAX_SEARCH_K
    }

    // ANCHOR[contextra-text:STREAM-010]
    async fn refill(&mut self) -> Result<()> {
        if self.exhausted {
            return Ok(());
        }

        let target_k = self
            .batch_size
            .max(self.fetched_k.saturating_mul(2))
            .min(MAX_SEARCH_K);

        let candidates = self
            .index
            .search_bm25_at(&self.query, target_k, self.max_seq)
            .await?;

        let res_len = candidates.len();
        let mut new_items = 0usize;

        for (doc_id, score) in candidates {
            if self.seen.insert(doc_id) {
                self.buffer.push_back((doc_id, score));
                new_items = new_items.saturating_add(1);
            }
        }

        self.fetched_k = target_k;

        if res_len < target_k || (target_k == MAX_SEARCH_K && new_items == 0) {
            self.exhausted = true;
        }

        Ok(())
    }
}
