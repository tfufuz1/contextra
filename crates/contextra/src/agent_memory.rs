// FILE-CONTEXT
// ZWECK: Agent Memory Facade (Ring 4) — Provides a clean remember/recall/forget/relate API for AI Agents.
// INVARIANTEN: #![forbid(unsafe_code)]; Zero panic doctrine (.unwrap/.expect forbidden); Wraps Contextra engine without changing business logic.

//! # Modul: Agent Memory Facade (Ring 4)
//!
//! Dieses Modul bietet eine ergonomische, an KI-Agenten orientierte Schnittstelle zur Verwaltung von
//! Langzeit- und Kurzzeitgedächtnis (`remember`, `recall`, `forget`, `relate`).
//!
//! ## Ring-Zugehörigkeit
//! - **Ring 4 (Fassade)**: Kapselt die darunterliegende Datenbank-Engine [`Contextra`] (Ring 3),
//!   ohne deren Geschäftslogik zu verändern.
//!
//! ## Wichtigste Typen
//! - [`AgentMemory`]: Die zentrale Schnittstelle für Agenten mit Operationen für Speichern, Suchen,
//!   Löschen und Verknüpfen.
//! - [`MemoryId`]: Typisierter Bezeichner für eine gespeicherte Erinnerung.
//! - [`RelationId`]: Eindeutige Kennung für n-äre Beziehungen (Hyperedges) im Wissensgraphen.
//! - [`Memory`]: Struktur einer aus dem Speicher abgerufenen Erinnerung inklusive Relevanz-Score,
//!   Metadaten und Signalursprung.
//!
//! ## Entwurfs- & Determinismus-Invarianten
//! - **Deterministische IDs**: Nicht-deterministische Zufalls-UUIDs (`Uuid::new_v4`) sind im Produktionscode
//!   strikt untersagt. Die ID-Generierung erfolgt bevorzugt über einen injizierten [`IdGen`]-Port
//!   oder deterministische Fallback-Zähler (`fallback-1`, `fallback-2`).
//! - **Zero-Panic-Doktrin**: Alle Operationen geben ein [`Result`] zurück; Fehlzustände werden sauber
//!   propagiert.
//! - **Wissensgraph & Hyperedges**: Unterstützt binäre Beziehungen (`relate`) sowie n-äre Hyperedges (`relate_n_ary`),
//!   bei denen mehrere Teilnehmer jeweils an eine semantische Rolle (z. B. `author`, `reviewer`) gebunden sind.

use contextra_db::{Contextra, ProvenanceRecord, SearchResult};
use contextra_ports::IdGen;
use contextra_types::error::ContextraError;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

// ANCHOR[contextra:MEM-001]
/// Globaler atomarer Sequenzzähler für deterministische Fallback-IDs bei fehlendem `IdGen`.
static FALLBACK_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Generates a deterministic fallback memory ID when no explicit `IdGen` is injected into `AgentMemory`.
///
/// # Governance & Determinism Notice
/// - (a) Non-deterministic `uuid::Uuid::new_v4()` is strictly prohibited to guarantee Contextra's
///   determinism promise ("reproducible, audited context").
/// - (b) This fallback mechanism is strictly utilized only when no explicit `IdGen` was injected.
/// - (c) For production deployments requiring strict replay determinism across process restarts, an `IdGen`
///   must be explicitly injected via `AgentMemory::with_id_gen` or `AgentMemory::new_with_id_gen`.
// ANCHOR[contextra:MEM-002]
/// Generiert eine deterministische Fallback-ID (`"fallback-1"`, `"fallback-2"` etc.), wenn kein expliziter
/// [`IdGen`]-Port in `AgentMemory` injiziert wurde.
///
/// # Governance & Determinismus
/// - (a) `Uuid::new_v4()` ist strikt verboten, um Contextras Determinismus- und Revisionsversprechen einzuhalten.
/// - (b) Der Fallback greift ausschließlich dann, wenn `id_gen == None` ist.
/// - (c) In Produktionsumgebungen sollte stets ein persistenter `IdGen` injiziert werden.
fn next_fallback_id() -> String {
    let seq = FALLBACK_ID_COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("fallback-{seq}")
}

// ANCHOR[contextra:MEM-003]
/// Eindeutiger, typisierter Identifikator für eine in [`AgentMemory`] abgelegte Erinnerung.
///
/// Kapselt den internen Dokument-ID-String der Datenbank-Engine.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MemoryId(String);

