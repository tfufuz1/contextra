//! Autarker, thread-sicherer Inverted-Index mit Snapshot-Isolation für Klasse-A-Betrieb (SPEC §4.3, B1–B12).
//!
//! # Architektur & Nebenläufigkeit
//! - **Lesende Sicht**: Vollständig unveränderlicher Snapshot (`IndexSnapshot`). Leser halten lediglich
//!   einen `Arc<IndexSnapshot>`, wodurch Leseoperationen Schreiber nicht länger als O(1) blockieren.
//! - **Schreiber**: Arbeiten Copy-on-Write (CoW). Alle Aktualisierungen (`insert`, `remove`, `compact`)
//!   erzeugen atomar einen neuen Snapshot mit monoton inkrementierter [`Epoch`].
//! - **Statistikkonsistenz**: Wörterbuch, Posting-Listen und Korpusstatistiken (`N`, `avgdl`, `df`)
//!   sind in jedem Snapshot strikt konsistent und synchronisiert.
//! - **Neutrale Löschung**: `remove` markiert das Dokument per Tombstone; `compact` bereinigt Postings
//!   und Wörterbuch physisch; `vacuum` reorganisiert Speicherstrukturen.

use crate::bm25::BM25;
use crate::contract::{DocKey, Epoch};
use crate::error::{Result, TextError};
use crate::posting_list::{DocIdRaw, Posting};
use crate::tokenizer::{DefaultTokenizer, Tokenizer};
use crate::traits::{Index, Maintain};
use crate::types::{DocId, TextHit, MAX_SEARCH_K};
use parking_lot::RwLock;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// Maximale Dokumentgröße in Bytes (fail-closed, 10 MB).
pub const MAX_DOC_BYTES: usize = 10 * 1024 * 1024;

/// Maximale Anzahl von Begriffen (Tokens) pro Dokument (fail-closed).
pub const MAX_DOC_TERMS: usize = 1_000_000;

/// Maximale Anzahl von Postings pro Term (fail-closed).
pub const MAX_POSTINGS_PER_TERM: usize = 10_000_000;

/// Statistiken über den Kompaktierungsvorgang.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CompactStats {
    /// Anzahl physisch bereinigter Tombstones.
    pub purged_docs: usize,
    /// Anzahl verbliebener aktiver Dokumente.
    pub active_docs: usize,
}

/// Prüfergebnis einer Schlüssel-Inspektion (Feature `inspect`).
#[cfg(feature = "inspect")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyInspection {
    /// Ob der Schlüssel in den aktiven Dokumentlängen vorkommt.
    pub in_doc_lengths: bool,
    /// Ob der Schlüssel aktuell als Tombstone markiert ist.
    pub is_tombstone: bool,
    /// Anzahl der Posting-Einträge, in denen dieser Schlüssel noch auftaucht.
    pub posting_occurrences: usize,
}

/// Unveränderlicher Snapshot des Index für lock-freie, konsistente Lesezugriffe.
#[derive(Clone)]
pub struct IndexSnapshot {
    /// Opake, monotone Epochenmarke dieses Snapshots.
    pub epoch: Epoch,
    /// Invertiertes Wörterbuch mit sortierten Posting-Listen.
    pub postings: BTreeMap<String, Vec<Posting>>,
    /// Dokumentlängen aller aktiven und tombstoned Dokumente.
    pub doc_lengths: BTreeMap<DocKey, u32>,
    /// Menge aller als gelöscht markierten Dokumente (Tombstones).
    pub tombstones: BTreeSet<DocKey>,
    /// Anzahl aktiver Dokumente (`N`).
    pub active_docs: usize,
    /// Durchschnittliche Dokumentenlänge (`avgdl`).
    pub avg_doc_len: f32,
}

impl IndexSnapshot {
    /// Erzeugt einen leeren Snapshot mit Anfangsepoche 0.
    fn empty() -> Self {
        Self {
            epoch: Epoch(0),
            postings: BTreeMap::new(),
            doc_lengths: BTreeMap::new(),
            tombstones: BTreeSet::new(),
            active_docs: 0,
            avg_doc_len: 1.0,
        }
    }

