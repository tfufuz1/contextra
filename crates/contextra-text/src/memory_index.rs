//! Autarker In-Memory Textindex für Klasse-A-Betrieb (SPEC §4.3, B1–B12).

use crate::bm25::BM25;
use crate::error::{Result, TextError};
use crate::posting_list::Posting;
use crate::tokenizer::{DefaultTokenizer, Tokenizer};
use crate::traits::{Index, Maintain};
use crate::types::{DocId, TextHit};
use std::collections::BTreeMap;
use std::sync::Arc;

/// Statistiken über den Kompaktierungsvorgang.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CompactStats {
    /// Anzahl physisch bereinigter Tombstones.
    pub purged_docs: usize,
    /// Anzahl verbliebener aktiver Dokumente.
    pub active_docs: usize,
}

/// Autarker In-Memory Volltextindex (Klasse A).
///
/// Bietet die standardisierte Index-Schnittstelle gemäß SPEC §4.3 (`insert`, `remove`, `search`)
/// sowie neutrale Wartung (`compact`, `vacuum`) gänzlich ohne Fremd- oder Storage-Abhängigkeiten.
pub struct MemoryTextIndex {
    tokenizer: Arc<dyn Tokenizer>,
    scorer: BM25,
    postings: BTreeMap<String, Vec<Posting>>,
    doc_lengths: BTreeMap<DocId, u32>,
    tombstones: BTreeMap<DocId, bool>,
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
            postings: BTreeMap::new(),
            doc_lengths: BTreeMap::new(),
            tombstones: BTreeMap::new(),
        }
    }

    /// Konfiguriert einen spezifischen Tokenizer.
    pub fn with_tokenizer(mut self, tokenizer: Arc<dyn Tokenizer>) -> Self {
        self.tokenizer = tokenizer;
        self
    }

    /// Liefert die Anzahl aktiver (nicht als gelöscht markierter) Dokumente.
    pub fn len(&self) -> usize {
        self.doc_lengths
            .keys()
            .filter(|id| !self.tombstones.contains_key(id))
            .count()
    }

    /// Gibt an, ob der Index leer ist.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Rohsicht-Inspektion unter dem Feature `inspect`.
    #[cfg(feature = "inspect")]
    pub fn inspect_postings(&self) -> &BTreeMap<String, Vec<Posting>> {
        &self.postings
    }

    /// Rohsicht-Inspektion der Dokumentlängen unter dem Feature `inspect`.
    #[cfg(feature = "inspect")]
    pub fn inspect_doc_lengths(&self) -> &BTreeMap<DocId, u32> {
        &self.doc_lengths
    }
}

impl Index<DocId, str> for MemoryTextIndex {
    type Hit = TextHit;

    fn insert(&mut self, id: DocId, payload: &[u8]) -> Result<()> {
        let text = std::str::from_utf8(payload)
            .map_err(|e| TextError::InvalidInput(format!("UTF-8 Fehler: {e}")))?;

        // Falls das Dokument vorher im Tombstone-Status war, aufheben
        self.tombstones.remove(&id);

        let tokens = self.tokenizer.tokenize(text);
        let doc_len = tokens.len() as u32;

        let mut tfs: BTreeMap<String, u32> = BTreeMap::new();
        for t in tokens {
            *tfs.entry(t).or_insert(0) += 1;
        }

        for (term, tf) in tfs {
            let list = self.postings.entry(term).or_default();
            // Existierenden Eintrag für selbe DocId ggf. aktualisieren
            if let Some(pos) = list.iter().position(|p| p.doc_id == id.inner()) {
                list[pos] = Posting {
                    doc_id: id.inner(),
                    tf,
                    doc_len,
                };
            } else {
                list.push(Posting {
                    doc_id: id.inner(),
                    tf,
                    doc_len,
                });
                list.sort_by_key(|p| p.doc_id);
            }
        }

        self.doc_lengths.insert(id, doc_len);
        Ok(())
    }

