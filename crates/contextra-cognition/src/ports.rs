//! Lokale Port-Definitionen für contextra-cognition (SPEC §6.4, S-63).
//
// FILE-CONTEXT
// ZWECK: Abstrakte Port-Interfaces (DocumentSource, ConsolidationSink, HyperedgeSink, DecayPolicy).
// INVARIANTEN: Zero-Panic; Dyn-Safety; Send + Sync für asynchrone Worker.
// STAND: 2026-11-11

use async_trait::async_trait;
use contextra_types::{DocId, Result, TxId};

// =========================================================================
// SECTION: Commit-Intent Abstraktion
// =========================================================================

// ANCHOR[contextra-cognition:PORT-001]
/// Lokaler Ersatztyp für `CommitIntent`, falls die direkte Engine-Bindung entfällt.
///
/// Hält die für atomare Transaktionsabschlüsse notwendigen Metadaten
/// isoliert von der physischen Speicher-Engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommitIntent {
    /// Eindeutiger Intent-Key oder Bezeichner.
    pub intent_id: String,
    /// Transaktions-ID des Commits.
    pub tx_id: TxId,
    /// Betroffene Dokumente.
    pub target_docs: Vec<DocId>,
}
// !SECTION: Commit-Intent Abstraktion

// =========================================================================
// SECTION: Port-Traits
// =========================================================================

// ANCHOR[contextra-cognition:PORT-002]
/// Abstrakte Quelle für Dokumente zur Konsolidierung.
///
/// Entkoppelt das Laden von Chunks oder Gesprächs-Turns von der konkreten
/// Datenbank- oder Vektordatenbank-Implementierung.
#[async_trait]
pub trait DocumentSource: Send + Sync {
    /// Lädt Turns / Dokumente für die angegebene Kollektion oder den Kontext.
    async fn fetch_turns(&self) -> Result<Vec<(DocId, Vec<f32>)>>;
}

// ANCHOR[contextra-cognition:PORT-003]
/// Abstrakte Senke für Konsolidierungsergebnisse.
///
/// Garantiert, dass konsolidierte Chunks atomar in das Zielsubsystem
/// eingepflegt werden können.
#[async_trait]
pub trait ConsolidationSink: Send + Sync {
    /// Wendet einen Commit-Intent atomar an.
    async fn apply_intent(&self, intent: CommitIntent) -> Result<()>;
}

// ANCHOR[contextra-cognition:PORT-004]
/// Senke für synthetische Hyperkanten und Kaskaden-Invalidierungen.
///
/// Wird in der semantischen Aggregationsphase (LeanRAG Stage 3) verwendet,
/// um verdichtete Superkanten im Wissensgraphen anzulegen.
pub trait HyperedgeSink: Send + Sync {
    /// Invalidiert eine bestehende Hyperkante anhand ihrer ID.
    fn invalidate_edge(&mut self, edge_id: u64);
    /// Schreibt eine neu synthetisierte Hyperkante.
    fn write_super_edge(&mut self, edge_data: Vec<u8>);
    /// Veröffentlicht alle gesammelten Änderungen atomar.
    fn commit(&mut self) -> Result<()>;
}

// ANCHOR[contextra-cognition:PORT-005]
/// Richtlinie zur adaptiven Abwertung zeitlicher Relevanz (Decay Control).
///
/// Erlaubt anpassbare Abklingfunktionen für Langzeitgedächtnisse.
pub trait DecayPolicy: Send + Sync {
    /// Berechnet den aktuellen Dämpfungsfaktor für ein Dokument.
    fn compute_decay(&self, elapsed_secs: u64, access_count: u64) -> f32;
}
// !SECTION: Port-Traits