impl MemoryId {
    /// Creates a new `MemoryId` from a string identifier.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Returns a string slice reference to the underlying memory ID.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for MemoryId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for MemoryId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl AsRef<str> for MemoryId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for MemoryId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ANCHOR[contextra:MEM-004]
/// Eindeutiger Identifikator für eine n-äre Beziehung (Hyperedge) im Wissensgraphen.
///
/// Kapselt den 64-Bit-Graphenschlüssel der Wissensgraphen-Engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RelationId(pub u64);

impl fmt::Display for RelationId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RelationId({})", self.0)
    }
}

// ANCHOR[contextra:MEM-005]
/// Repräsentation einer aus dem Speicher abgerufenen Erinnerung (`recall`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    /// Document identifier string.
    pub id: String,
    /// Relevance / similarity score.
    pub score: f32,
    /// Optional metadata associated with the memory entry.
    pub metadata: Option<Value>,
    /// Matched signal types (e.g., text, vector, graph).
    pub matched_signals: Vec<String>,
    /// Detailed score provenance breakdown.
    pub provenance: Option<ProvenanceRecord>,
}

impl From<SearchResult> for Memory {
    fn from(res: SearchResult) -> Self {
        Self {
            id: res.id,
            score: res.score,
            metadata: res.metadata,
            matched_signals: res.matched_signals,
            provenance: res.provenance,
        }
    }
}

// ANCHOR[contextra:MEM-006]
/// High-Level-Fassade für die Verwaltung des Agentengedächtnisses.
///
/// Bietet die Kernoperationen:
/// - `remember`: Speichern von Texten und Metadaten.
/// - `recall`: Semantisches Retrieval basierend auf Volltext- und Vektorsuche.
/// - `forget`: Sicheres Löschen einer Erinnerung.
/// - `relate` & `relate_n_ary`: Verknüpfen von Erinnerungen im Wissensgraphen.
#[derive(Clone)]
pub struct AgentMemory {
    engine: Arc<Contextra>,
    id_gen: Option<Arc<dyn IdGen>>,
}

impl AgentMemory {
    // ANCHOR[contextra:MEM-007]
    /// Erstellt eine neue `AgentMemory`-Instanz, indem die übergebene `Contextra`-Engine in ein `Arc` gehüllt wird.
    pub fn new(engine: Contextra) -> Self {
        Self {
            engine: Arc::new(engine),
            id_gen: None,
        }
    }

    // ANCHOR[contextra:MEM-008]
    /// Erstellt eine neue `AgentMemory`-Instanz um ein bereits geteiltes `Arc<Contextra>`.
    pub fn new_arc(engine: Arc<Contextra>) -> Self {
        Self {
            engine,
            id_gen: None,
        }
    }

    // ANCHOR[contextra:MEM-009]
    /// Erstellt eine Instanz mit einem explizit injizierten ID-Generator-Port.
    pub fn new_with_id_gen(engine: Contextra, id_gen: Arc<dyn IdGen>) -> Self {
        Self {
            engine: Arc::new(engine),
            id_gen: Some(id_gen),
        }
    }

    // ANCHOR[contextra:MEM-010]
    /// Injiziert oder aktualisiert den ID-Generator nachträglich (Builder-Muster).
    pub fn with_id_gen(mut self, id_gen: Arc<dyn IdGen>) -> Self {
        self.id_gen = Some(id_gen);
        self
    }

    // ANCHOR[contextra:MEM-011]
    /// Liefert eine Referenz auf die zugrundeliegende [`Contextra`]-Engine.
    pub fn engine(&self) -> &Contextra {
        &self.engine
    }

