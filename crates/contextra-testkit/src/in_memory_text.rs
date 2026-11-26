//! # In-Memory-Volltextindex für Tests (`in_memory_text`)
//!
//! Dieses Modul stellt mit [`InMemoryTextIndex`] eine deterministische
//! Referenzimplementierung des [`contextra_ports::TextIndex`]-Ports bereit.
//!
//! ## Ring & Architektur-Einordnung
//! - **Ring:** Werkzeug / Tooling
//! - **Port:** Implementiert [`contextra_ports::TextIndex`].
//! - **Textnormalisierung:** Einfache Kleinschreibung und Umlautersetzung (`ä->ae`, `ö->oe`, `ü->ue`, `ß->ss`).
//! - **Suchalgorithmus:** Token-basierte Überlappungssuche mit normalisierter Trefferquote als Relevanz-Score:
//!   `score = treffer / anzahl_suchbegriffe`.
//! - **Transaktionales Staging:** Isolierte Haltung unbestätigter Textmutationen via `TxId`.

use contextra_ports::{TextIndex, TextIndexStats};
use contextra_types::{DocId, Result, ScoredDocument, TxId};
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::sync::Arc;

// ANCHOR[contextra-testkit:STAGED-TEXT-OP-ENUM-001]
/// Unbestätigte Text-Operation innerhalb einer Transaktion.
#[derive(Debug, Clone)]
enum StagedTextOp {
    /// Einfügen oder Aktualisieren eines Volltextdokuments.
    Insert(String),
    /// Löschen eines Volltextdokuments.
    Delete,
}

/// Dokumentenbestand: Dokument-ID -> Textinhalt.
type TextMap = BTreeMap<DocId, String>;
/// Staging-Puffer: Transaktions-ID -> (Dokument-ID -> StagedTextOp).
type StagedTextTxMap = BTreeMap<TxId, BTreeMap<DocId, StagedTextOp>>;

// ANCHOR[contextra-testkit:IN-MEM-TEXT-STRUCT-001]
/// Thread-sichere In-Memory-Referenzimplementierung von [`TextIndex`] für Tests.
///
/// **Zweck:** Ermöglicht Volltext- und Stichwortsuchen in Tests ohne externe
/// Such-Engines (wie Tantivy oder Lucene).
#[derive(Debug, Clone)]
pub struct InMemoryTextIndex {
    /// Bestätigter Dokumentendatenbestand.
    data: Arc<RwLock<TextMap>>,
    /// Unbestätigte Transaktionsänderungen.
    staged: Arc<RwLock<StagedTextTxMap>>,
    /// ID der letzten bestätigten Transaktion.
    last_tx: Arc<RwLock<TxId>>,
}

impl Default for InMemoryTextIndex {
    fn default() -> Self {
        Self {
            data: Arc::new(RwLock::new(BTreeMap::new())),
            staged: Arc::new(RwLock::new(BTreeMap::new())),
            last_tx: Arc::new(RwLock::new(TxId::new(0))),
        }
    }
}

impl InMemoryTextIndex {
    // ANCHOR[contextra-testkit:IN-MEM-TEXT-NEW-001]
    /// Erstellt einen neuen, leeren `InMemoryTextIndex`.
    pub fn new() -> Self {
        Self::default()
    }

    // ANCHOR[contextra-testkit:IN-MEM-TEXT-NORM-001]
    /// Normalisiert Zeichenketten: Wandelt in Kleinbuchstaben um und ersetzt deutsche Umlaute.
    fn normalize_term(term: &str) -> String {
        term.to_lowercase()
            .replace('ä', "ae")
            .replace('ö', "oe")
            .replace('ü', "ue")
            .replace('ß', "ss")
    }
}

// ANCHOR[contextra-testkit:IN-MEM-TEXT-TRAIT-IMPL-001]
impl TextIndex for InMemoryTextIndex {
    /// Stellt das Einfügen eines Textdokuments für Transaktion `tx` bereit.
    async fn insert(&self, tx: TxId, id: DocId, text: &str) -> Result<()> {
        let mut staged = self.staged.write();
        staged
            .entry(tx)
            .or_default()
            .insert(id, StagedTextOp::Insert(text.to_string()));
        Ok(())
    }

