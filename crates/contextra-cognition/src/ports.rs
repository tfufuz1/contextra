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
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum CommitIntent {
    /// Transaction is in the "Prepared" state. Stored DocIds assist recovery.
    Pending {
        doc_ids: std::sync::Arc<Vec<DocId>>,
        #[serde(default)]
        has_text: bool,
        #[serde(default)]
        has_graph: bool,
        #[serde(default)]
        stages_completed: u8,
    },
    /// Transaction is committed across all indices.
    Committed,
    /// Transaction was aborted and compensated.
    Aborted,
    /// Memory consolidation intent (Sleep-Cycle OCC).
    Consolidation {
        source_docs: Vec<(DocId, TxId)>,
        target_id: DocId,
        base_tx: TxId,
    },
    /// Transaction failed during forward recovery or commit.
    Failed { reason: String },
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

// ANCHOR[contextra-cognition:PORT-006]
/// Abstrakte Port-Schnittstelle für Collection-Operationen (SPEC S-63).
///
/// Entkoppelt contextra-cognition von konkreten Storage- und Index-Engines.
#[async_trait]
pub trait CollectionOps: Send + Sync {
    /// Gibt den Namen der Collection zurück.
    fn name(&self) -> &str;

    /// Erzeugt einen namensraumisolierten Schlüssel für den physischen Speicher.
    fn namespaced_key(&self, raw: &[u8], key_type: u8) -> Vec<u8>;

    /// Gibt das Schlüsselpräfix für reguläre Benutzerdaten zurück.
    fn user_key_prefix(&self) -> Vec<u8>;

    /// Prüft, ob aktuell eine Konsolidierung läuft.
    fn consolidation_in_progress(&self) -> bool;

    /// Ermittelt die Vektordimension der Collection.
    fn dimension(&self) -> usize;

    /// Allokiert eine monoton steigende Transaktions-ID.
    fn allocate_tx(&self) -> Result<TxId>;

    /// Ruft ein Dokument anhand seines Bezeichners ab.
    async fn get(&self, id: &str) -> Result<Option<crate::collection_types::StoredDocument>>;

    /// Löscht ein Dokument anhand seines Bezeichners.
    async fn delete(&self, id: &str) -> Result<()>;

    /// Schreibt einen Schlüssel-Wert-Eintrag.
    async fn put_kv(&self, key: &str, value: &serde_json::Value) -> Result<()>;

    /// Führt die zeitliche Abwertung und Bereinigung verfallener Chunks durch.
    async fn evict_decayed_chunks(
        &self,
        controller: &crate::decay::AdaptiveDecayController,
        limit: usize,
    ) -> Result<usize>;
}
// !SECTION: Port-Traits