    /// Sucht im unveränderlichen Snapshot anhand von BM25.
    pub fn search(&self, query_tokens: &[String], scorer: &BM25, limit: usize) -> Vec<TextHit> {
        if query_tokens.is_empty() || limit == 0 || self.active_docs == 0 {
            return Vec::new();
        }

        let total_docs = self.active_docs;
        let avg_doc_len = self.avg_doc_len;
        let mut scores: BTreeMap<DocKey, f32> = BTreeMap::new();

        for term in query_tokens {
            if let Some(posting_list) = self.postings.get(term) {
                // Berechne df (nur über aktive Dokumente)
                let df = posting_list
                    .iter()
                    .filter(|p| !self.tombstones.contains(&DocKey(p.doc_id as u128)))
                    .count();

                if df == 0 {
                    continue;
                }

                for p in posting_list {
                    let key = DocKey(p.doc_id as u128);
                    if self.tombstones.contains(&key) {
                        continue;
                    }
                    let score = scorer.score_term(
                        p.tf,
                        p.doc_len,
                        avg_doc_len,
                        df as u32,
                        total_docs as u32,
                    );
                    *scores.entry(key).or_insert(0.0) += score;
                }
            }
        }

        let mut hits: Vec<TextHit> = scores
            .into_iter()
            .map(|(key, score)| TextHit {
                doc_id: DocId::new(key.0 as _),
                score,
            })
            .collect();

        // Sortierung: Score absteigend, bei Gleichstand DocId aufsteigend (Determinismus B11)
        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.doc_id.cmp(&b.doc_id))
        });

        if hits.len() > limit {
            hits.truncate(limit);
        }

        hits
    }
}

/// Autarker, thread-sicherer In-Memory Volltextindex (Klasse A).
///
/// Bietet die standardisierte Index-Schnittstelle gemäß SPEC §4.3 (`insert`, `remove`, `search`)
/// sowie neutrale Wartung (`compact`, `vacuum`) mit Copy-on-Write Snapshot-Isolation.
pub struct MemoryTextIndex {
    tokenizer: Arc<dyn Tokenizer>,
    scorer: BM25,
    snapshot: RwLock<Arc<IndexSnapshot>>,
}

impl Default for MemoryTextIndex {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryTextIndex {
    /// Erzeugt einen neuen `MemoryTextIndex` mit Standard-Tokenizer und BM25-Scorer.
    pub fn new() -> Self {
        Self {
            tokenizer: Arc::new(DefaultTokenizer),
            scorer: BM25::default(),
            snapshot: RwLock::new(Arc::new(IndexSnapshot::empty())),
        }
    }

    /// Konfiguriert einen spezifischen Tokenizer.
    pub fn with_tokenizer(mut self, tokenizer: Arc<dyn Tokenizer>) -> Self {
        self.tokenizer = tokenizer;
        self
    }

    /// Liefert einen unveränderlichen Snapshot des Index (O(1) Lesezugriff).
    pub fn snapshot(&self) -> Arc<IndexSnapshot> {
        self.snapshot.read().clone()
    }

    /// Liefert die aktuelle Epoche des Index.
    pub fn current_epoch(&self) -> Epoch {
        self.snapshot.read().epoch
    }

    /// Liefert die Anzahl aktiver Dokumente im Index.
    pub fn len(&self) -> usize {
        self.snapshot.read().active_docs
    }

    /// Gibt an, ob der Index keine aktiven Dokumente enthält.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Rohsicht-Inspektion unter dem Feature `inspect`.
    #[cfg(feature = "inspect")]
    pub fn inspect_key(&self, key: &DocKey) -> KeyInspection {
        let snap = self.snapshot.read();
        let in_doc_lengths = snap.doc_lengths.contains_key(key);
        let is_tombstone = snap.tombstones.contains(key);
        let target_raw = key.0 as DocIdRaw;
        let mut occurrences = 0;
        for list in snap.postings.values() {
            if list.iter().any(|p| p.doc_id == target_raw) {
                occurrences += 1;
            }
        }
        KeyInspection {
            in_doc_lengths,
            is_tombstone,
            posting_occurrences: occurrences,
        }
    }

    /// Rohsicht-Inspektion aller Postings unter dem Feature `inspect`.
    #[cfg(feature = "inspect")]
    pub fn inspect_postings(&self) -> BTreeMap<String, Vec<Posting>> {
        self.snapshot.read().postings.clone()
    }

    /// Rohsicht-Inspektion der Dokumentlängen unter dem Feature `inspect`.
    #[cfg(feature = "inspect")]
    pub fn inspect_doc_lengths(&self) -> BTreeMap<DocKey, u32> {
        self.snapshot.read().doc_lengths.clone()
    }
}

impl Index<DocKey, str> for MemoryTextIndex {
    type Hit = TextHit;

