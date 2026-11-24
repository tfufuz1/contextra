//! Bm25Scorer Ports-Adapter (SPEC §6.2 text, B04).
// ANCHOR[contextra-text:COMPAT-003]

use crate::inverted::InvertedIndex;
use contextra_ports::{TextIndex, TextIndexStats};
use contextra_types::{DocId, Result, ScoredDocument, TxId};
#[allow(unused_imports)]
use contextra_ports::TextIndex as _;

/// `Bm25Scorer` fungiert als primäre Integrationsfassade für den Port [`TextIndex`].
///
/// Alle Operationen (Suche, Snapshot-Isolation,
/// Einfügen, Löschen, Transaktions-Commit/-Rollback) werden
/// direkt an den internen [`InvertedIndex`] delegiert.
///
/// # Generische Parameter
/// - `S`: Implementierung von [`contextra_ports::StorageEngine`], über welche
///   Posting-Listen und Dokument-Längen persistent abgelegt werden.
#[deprecated(
    since = "0.1.0",
    note = "Ports-Anbindung gehört in contextra-erasure-adapters / contextra-ports Adapter"
)]
pub struct Bm25Scorer<S: contextra_ports::StorageEngine> {
    index: InvertedIndex<S>,
}

impl<S: contextra_ports::StorageEngine> Bm25Scorer<S> {
    // ANCHOR[contextra-text:LIB-002]
    /// Erzeugt eine neue `Bm25Scorer`-Instanz über der angegebenen Speicher-Engine.
    ///
    /// # Ablauf
    /// 1. Übernahme der Referenz auf die `StorageEngine`.
    /// 2. Initialisierung des internen [`InvertedIndex`] mit dem Namensraum-Präfix.
    ///
    /// # Parameter
    /// - `storage`: Referenzgezählter Zeiger ([`std::sync::Arc`]) auf den Speicher-Port.
    /// - `namespace`: Präfix zur Mandanten- und Tabellen-Isolation der Schlüssel.
    pub fn new(storage: std::sync::Arc<S>, namespace: &str) -> Self {
        Self {
            index: InvertedIndex::new(storage, namespace),
        }
    }
}

impl<S: contextra_ports::StorageEngine> TextIndex for Bm25Scorer<S> {
    // ANCHOR[contextra-text:LIB-003]
    /// Führt eine Volltextsuche anhand der Query aus und liefert bis zu `k` Treffer.
    ///
    /// # Ablauf
    /// 1. Tokenisierung der Anfrage mit dem Standard-Tokenizer.
    /// 2. Ermittlung der Kandidaten und Bewertung per BM25.
    /// 3. Sortierung nach Relevanzwert (absteigend) und DocId (aufsteigend bei Gleichstand).
    async fn search(&self, query: &str, k: usize) -> Result<Vec<ScoredDocument>> {
        self.index.search(query, k).await
    }

    // ANCHOR[contextra-text:LIB-004]
    /// Führt eine Volltextsuche zu einem bestimmten MVCC-Sequenznummer-Snapshot aus.
    ///
    /// Snapshot-Isolation wird gewahrt, indem nur Einträge berücksichtigt werden,
    /// deren Sequenznummer kleiner oder gleich `seq_no` ist.
    async fn search_at(&self, query: &str, k: usize, seq_no: u64) -> Result<Vec<ScoredDocument>> {
        self.index.search_at(query, k, seq_no).await
    }

    // ANCHOR[contextra-text:LIB-005]
    /// Fügt ein Dokument unter der Transaktions-ID `tx` in den Index ein.
    ///
    /// # Ablauf
    /// 1. Tokenisierung und Ermittlung von Termfrequenzen und Dokumentlänge.
    /// 2. Vorbereitung der Änderungen im Transaktions-Staging-Puffer.
    async fn insert(&self, tx: TxId, id: DocId, text: &str) -> Result<()> {
        self.index.insert(tx, id, text).await
    }

    // ANCHOR[contextra-text:LIB-006]
    /// Markiert ein Dokument unter der Transaktions-ID `tx` als gelöscht (Tombstone).
    async fn delete(&self, tx: TxId, id: DocId) -> Result<()> {
        self.index.delete(tx, id).await
    }

    // ANCHOR[contextra-text:LIB-007]
    /// Schreibt alle für Transaktion `tx` vorgemerkten Änderungen dauerhaft in die Storage-Engine.
    async fn commit(&self, tx: TxId) -> Result<()> {
        self.index.commit(contextra_ids::TxId::new(tx.inner())).await.map_err(Into::into)
    }

    // ANCHOR[contextra-text:LIB-008]
    /// Verwirft alle für die Transaktion `tx` im Speicher gepufferten Änderungen.
    async fn rollback(&self, tx: TxId) -> Result<()> {
        self.index.rollback(contextra_ids::TxId::new(tx.inner())).await.map_err(Into::into)
    }

    // ANCHOR[contextra-text:LIB-009]
    /// Führt einen Rollback aller Transaktionen bis einschließlich `tx_id` aus.
    async fn rollback_to_tx(&self, tx_id: TxId) -> Result<()> {
        self.index.rollback_to_tx(contextra_ids::TxId::new(tx_id.inner())).await.map_err(Into::into)
    }

    // ANCHOR[contextra-text:LIB-010]
    /// Liefert die höchste im Index registrierte Transaktions-ID.
    async fn last_tx_id(&self) -> Result<TxId> {
        self.index.last_tx_id().await.map(|t| TxId::new(t.inner())).map_err(Into::into)
    }

    // ANCHOR[contextra-text:LIB-011]
    /// Liefert die Gesamtzahl aktiver Dokumente im Index.
    async fn len(&self) -> usize {
        self.index.len().await
    }

    // ANCHOR[contextra-text:LIB-012]
    /// Liefert statistische Kennzahlen über den Textindex (z. B. Dokumentenanzahl, Terme).
    async fn stats(&self) -> Result<TextIndexStats> {
        self.index.stats().await
    }
}