    fn remove(&mut self, id: &DocId) -> Result<bool> {
        if self.doc_lengths.contains_key(id) && !self.tombstones.contains_key(id) {
            self.tombstones.insert(*id, true);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn search(&self, query: &str, limit: usize) -> Result<Vec<Self::Hit>> {
        if query.trim().is_empty() || limit == 0 {
            return Ok(Vec::new());
        }

        let query_tokens = self.tokenizer.tokenize(query);
        if query_tokens.is_empty() {
            return Ok(Vec::new());
        }

        let total_docs = self.len();
        if total_docs == 0 {
            return Ok(Vec::new());
        }

        let total_len: u64 = self
            .doc_lengths
            .iter()
            .filter(|(id, _)| !self.tombstones.contains_key(id))
            .map(|(_, len)| *len as u64)
            .sum();
        let avg_doc_len = if total_docs > 0 {
            (total_len as f32) / (total_docs as f32)
        } else {
            1.0
        };

        let mut scores: BTreeMap<DocId, f32> = BTreeMap::new();

        for term in query_tokens {
            if let Some(posting_list) = self.postings.get(&term) {
                // Berechne df (nur über aktive Dokumente)
                let df = posting_list
                    .iter()
                    .filter(|p| !self.tombstones.contains_key(&DocId::new(p.doc_id)))
                    .count();

                for p in posting_list {
                    let doc_id = DocId::new(p.doc_id);
                    if self.tombstones.contains_key(&doc_id) {
                        continue;
                    }
                    let score = self.scorer.score_term(
                        p.tf,
                        p.doc_len,
                        avg_doc_len,
                        df as u32,
                        total_docs as u32,
                    );
                    *scores.entry(doc_id).or_insert(0.0) += score;
                }
            }
        }

        let mut hits: Vec<TextHit> = scores
            .into_iter()
            .map(|(doc_id, score)| TextHit { doc_id, score })
            .collect();

        // Sortierung: Score absteigend, bei Gleichstand DocId aufsteigend (Determinismus)
        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.doc_id.cmp(&b.doc_id))
        });

        if hits.len() > limit {
            hits.truncate(limit);
        }

        Ok(hits)
    }
}

impl Maintain for MemoryTextIndex {
    type Stats = CompactStats;

    fn compact(&mut self) -> Result<Self::Stats> {
        let purged_count = self.tombstones.len();
        if purged_count == 0 {
            return Ok(CompactStats {
                purged_docs: 0,
                active_docs: self.doc_lengths.len(),
            });
        }

        // Entferne gelöschte Dokumente aus doc_lengths
        for dead_id in self.tombstones.keys() {
            self.doc_lengths.remove(dead_id);
        }

        // Bereinige alle Postinglisten physisch
        for list in self.postings.values_mut() {
            list.retain(|p| !self.tombstones.contains_key(&DocId::new(p.doc_id)));
        }

        // Leere Terme löschen
        self.postings.retain(|_, list| !list.is_empty());

        self.tombstones.clear();

        Ok(CompactStats {
            purged_docs: purged_count,
            active_docs: self.doc_lengths.len(),
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
        let doc1 = DocId::new(1);
        let doc2 = DocId::new(2);

        index.insert(doc1, b"apple orange banana").unwrap();
        index.insert(doc2, b"banana melon kiwi").unwrap();

        assert_eq!(index.len(), 2);

        let results = index.search("banana", 10).unwrap();
        assert_eq!(results.len(), 2);

        // Entfernen von doc1
        assert!(index.remove(&doc1).unwrap());
        assert_eq!(index.len(), 1);

        let results_after = index.search("banana", 10).unwrap();
        assert_eq!(results_after.len(), 1);
        assert_eq!(results_after[0].doc_id, doc2);

        // Compact ausführen
        let stats = index.compact().unwrap();
        assert_eq!(stats.purged_docs, 1);
        assert_eq!(stats.active_docs, 1);

        // Nach compact bleibt Suche konsistent
        let results_compact = index.search("banana", 10).unwrap();
        assert_eq!(results_compact.len(), 1);
        assert_eq!(results_compact[0].doc_id, doc2);
    }
}