    fn insert(&mut self, id: DocKey, payload: &[u8]) -> Result<()> {
        if payload.len() > MAX_DOC_BYTES {
            return Err(TextError::CapacityExceeded {
                requested: payload.len(),
                maximum: MAX_DOC_BYTES,
            });
        }

        let text = std::str::from_utf8(payload)
            .map_err(|e| TextError::InvalidInput(format!("UTF-8 Fehler: {e}")))?;

        let tokens = self.tokenizer.tokenize(text);
        if tokens.len() > MAX_DOC_TERMS {
            return Err(TextError::CapacityExceeded {
                requested: tokens.len(),
                maximum: MAX_DOC_TERMS,
            });
        }
        let doc_len = tokens.len() as u32;

        let mut tfs: BTreeMap<String, u32> = BTreeMap::new();
        for t in tokens {
            *tfs.entry(t).or_insert(0) += 1;
        }

        let raw_id = id.0 as DocIdRaw;

        // Copy-on-Write: Aktuellen Snapshot duplizieren und modifizieren
        let current = self.snapshot.read().clone();
        let mut new_postings = current.postings.clone();
        let mut new_doc_lengths = current.doc_lengths.clone();
        let mut new_tombstones = current.tombstones.clone();

        // Tombstone für diese Id aufheben
        new_tombstones.remove(&id);

        for (term, tf) in tfs {
            let list = new_postings.entry(term).or_default();
            if let Some(pos) = list.iter().position(|p| p.doc_id == raw_id) {
                list[pos] = Posting {
                    doc_id: raw_id,
                    tf,
                    doc_len,
                };
            } else {
                if list.len() >= MAX_POSTINGS_PER_TERM {
                    return Err(TextError::CapacityExceeded {
                        requested: list.len() + 1,
                        maximum: MAX_POSTINGS_PER_TERM,
                    });
                }
                list.push(Posting {
                    doc_id: raw_id,
                    tf,
                    doc_len,
                });
                list.sort_by_key(|p| p.doc_id);
            }
        }

        new_doc_lengths.insert(id, doc_len);

        // Konsistente Neuberechnung von N und avgdl
        let active_docs = new_doc_lengths
            .keys()
            .filter(|k| !new_tombstones.contains(k))
            .count();
        let total_len: u64 = new_doc_lengths
            .iter()
            .filter(|(k, _)| !new_tombstones.contains(k))
            .map(|(_, len)| *len as u64)
            .sum();
        let avg_doc_len = if active_docs > 0 {
            (total_len as f32) / (active_docs as f32)
        } else {
            1.0
        };

        let new_epoch = Epoch(current.epoch.0.saturating_add(1));
        let new_snapshot = Arc::new(IndexSnapshot {
            epoch: new_epoch,
            postings: new_postings,
            doc_lengths: new_doc_lengths,
            tombstones: new_tombstones,
            active_docs,
            avg_doc_len,
        });

        *self.snapshot.write() = new_snapshot;
        Ok(())
    }