    /// Stellt das Löschen eines Textdokuments für Transaktion `tx` bereit.
    async fn delete(&self, tx: TxId, id: DocId) -> Result<()> {
        let mut staged = self.staged.write();
        staged
            .entry(tx)
            .or_default()
            .insert(id, StagedTextOp::Delete);
        Ok(())
    }

    /// Bestätigt alle Textmutationen der Transaktion `tx`.
    async fn commit(&self, tx: TxId) -> Result<()> {
        let ops = {
            let mut staged = self.staged.write();
            staged.remove(&tx)
        };

        if let Some(ops) = ops {
            let mut data = self.data.write();
            for (id, op) in ops {
                match op {
                    StagedTextOp::Insert(t) => {
                        data.insert(id, t);
                    }
                    StagedTextOp::Delete => {
                        data.remove(&id);
                    }
                }
            }
            *self.last_tx.write() = tx;
        }
        Ok(())
    }

    /// Verwirft alle gestageten Änderungen der Transaktion `tx`.
    async fn rollback(&self, tx: TxId) -> Result<()> {
        self.staged.write().remove(&tx);
        Ok(())
    }

    /// Setzt alle offenen Staging-Puffer zurück.
    async fn rollback_to_tx(&self, _tx_id: TxId) -> Result<()> {
        self.staged.write().clear();
        Ok(())
    }

    /// Liefert die Transaktions-ID des letzten Commits.
    async fn last_tx_id(&self) -> Result<TxId> {
        Ok(*self.last_tx.read())
    }

    // ANCHOR[contextra-testkit:IN-MEM-TEXT-SEARCH-001]
    /// Durchsucht den Index nach Dokumenten, die die Token der Suchanfrage enthalten.
    ///
    /// **Ablauf:**
    /// 1. Suchanfrage normalisieren und trimmen. Bei leerer Anfrage leere Trefferliste zurückgeben.
    /// 2. Suchbegriffe an Whitespace splitten.
    /// 3. Für jedes bestätigte Dokument prüfen, wie viele der Suchbegriffe im normalisierten Text vorkommen.
    /// 4. Score berechnen: `match_count / query_tokens.len()`.
    /// 5. Treffer absteigend nach Score sortieren (Tie-Breaking über `DocId`).
    /// 6. Auf `k` Einträge begrenzen.
    async fn search(&self, query: &str, k: usize) -> Result<Vec<ScoredDocument>> {
        let norm_query = Self::normalize_term(query.trim());
        if norm_query.is_empty() {
            return Ok(Vec::new());
        }

        let query_tokens: Vec<&str> = norm_query.split_whitespace().collect();
        let data = self.data.read();

        let mut matches = Vec::new();
        for (&doc_id, text) in data.iter() {
            let norm_doc = Self::normalize_term(text);
            let mut match_count = 0u32;

            for &q_token in &query_tokens {
                if norm_doc.contains(q_token) {
                    match_count += 1;
                }
            }

            if match_count > 0 {
                let score = match_count as f32 / query_tokens.len().max(1) as f32;
                matches.push(ScoredDocument::new(doc_id, score));
            }
        }

        matches.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.doc_id.cmp(&b.doc_id))
        });

        matches.truncate(k);
        Ok(matches)
    }

    /// Liefert die Anzahl der aktuell bestätigten Textdokumente.
    async fn len(&self) -> usize {
        self.data.read().len()
    }

    /// Liefert statistische Kennzahlen zum In-Memory-Textindex.
    async fn stats(&self) -> Result<TextIndexStats> {
        let data = self.data.read();
        let num_documents = data.len();
        let num_tokens: usize = data.values().map(|t| t.split_whitespace().count()).sum();
        let memory_usage_bytes: usize = data.values().map(|t| t.len()).sum();

        Ok(TextIndexStats {
            num_documents,
            num_tokens,
            memory_usage_bytes,
        })
    }
}
