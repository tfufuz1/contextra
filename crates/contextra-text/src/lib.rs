#![deny(dead_code)]
// FILE-CONTEXT: Layer 1 text search integration facade (contextra-text).
// ZWECK: Exportiert Bm25Scorer, InvertedIndex, Morphologie-Tools und BM25-Modelle für DB-Hybrid-Suche.
// INVARIANTEN: #![forbid(unsafe_code)], TextIndex-Trait Implementierung ist fully async & transaction-aware.
// NICHT-OFFENSICHTLICH: Scorer delegiert direkt an InvertedIndex; MVCC & Lock-Free Storage durch StorageEngine.
// HOTSPOTS: Bm25Scorer::search, Bm25Scorer::insert
// STAND: TS:2026-09-09T13:22:45Z (SESSION: dc71d70)

//! BM25-Volltextindex als verschlüsselte Segmente mit deutschem Analyzer.
//!
//! # Ring und Abhängigkeiten
//! Ring 2. Erlaubte Workspace-Abhängigkeiten: `contextra-types`, `contextra-ports`, `contextra-crypto` (SPEC §5.2, §5.3, D-88).
//! Konsument: `contextra-engine`.
//!
//! # Hauptkomponenten
//! - Invertierte Textindizes und segmentbasierter BM25-Index
//! - Deutscher Analyzer `de/1` mit Kompositazerlegung und normalisiertem Vokabular
//! - Block-Max WAND Pruning auf 128er-Blöcken

#![forbid(unsafe_code)]
// Begründung: Unit-Tests in src/ dürfen für Test-Assertions und Test-Fixtures Test-Helfer verwenden.
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::unreachable,
        clippy::disallowed_methods,
        clippy::disallowed_types,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::unnecessary_cast,
    )
)]

pub mod analyzer;
pub mod bm25;
pub mod buffer;
pub mod domain;
pub mod erasure;
pub mod inverted;
pub mod morphology;
pub mod phrase;
pub mod posting_list;
pub mod postings;
pub mod segment;
pub mod stats;
pub mod stream;
pub mod tokenizer;
pub mod wand;

pub use analyzer::{
    compute_term_tag, get_analyzer, Analyzer, GermanAnalyzer, UndAnalyzer, UndeterminedAnalyzer,
};
pub use buffer::{open_binding_envelope, seal_binding_envelope, TextBufRecord, TextBuffer};
pub use erasure::TextErasureLayer;
pub use phrase::match_phrase_positions;
pub use postings::PostingBlock;
pub use segment::{
    DictEntry, DocEntry, SegmentHeader, SegmentReader, SegmentWriter, POSTING_BLOCK_SIZE,
};
pub use stats::SegmentStats;

pub use bm25::{score_term_bm25f, FieldId, FieldWeight, BM25, BM25F};
pub use domain::{DomainVocabulary, LegalDomainVocabulary, MedicalDomainVocabulary};
pub use inverted::{
    BM25MorphIndex, InvertedIndex, Language, ParsedTextTarget, PurgeReport, PurgeStats, ScanTarget,
    TextPurger,
};
pub use morphology::{normalize_umlauts, GermanCompoundSplitter, MorphologicalTokenizer};
pub use posting_list::{Posting, PostingList, ResidentPostingIndex};
pub use stream::{Bm25CandidateStream, DEFAULT_STREAM_BATCH_SIZE};
pub use tokenizer::{DefaultTokenizer, GermanMorphTokenizer, Tokenizer};

use contextra_ports::{TextIndex, TextIndexStats};
use contextra_types::{DocId, Result, ScoredDocument, TxId};

// ANCHOR[contextra-text:LIB-001]
/// Berechnet Begriffsgewichte und wendet die Standard-BM25-Relevanzbewertung an.
pub struct Bm25Scorer<S: contextra_ports::StorageEngine> {
    index: InvertedIndex<S>,
}

impl<S: contextra_ports::StorageEngine> Bm25Scorer<S> {
    // ANCHOR[contextra-text:LIB-002]
    pub fn new(storage: std::sync::Arc<S>, namespace: &str) -> Self {
        Self {
            index: InvertedIndex::new(storage, namespace),
        }
    }
}

impl<S: contextra_ports::StorageEngine> TextIndex for Bm25Scorer<S> {
    // ANCHOR[contextra-text:LIB-003]
    async fn search(&self, query: &str, k: usize) -> Result<Vec<ScoredDocument>> {
        self.index.search(query, k).await
    }

    // ANCHOR[contextra-text:LIB-004]
    async fn search_at(&self, query: &str, k: usize, seq_no: u64) -> Result<Vec<ScoredDocument>> {
        self.index.search_at(query, k, seq_no).await
    }

    // ANCHOR[contextra-text:LIB-005]
    async fn insert(&self, tx: TxId, id: DocId, text: &str) -> Result<()> {
        self.index.insert(tx, id, text).await
    }

    // ANCHOR[contextra-text:LIB-006]
    async fn delete(&self, tx: TxId, id: DocId) -> Result<()> {
        self.index.delete(tx, id).await
    }

    // ANCHOR[contextra-text:LIB-007]
    async fn commit(&self, tx: TxId) -> Result<()> {
        self.index.commit(tx).await
    }

    // ANCHOR[contextra-text:LIB-008]
    async fn rollback(&self, tx: TxId) -> Result<()> {
        self.index.rollback(tx).await
    }

    // ANCHOR[contextra-text:LIB-009]
    async fn rollback_to_tx(&self, tx_id: TxId) -> Result<()> {
        self.index.rollback_to_tx(tx_id).await
    }

    // ANCHOR[contextra-text:LIB-010]
    async fn last_tx_id(&self) -> Result<TxId> {
        self.index.last_tx_id().await
    }

    // ANCHOR[contextra-text:LIB-011]
    async fn len(&self) -> usize {
        self.index.len().await
    }

    // ANCHOR[contextra-text:LIB-012]
    async fn stats(&self) -> Result<TextIndexStats> {
        self.index.stats().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use contextra_ports::BoxFuture;
    use contextra_ports::{StorageEngine, StorageStats};
    use std::collections::BTreeMap;
    use std::sync::Mutex;

    struct MockStorage {
        data: Mutex<BTreeMap<Vec<u8>, Vec<u8>>>,
    }

    impl MockStorage {
        fn new() -> Self {
            Self {
                data: Mutex::new(BTreeMap::new()),
            }
        }
    }

    impl StorageEngine for MockStorage {
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

    // ANCHOR[contextra-text:TEST-LIB-001]
    #[tokio::test]
    async fn bm25_scorer_case_full_lifecycle() -> Result<()> {
        let storage = std::sync::Arc::new(MockStorage::new());
        let scorer = Bm25Scorer::new(storage, "test_ns");

        let tx = TxId::new(1);
        let doc_id = DocId::from(100u64);

        scorer.insert(tx, doc_id, "rust text search engine").await?;
        scorer.commit(tx).await?;

        assert_eq!(scorer.len().await, 1);

        let results = scorer.search("rust engine", 10).await?;
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].doc_id, doc_id);

        let stats = scorer.stats().await?;
        assert_eq!(stats.num_documents, 1);
        assert_eq!(scorer.last_tx_id().await?, TxId(1));

        scorer.delete(tx, doc_id).await?;
        scorer.commit(tx).await?;
        assert_eq!(scorer.len().await, 0);

        scorer.rollback(tx).await?;
        scorer.rollback_to_tx(tx).await?;

        Ok(())
    }
}
