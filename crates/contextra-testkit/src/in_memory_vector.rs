//! # In-Memory-Vektorindex für Tests (`in_memory_vector`)
//!
//! Dieses Modul stellt mit [`InMemoryVectorIndex`] eine einfache, deterministische
//! Referenzimplementierung des [`contextra_ports::VectorIndex`]-Ports bereit.
//!
//! ## Ring & Architektur-Einordnung
//! - **Ring:** Werkzeug / Tooling
//! - **Port:** Implementiert [`contextra_ports::VectorIndex`].
//! - **Distanzmetrik:** Verwendet euklidischen Abstand zur Ähnlichkeitsberechnung:
//!   `score = 1.0 / (1.0 + dist)`.
//! - **Transaktionales Staging:** Unterstützt wie `InMemoryStorageEngine` isoliertes Staging
//!   via `TxId` und verzögertes Commit/Rollback.
//!
//! ## Validierung & Fehlerbehandlung
//! - Prüft Vektordimensionen strikt gegen `dim` (`ContextraError::invalid_input`).
//! - Prüft auf Nicht-Endlichkeit (`NaN`, `Infinity`) in Eingabevektoren und Queries.
//! - Verhindert unbestimmte Fließkomma-Sortierungen durch `then_with` auf `DocId`.

use contextra_ports::{VectorIndex, VectorIndexStats};
use contextra_types::{ContextraError, DocId, Result, ScoredDocument, TxId};
use parking_lot::RwLock;
use std::collections::BTreeMap;
use std::sync::Arc;

// ANCHOR[contextra-testkit:STAGED-VECTOR-OP-ENUM-001]
/// Staging-Operation für Vektor-Modifikationen innerhalb einer offenen Transaktion.
#[derive(Debug, Clone)]
enum StagedVectorOp {
    /// Einfügen oder Aktualisieren eines Embeddings.
    Insert(Vec<f32>),
    /// Löschen eines Vektors anhand seiner Dokumenten-ID.
    Delete,
}

/// Dokument-ID -> Vektor (f32-Array).
type VectorMap = BTreeMap<DocId, Vec<f32>>;
/// Transaktions-ID -> (Dokument-ID -> StagedVectorOp).
type StagedVectorTxMap = BTreeMap<TxId, BTreeMap<DocId, StagedVectorOp>>;

// ANCHOR[contextra-testkit:IN-MEM-VECTOR-STRUCT-001]
/// Thread-sichere Referenzimplementierung von [`VectorIndex`] für Tests.
///
/// **Zweck:** Ermöglicht semantische Vektorsuchen in Unit- und Integrationstests,
/// ohne externe Vektordatenbanken oder komplexe ANN-Indizes (wie HNSW oder DiskANN)
/// instanziieren zu müssen.
///
/// **Invarianten:**
/// - Alle gespeicherten Vektoren haben exakt die Dimension `dim`.
/// - Vektoren enthalten keine NaN- oder Infinity-Werte.
#[derive(Debug, Clone)]
pub struct InMemoryVectorIndex {
    /// Erwartete Dimension der Vektoren.
    dim: usize,
    /// Bestätigter Vektordatenbestand.
    data: Arc<RwLock<VectorMap>>,
    /// Unbestätigte Staging-Änderungen nach Transaktionen.
    staged: Arc<RwLock<StagedVectorTxMap>>,
    /// ID der letzten bestätigten Transaktion.
    last_tx: Arc<RwLock<TxId>>,
}

impl InMemoryVectorIndex {
    // ANCHOR[contextra-testkit:IN-MEM-VECTOR-NEW-001]
    /// Erstellt einen neuen `InMemoryVectorIndex` mit der erwarteten Vektordimension `dim`.
    pub fn new(dim: usize) -> Self {
        Self {
            dim,
            data: Arc::new(RwLock::new(BTreeMap::new())),
            staged: Arc::new(RwLock::new(BTreeMap::new())),
            last_tx: Arc::new(RwLock::new(TxId::new(0))),
        }
    }
}