    fn remove(&mut self, id: &DocKey) -> Result<bool> {
        let current = self.snapshot.read().clone();
        if current.doc_lengths.contains_key(id) && !current.tombstones.contains(id) {
            let mut new_tombstones = current.tombstones.clone();
            new_tombstones.insert(*id);

            let active_docs = current
                .doc_lengths
                .keys()
                .filter(|k| !new_tombstones.contains(k))
                .count();
            let total_len: u64 = current
                .doc_lengths
                .iter()
                .filter(|(k, _)| !new_tombstones.contains(k))
                .map(|(_, len)| *len as u64)
                .sum();
            let avg_doc_len = if active_docs > 0 {
                (total_len as f32) / (active_docs as f32)
            } else {
                1.0
            };

            let new_epoch = Epoch(current.epoch.0.saturating_add(1));
            let new_snapshot = Arc::new(IndexSnapshot {
                epoch: new_epoch,
                postings: current.postings.clone(),
                doc_lengths: current.doc_lengths.clone(),
                tombstones: new_tombstones,
                active_docs,
                avg_doc_len,
            });

            *self.snapshot.write() = new_snapshot;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn search(&self, query: &str, limit: usize) -> Result<Vec<Self::Hit>> {
        let bounded_limit = limit.min(MAX_SEARCH_K);
        if query.trim().is_empty() || bounded_limit == 0 {
            return Ok(Vec::new());
        }

        if query.len() > MAX_DOC_BYTES {
            return Err(TextError::CapacityExceeded {
                requested: query.len(),
                maximum: MAX_DOC_BYTES,
            });
        }

        let snap = self.snapshot.read().clone();
        let query_tokens = self.tokenizer.tokenize(query);
        Ok(snap.search(&query_tokens, &self.scorer, bounded_limit))
    }
}

// Auch Implementierung für DocId zur Abwärtskompatibilität bereitstellen
impl Index<DocId, str> for MemoryTextIndex {
    type Hit = TextHit;

    fn insert(&mut self, id: DocId, payload: &[u8]) -> Result<()> {
        Index::<DocKey, str>::insert(self, DocKey(id.inner() as u128), payload)
    }

    fn remove(&mut self, id: &DocId) -> Result<bool> {
        Index::<DocKey, str>::remove(self, &DocKey(id.inner() as u128))
    }

    fn search(&self, query: &str, limit: usize) -> Result<Vec<Self::Hit>> {
        Index::<DocKey, str>::search(self, query, limit)
    }
}

impl Maintain for MemoryTextIndex {
    type Stats = CompactStats;

    fn compact(&mut self) -> Result<Self::Stats> {
        let current = self.snapshot.read().clone();
        let purged_count = current.tombstones.len();
        if purged_count == 0 {
            return Ok(CompactStats {
                purged_docs: 0,
                active_docs: current.active_docs,
            });
        }

        let mut new_doc_lengths = current.doc_lengths.clone();
        for dead_id in &current.tombstones {
            new_doc_lengths.remove(dead_id);
        }

        let mut new_postings = current.postings.clone();
        for list in new_postings.values_mut() {
            list.retain(|p| !current.tombstones.contains(&DocKey(p.doc_id as u128)));
        }
        new_postings.retain(|_, list| !list.is_empty());

        let active_docs = new_doc_lengths.len();
        let total_len: u64 = new_doc_lengths.values().map(|len| *len as u64).sum();
        let avg_doc_len = if active_docs > 0 {
            (total_len as f32) / (active_docs as f32)
        } else {
            1.0
        };

        let new_epoch = Epoch(current.epoch.0.saturating_add(1));
        let new_snapshot = Arc::new(IndexSnapshot {
            epoch: new_epoch,
            postings: new_postings,
            doc_lengths: new_doc_lengths,
            tombstones: BTreeSet::new(),
            active_docs,
            avg_doc_len,
        });

        *self.snapshot.write() = new_snapshot;

        Ok(CompactStats {
            purged_docs: purged_count,
            active_docs,
        })
    }

    fn vacuum(&mut self) -> Result<()> {
        let _ = self.compact()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_text_index_crud_and_compact() {
        let mut index = MemoryTextIndex::new();
        let doc1 = DocKey::from(1u64);
        let doc2 = DocKey::from(2u64);

        index.insert(doc1, b"apple orange banana").unwrap();
        index.insert(doc2, b"banana melon kiwi").unwrap();

        assert_eq!(index.len(), 2);
        assert_eq!(index.current_epoch(), Epoch(2));

        let results = Index::<DocKey, str>::search(&index, "banana", 10).unwrap();
        assert_eq!(results.len(), 2);

        // Snapshot isolieren
        let snap1 = index.snapshot();
        assert_eq!(snap1.active_docs, 2);

        // Entfernen von doc1 setzt Tombstone
        assert!(index.remove(&doc1).unwrap());
        assert_eq!(index.len(), 1);
        assert_eq!(index.current_epoch(), Epoch(3));

        // Alter Snapshot sieht doc1 immer noch
        let old_results = snap1.search(&["banana".into()], &BM25::default(), 10);
        assert_eq!(old_results.len(), 2);

        // Neue Suche sieht doc1 nicht mehr
        let results_after = Index::<DocKey, str>::search(&index, "banana", 10).unwrap();
        assert_eq!(results_after.len(), 1);
        assert_eq!(results_after[0].doc_id, DocId::new(2));

        // Compact ausführen
        let stats = index.compact().unwrap();
        assert_eq!(stats.purged_docs, 1);
        assert_eq!(stats.active_docs, 1);
        assert_eq!(index.current_epoch(), Epoch(4));

        // Nach compact bleibt Suche konsistent
        let results_compact = Index::<DocKey, str>::search(&index, "banana", 10).unwrap();
        assert_eq!(results_compact.len(), 1);
        assert_eq!(results_compact[0].doc_id, DocId::new(2));
    }
}