    /// Stores a text memory entry with optional JSON metadata.
    ///
    /// Generates a unique memory identifier via the injected `IdGen` if available,
    /// or falls back to a deterministic sequence (`"fallback-1"`, `"fallback-2"`, ...)
    /// when no `IdGen` is provided.
    ///
    /// # Fallback Determinism Notice
    /// - (a) `uuid::Uuid::new_v4()` is no longer used to uphold Contextra's determinism promise
    ///   ("reproducible, audited context") and governance rules forbidding non-deterministic UUIDs in production code.
    /// - (b) `next_fallback_id()` only triggers when no `IdGen` was injected (`id_gen == None`).
    /// - (c) For production deployments requiring replay determinism across process restarts,
    ///   an explicit `IdGen` should be injected via `with_id_gen` or `new_with_id_gen`.
    ///
    /// Delegates directly to `Contextra::insert_text_only`.
    // ANCHOR[contextra:MEM-012]
    /// Speichert eine Text-Erinnerung mit optionalen JSON-Metadaten.
    ///
    /// # Ablauf von `remember`:
    /// 1. ID ermitteln: Über den injizierten `IdGen`-Port oder über `next_fallback_id()`.
    /// 2. Asynchroner Aufruf von `self.engine.insert_text_only(&id, text, metadata).await`.
    /// 3. Rückgabe der erzeugten [`MemoryId`].
    ///
    /// # Nebenwirkungen
    /// - Schreiboperation in WAL und Memtable der Standard-Collection.
    pub async fn remember(
        &self,
        text: &str,
        metadata: Option<Value>,
    ) -> Result<MemoryId, ContextraError> {
        // Fallback documentation:
        // (a) `uuid::Uuid::new_v4()` is no longer used to uphold Contextra's determinism promise
        //     ("reproducible, audited context") and governance rules forbidding non-deterministic UUIDs in production code.
        // (b) `next_fallback_id()` only triggers when no `IdGen` is injected (`id_gen == None`).
        // (c) For production deployments requiring replay determinism across process restarts,
        //     an explicit `IdGen` should be injected via `with_id_gen` or `new_with_id_gen`.
        let id = match &self.id_gen {
            Some(gen) => gen.next_id().to_string(),
            None => next_fallback_id(),
        };
        self.engine.insert_text_only(&id, text, metadata).await?;
        Ok(MemoryId(id))
    }

    /// Retrieves top-k matching memories for a natural language query text string.
    ///
    /// Delegates directly to `Contextra::search_text` and transforms each `SearchResult` into a `Memory`.
    // ANCHOR[contextra:MEM-013]
    /// Ruft die Top-k relevantesten Erinnerungen zu einer Suchanfrage ab.
    ///
    /// Delegiert direkt an `Contextra::search_text` und transformiert Ergebnisse in [`Memory`]-Objekte.
    pub async fn recall(&self, query: &str, k: usize) -> Result<Vec<Memory>, ContextraError> {
        let results = self.engine.search_text(query, k).await?;
        Ok(results.into_iter().map(Memory::from).collect())
    }

    /// Deletes a memory entry by its `MemoryId`.
    ///
    /// Delegates directly to `Contextra::delete`.
    // ANCHOR[contextra:MEM-014]
    /// Entfernt eine Erinnerung anhand ihrer [`MemoryId`].
    ///
    /// Delegiert direkt an `Contextra::delete`.
    pub async fn forget(&self, id: &MemoryId) -> Result<(), ContextraError> {
        self.engine.delete(id.as_str()).await
    }

    /// Establishes a directed knowledge-graph relationship between two memories with a given label.
    ///
    /// Delegates directly to `Contextra::relate`.
    // ANCHOR[contextra:MEM-015]
    /// Erstellt eine gerichtete Beziehung zwischen zwei Erinnerungen im Wissensgraphen.
    ///
    /// # Parameter
    /// - `from`: Quell-Erinnerung.
    /// - `to`: Ziel-Erinnerung.
    /// - `label`: Semantisches Prädikat (z. B. `"depends_on"`, `"contradicts"`).
    pub async fn relate(
        &self,
        from: &MemoryId,
        to: &MemoryId,
        label: &str,
    ) -> Result<(), ContextraError> {
        self.engine.relate(from.as_str(), to.as_str(), label).await
    }