// ANCHOR[contextra-testkit:IN-MEM-VECTOR-TRAIT-IMPL-001]
impl VectorIndex for InMemoryVectorIndex {
    /// Stellt das Einfügen eines Vektors für Dokument `id` im Transaktionskontext bereit.
    ///
    /// **Validierung:**
    /// - Liefert Fehler bei falscher Dimension.
    /// - Liefert Fehler, falls Komponenten nicht endlich sind (`NaN`/`Infinity`).
    async fn insert(&self, tx: TxId, id: DocId, embedding: &[f32]) -> Result<()> {
        if embedding.len() != self.dim {
            return Err(ContextraError::invalid_input(format!(
                "vector dimension mismatch: expected {}, got {}",
                self.dim,
                embedding.len()
            )));
        }
        for &val in embedding {
            if !val.is_finite() {
                return Err(ContextraError::invalid_input(
                    "vector embedding contains non-finite value (NaN/Infinity)",
                ));
            }
        }

        let mut staged = self.staged.write();
        staged
            .entry(tx)
            .or_default()
            .insert(id, StagedVectorOp::Insert(embedding.to_vec()));
        Ok(())
    }

    /// Stellt das Löschen des Vektors für Dokument `id` im Transaktionskontext bereit.
    async fn delete(&self, tx: TxId, id: DocId) -> Result<()> {
        let mut staged = self.staged.write();
        staged
            .entry(tx)
            .or_default()
            .insert(id, StagedVectorOp::Delete);
        Ok(())
    }

    /// Bestätigt alle gestageten Vektor-Operationen der Transaktion `tx`.
    async fn commit(&self, tx: TxId) -> Result<()> {
        let ops = {
            let mut staged = self.staged.write();
            staged.remove(&tx)
        };

        if let Some(ops) = ops {
            let mut data = self.data.write();
            for (id, op) in ops {
                match op {
                    StagedVectorOp::Insert(v) => {
                        data.insert(id, v);
                    }
                    StagedVectorOp::Delete => {
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
        let mut staged = self.staged.write();
        staged.remove(&tx);
        Ok(())
    }

    /// Setzt alle offenen Staging-Puffer zurück.
    async fn rollback_to_tx(&self, _tx_id: TxId) -> Result<()> {
        self.staged.write().clear();
        Ok(())
    }

    /// Liefert die Transaktions-ID des letzten erfolgreichen Commits.
    async fn last_tx_id(&self) -> Result<TxId> {
        Ok(*self.last_tx.read())
    }

    /// Liefert alle aktuell bestätigten Dokument-IDs im Vektorindex.
    async fn all_doc_ids(&self) -> Result<Vec<DocId>> {
        let data = self.data.read();
        Ok(data.keys().copied().collect())
    }

    // ANCHOR[contextra-testkit:IN-MEM-VECTOR-SEARCH-001]
    /// Führt einen linearen Exact-kNN-Scan (k-Nearest Neighbors) durch.
    ///
    /// **Ablauf:**
    /// 1. Dimension und Endlichkeit des Query-Vektors validieren.
    /// 2. Euklidischen Abstand zu jedem bestätigten Vektor berechnen (`dist = sqrt(sum((q-v)^2))`).
    /// 3. Normalisierten Ähnlichkeitsscore ermitteln: `score = 1.0 / (1.0 + dist)`.
    /// 4. Nach absteigendem Score sortieren (Tie-Breaking über aufsteigende `DocId`).
    /// 5. Auf `k` beste Treffer abschneiden (`truncate`).
    async fn search(&self, query: &[f32], k: usize) -> Result<Vec<ScoredDocument>> {
        if query.len() != self.dim {
            return Err(ContextraError::invalid_input(format!(
                "query dimension mismatch: expected {}, got {}",
                self.dim,
                query.len()
            )));
        }
        for &val in query {
            if !val.is_finite() {
                return Err(ContextraError::invalid_input(
                    "query vector contains non-finite value (NaN/Infinity)",
                ));
            }
        }

        let data = self.data.read();
        let mut scored: Vec<ScoredDocument> = data
            .iter()
            .map(|(&doc_id, vec)| {
                let dist_sq: f32 = query
                    .iter()
                    .zip(vec.iter())
                    .map(|(q, v)| (q - v) * (q - v))
                    .sum();
                let dist = dist_sq.sqrt();
                let score = 1.0 / (1.0 + dist);
                ScoredDocument::new(doc_id, score)
            })
            .collect();

        scored.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.doc_id.cmp(&b.doc_id))
        });

        scored.truncate(k);
        Ok(scored)
    }

    /// Gibt die Gesamtzahl der gespeicherten Vektoren zurück.
    async fn len(&self) -> usize {
        self.data.read().len()
    }

    /// Ermittelt statistische Kennzahlen zum In-Memory-Vektorindex.
    async fn stats(&self) -> Result<VectorIndexStats> {
        let len = self.len().await;
        Ok(VectorIndexStats {
            num_vectors: len,
            memory_usage_bytes: len * self.dim * 4,
            num_layers: 1,
            deleted_ratio: 0.0,
            rebuild_count: 0,
        })
    }
}
