//! # Konsolidierungs-Locks und Kaskaden-Invalidierung (`consolidation_locks.rs`)
//!
//! Stellt das Guard-Konstrukt [`ConsolidationNodesGuard`] für nebenläufigkeitssicheres Locking
//! und geordnete Kaskaden-Invalidierung veralteter Graphkanten im Schlafzyklus bereit.
//!
//! ## Ring-Zugehörigkeit
//! Teil der Kognitionsschicht (Ring 3). Koordiniert `Collection` (Ring 2) und Graph-Operationen (Ring 0).
//!
//! ## Invarianten
//! - **Nicht-blockierender Erwerb:** Der Lock wird atomar via `try_lock` angefordert, um Livelocks zu verhindern.
//! - **Kanonische Sortierung (Deadlock-Prävention):** Kaskadierende Kanteninvalidierungen werden zwingend
//!   aufsteigend nach numerischer `DocId` sortiert (APM-LOCK-ORDER-INVERSION Schutz).
//! - **Fehlerisolation:** Scheitert die Invalidierung für eine `DocId`, wird dies protokolliert,
//!   während nachfolgende IDs weiter abgearbeitet werden.

// FILE-CONTEXT
// ZWECK: Guard-Konstrukt fuer typsicheres Locking und geordnete Kaskaden-Invalidierung in der Sleep-Cycle-Konsolidierung.
// INVARIANTEN: Lock-Erwerb erfolgt atomic via try_lock. Kaskadierende Kanten-Invalidierungen werden kanonisch nach DocId sortiert (Deadlock-Praevention).
// STAND: TS:2026-09-13T00:00:00Z

//! Guard-Konstrukt fuer typsicheres Locking in der Sleep-Cycle-Konsolidierung.
//!
//! # Warum keine direkte Wiederverwendung von `contextra_graph::session_dag::NodesGuard`?
//! `contextra_graph::session_dag::NodesGuard` ist strikt an `SessionBranchTree`, `AgentStateNode`
//! (`NodeIdx` = `u64`) und `DagEdge` gebunden. In `contextra-db` operieren Konsolidierung und Graph-Invalidierung
//! hingegen auf `Collection<S, V>`, `CsrGraph`, `DocId` und `EntityId`.
//! Aufgrund inkompatibler Datentypen, Lifetimes und Methoden kann `session_dag::NodesGuard` nicht direkt
//! importiert werden. `ConsolidationNodesGuard` implementiert das gleiche strukturelle Muster (einziger
//! Einstiegspunkt fuer geordnete Graph-/Node-Invalidierung waehrend der Konsolidierung).

use contextra_engine::collection::Collection;
use contextra_graph::cascade::CascadeInvalidationReport;
use contextra_ports::{StorageEngine, VectorIndex};
use contextra_types::{DocId, Result};
use tokio::sync::MutexGuard;

/// Guard zur Erzwingung der Lock-Reihenfolge und kanonischen Node-Sortierung waehrend der Konsolidierung.
///
/// Garantiert, dass:
/// 1. `collection.consolidation_guard` exklusiv gehalten wird (`try_lock`),
/// 2. Kaskadierende Graph-Edge-Invalidierungen in kanonisch sortierter `DocId`-Reihenfolge
///    ausgeführt werden, um Lock Order Inversions (APM-LOCK-ORDER-INVERSION) und Deadlocks zu verhindern.
// ANCHOR[contextra-cognition:LOCK-001]
/// Guard zur Erzwingung der Lock-Reihenfolge und kanonischen Node-Sortierung während der Konsolidierung.
pub struct ConsolidationNodesGuard<'a, S: StorageEngine, V: VectorIndex> {
    collection: &'a Collection<S, V>,
    _guard: MutexGuard<'a, ()>,
}

impl<'a, S: StorageEngine, V: VectorIndex> ConsolidationNodesGuard<'a, S, V> {
    /// Versucht, den `consolidation_guard` der Collection zu erwerben.
    /// Gibt `Some(ConsolidationNodesGuard)` zurueck, wenn der Lock frei war, sonst `None`.
    // ANCHOR[contextra-cognition:LOCK-002]
    /// Versucht, das Konsolidierungs-Lock atomar zu erwerben.
    pub fn try_acquire(collection: &'a Collection<S, V>) -> Option<Self> {
        match collection.consolidation_guard.try_lock() {
            Ok(guard) => Some(Self {
                collection,
                _guard: guard,
            }),
            Err(_) => None,
        }
    }

    /// Referenz auf die geschützte Collection.
    #[inline]
    // ANCHOR[contextra-cognition:LOCK-003]
    /// Liefert eine Referenz auf die geschützte Collection.
    pub fn collection(&self) -> &'a Collection<S, V> {
        self.collection
    }

    /// Führt kaskadierende Graph-Edge-Invalidierungen für eine Liste supersedeter Dokumente aus.
    ///
    /// Sortiert die Eingabe-`DocId`s vor der Verarbeitung kanonisch (nach numerischem `doc_id.inner()`),
    /// um Lock Order Inversion (APM-LOCK-ORDER-INVERSION) und Deadlocks zwischen parallelen Durchläufen
    /// oder konkurrierenden Lese/Schreib-Transaktionen zu verhindern.
    // ANCHOR[contextra-cognition:LOCK-004]
    /// Führt kaskadierende Graph-Edge-Invalidierungen in kanonischer Reihenfolge aus.
    pub async fn cascade_invalidate_edges_ordered(
        &self,
        superseded_doc_ids: &[DocId],
        wal_seq: u64,
    ) -> (Vec<(DocId, Result<CascadeInvalidationReport>)>, Vec<String>) {
        if superseded_doc_ids.is_empty() {
            return (Vec::new(), Vec::new());
        }

        // Kanonische Sortierung nach DocId zur Vermeidung von Lock-Order-Inversion
        let mut sorted_doc_ids = superseded_doc_ids.to_vec();
        sorted_doc_ids.sort_by_key(|id| id.inner());
        sorted_doc_ids.dedup();

        let mut reports = Vec::with_capacity(sorted_doc_ids.len());
        let mut errors = Vec::new();

        for doc_id in sorted_doc_ids {
            match contextra_graph::cascade_invalidate_edges_for_superseded_doc(
                &self.collection.graph_index(),
                doc_id,
                wal_seq,
            )
            .await
            {
                Ok(report) => {
                    tracing::debug!(
                        doc_id = ?doc_id,
                        tombstoned_edges = report.tombstoned_edge_count,
                        "Consolidation pass: cascade edge invalidation successful"
                    );
                    reports.push((doc_id, Ok(report)));
                }
                Err(e) => {
                    tracing::warn!(
                        doc_id = ?doc_id,
                        error = %e,
                        "Consolidation pass: cascade edge invalidation failed"
                    );
                    errors.push(format!("DocId {:?}: {}", doc_id, e));
                    reports.push((doc_id, Err(e)));
                }
            }
        }

        (reports, errors)
    }
}
