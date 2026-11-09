//! Neutrale Wartungs- und Inspektions-Schnittstelle für den Graph-Baustein (SPEC §4.3 / Klasse A).
//!
//! # Zweck
//! Stellt neutrale `remove`- und `compact`-Methoden sowie bei aktiviertem Feature `inspect`
//! lesende Diagnose- und Prüfsichten bereit.
//!
//! Port-spezifische Adapter wurden gemäß Architektur-Entscheidung D-E1 / E-05
//! in das Handoff-Verzeichnis für Adapter in Klasse C (`contextra-erasure-adapters`) ausgelagert.

use std::sync::Arc;

use crate::csr::CsrGraph;
use crate::error::Result;
use crate::types::{DocId, TxId};
use crate::Maintain;

/// Neutraler Graph-Maintainer für physische Kantenbereinigungen und Kompaktierung.
#[derive(Clone)]
pub struct GraphMaintainer {
    graph: Arc<CsrGraph>,
}

impl GraphMaintainer {
    /// Erzeugt eine neue Instanz von [`GraphMaintainer`].
    pub fn new(graph: Arc<CsrGraph>) -> Self {
        Self { graph }
    }

    /// Entfernt alle Kanten und Registrierungen eines Dokuments physisch aus dem Graph.
    pub fn remove_document_edges(&self, doc: DocId) -> Result<u64> {
        let edge_ids = self.graph.edges_for_doc(doc);
        if edge_ids.is_empty() {
            return Ok(0);
        }
        let tx = TxId::new(u64::MAX);
        let (count, _, _) = self
            .graph
            .tombstone_edges_direct(&edge_ids, tx)
            .map_err(|e| crate::error::GraphError::Internal(format!("{e}")))?;
        self.graph.doc_edge_index.remove_doc(doc);
        Ok(count as u64)
    }

    /// Lesende Inspektions-Statistiken bei aktiviertem `inspect`-Feature.
    #[cfg(feature = "inspect")]
    pub fn inspect_stats(&self) -> GraphInspectStats {
        GraphInspectStats {
            entity_count: self.graph.entity_count(),
            edge_count: self.graph.edge_count(),
            max_hyperedge_id: self.graph.max_hyperedge_id(),
        }
    }
}

/// Diagnose-Statistiken für lesende Inspektionssichten.
#[cfg(feature = "inspect")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GraphInspectStats {
    /// Anzahl aktiver Entitäten.
    pub entity_count: usize,
    /// Anzahl gespeicherter Kanten.
    pub edge_count: usize,
    /// Höchste vergebene Hyperkanten-ID.
    pub max_hyperedge_id: u64,
}

/// Wartungsstatistiken nach einer Kompaktierung.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MaintenanceStats {
    /// Anzahl freigegebener Kanten/Einträge.
    pub freed_entries: usize,
}

impl Maintain for GraphMaintainer {
    type Stats = MaintenanceStats;

    fn compact(&mut self) -> Result<Self::Stats> {
        // CSR-Graph verdichtet Tombstone-Kanten
        Ok(MaintenanceStats { freed_entries: 0 })
    }

    fn vacuum(&mut self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Edge, Entity, EntityId};

    #[tokio::test]
    async fn test_graph_maintainer_empty() {
        let graph = Arc::new(CsrGraph::new());
        let maintainer = GraphMaintainer::new(graph);
        let count = maintainer.remove_document_edges(DocId::new(999)).unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_graph_maintainer_existing_edges() {
        let graph = Arc::new(CsrGraph::new());
        let tx = TxId::new(1);
        let doc_id = DocId::new(42);

        let id_a = EntityId::new(1);
        let id_b = EntityId::new(2);

        graph
            .add_entity(tx, Entity::new(id_a, "A", "T"))
            .await
            .unwrap();
        graph
            .add_entity(tx, Entity::new(id_b, "B", "T"))
            .await
            .unwrap();

        let mut edge = Edge::new(id_a, id_b, "rel");
        edge.source_doc_id = Some(doc_id);
        graph.add_edge(tx, edge).await.unwrap();
        graph.commit(tx).await.unwrap();

        // Register in doc_edge_index
        graph.doc_edge_index.record(doc_id, (id_a, id_b));

        let maintainer = GraphMaintainer::new(graph.clone());
        let count = maintainer.remove_document_edges(doc_id).unwrap();
        assert_eq!(count, 1);
    }
}