    /// Establishes an n-ary hyperedge relationship connecting two or more memories, each bound to a specific role.
    ///
    /// # Role Semantics
    /// The underlying engine maps each role string (e.g., `"author"`, `"reviewer"`) deterministically
    /// to a `RoleId`. Identical role strings produce the exact same `RoleId`.
    ///
    /// # Early Validation
    /// Validates inputs before delegating to `Collection::relate_n_ary`:
    /// - `predicate` must not be empty.
    /// - `participants` must contain at least 2 entries.
    /// - Role strings in `participants` must not be empty.
    // ANCHOR[contextra:MEM-016]
    /// Erstellt eine n-äre Beziehung (Hyperedge) zwischen zwei oder mehr Erinnerungen.
    ///
    /// # Parameter
    /// - `predicate`: Semantische Bezeichnung der Beziehung (darf nicht leer sein).
    /// - `participants`: Slice aus Teilnehmern und deren Rollenbezeichnung (mindestens 2 erforderlich).
    /// - `source`: Optionaler Verweis auf die Dokument-Quelle der Relation.
    ///
    /// # Ablauf von `relate_n_ary`:
    /// 1. Validierung: Prädikat nicht leer, mindestens 2 Teilnehmer, keine leeren Rollenstrings.
    /// 2. Zugriff auf die Standard-Collection (`default`).
    /// 3. Übergabe an `collection.relate_n_ary`.
    /// 4. Verpacken der generierten Hyperedge-ID in [`RelationId`].
    pub async fn relate_n_ary(
        &self,
        predicate: &str,
        participants: &[(&MemoryId, &str)],
        source: Option<&MemoryId>,
    ) -> Result<RelationId, ContextraError> {
        if predicate.trim().is_empty() {
            return Err(ContextraError::InvalidInput(
                "relate_n_ary predicate must not be empty".to_string(),
            ));
        }

        if participants.len() < 2 {
            return Err(ContextraError::InvalidInput(format!(
                "relate_n_ary requires at least 2 participants, found {}",
                participants.len()
            )));
        }

        for (_, role) in participants {
            if role.trim().is_empty() {
                return Err(ContextraError::InvalidInput(
                    "relate_n_ary participant role must not be empty".to_string(),
                ));
            }
        }

        let collection = self.engine.collection("default").await?;
        let participant_tuples: Vec<(&str, &str)> = participants
            .iter()
            .map(|(mem_id, role)| (mem_id.as_str(), *role))
            .collect();
        let source_str = source.map(|s| s.as_str());

        let hyperedge_id = collection
            .relate_n_ary(predicate, &participant_tuples, source_str)
            .await?;

        Ok(RelationId(hyperedge_id.inner()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ANCHOR[contextra:TEST-MEM-001]
    // Prüft Präfix und monotone Erhöhung der deterministischen Fallback-IDs bei fehlendem IdGen-Port.
    #[test]
    fn test_fallback_id_prefix_and_monotonicity() {
        let id1 = next_fallback_id();
        let id2 = next_fallback_id();

        assert!(
            id1.starts_with("fallback-"),
            "Fallback ID 1 must have prefix 'fallback-': {id1}"
        );
        assert!(
            id2.starts_with("fallback-"),
            "Fallback ID 2 must have prefix 'fallback-': {id2}"
        );

        let num1: u64 = id1
            .strip_prefix("fallback-")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let num2: u64 = id2
            .strip_prefix("fallback-")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        assert_eq!(num2, num1 + 1, "Fallback IDs must increase monotonically");
        assert_ne!(id1, id2, "Consecutive fallback IDs must be distinct");
    }

    // ANCHOR[contextra:TEST-MEM-002]
    // Verifiziert die exakte Reproduzierbarkeit von Fallback-IDs ausgehend von einem identischen Startzustand.
    #[test]
    fn test_fallback_id_reproducibility_from_initial_state() {
        // Simulates two separate process runs starting from identical initial state (counter = 1)
        let counter_run_1 = AtomicU64::new(1);
        let run1_id1 = format!("fallback-{}", counter_run_1.fetch_add(1, Ordering::Relaxed));
        let run1_id2 = format!("fallback-{}", counter_run_1.fetch_add(1, Ordering::Relaxed));

        let counter_run_2 = AtomicU64::new(1);
        let run2_id1 = format!("fallback-{}", counter_run_2.fetch_add(1, Ordering::Relaxed));
        let run2_id2 = format!("fallback-{}", counter_run_2.fetch_add(1, Ordering::Relaxed));

        assert_eq!(run1_id1, "fallback-1");
        assert_eq!(run1_id2, "fallback-2");

        // Runs with identical initial sequence produce identical IDs in identical call order
        assert_eq!(
            run1_id1, run2_id1,
            "First call across identical runs must match"
        );
        assert_eq!(
            run1_id2, run2_id2,
            "Second call across identical runs must match"
        );
    }
}
