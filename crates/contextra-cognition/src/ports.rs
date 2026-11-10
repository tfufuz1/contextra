//! Lokale Port-Definitionen für contextra-cognition (SPEC-v2 §6.3, S-63).

use async_trait::async_trait;
use contextra_types::{DocId, Result, TxId};

/// Lokaler Ersatztyp für CommitIntent, falls die Engine-Bindung entfällt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitIntent {
    /// Eindeutiger Intent-Key oder Bezeichner.
    pub intent_id: String,
    /// Transaktions-ID des Commits.
    pub tx_id: TxId,
    /// Betroffene Dokumente.
    pub target_docs: Vec<DocId>,
}

/// Abstrakte Quelle für Dokumente zur Konsolidierung.
#[async_trait]
pub trait DocumentSource: Send + Sync {
    /// Lädt Turns / Dokumente für die angegebene Kollektion oder den Kontext.
    async fn fetch_turns(&self) -> Result<Vec<(DocId, Vec<f32>)>>;
}

/// Abstrakte Senke für Konsolidierungsergebnisse.
#[async_trait]
pub trait ConsolidationSink: Send + Sync {
    /// Wendet einen Commit-Intent atomar an.
    async fn apply_intent(&self, intent: CommitIntent) -> Result<()>;
}

/// Senke für synthetische Hyperkanten und Invalidierungen.
pub trait HyperedgeSink: Send + Sync {
    /// Invalidiert eine bestehende Hyperkante anhand ihrer ID.
    fn invalidate_edge(&mut self, edge_id: u64);
    /// Schreibt eine neu synthetisierte Hyperkante.
    fn write_super_edge(&mut self, edge_data: Vec<u8>);
    /// Veröffentlicht alle gesammelten Änderungen atomar.
    fn commit(&mut self) -> Result<()>;
}

/// Richtlinie zur adaptiven Abwertung zeitlicher Relevanz.
pub trait DecayPolicy: Send + Sync {
    /// Berechnet den aktuellen Dämpfungsfaktor für ein Dokument.
    fn compute_decay(&self, elapsed_secs: u64, access_count: u64) -> f32;
}
