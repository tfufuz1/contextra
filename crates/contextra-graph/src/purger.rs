//! Neutrale Wartungs- und Bereinigungs-Implementierung für den Graph-Baustein (`contextra-ports::scanner::Purger`).
//!
//! # Zweck
//! Ermöglicht die physische Bereinigung von Graph-Kanten und Knoten für ein gegebenes `ScanTarget`.
//!
//! # Baukasten-Hinweis (D-E1)
//! Löschschnittstellen und Löschtypen werden gemäß Architektur-Entscheidung D-E1 künftig
//! in `contextra-erasure-spi` geführt. Bis dahin bleibt diese Implementierung übergangsweise
//! als deprecated markiert erhalten, um die Abwärtskompatibilität des Workspace zu wahren.

#![allow(deprecated)]

use std::sync::Arc;

use contextra_ports::scanner::{PurgeStats, Purger, ScanTarget};
use contextra_types::{DocId, RequestContext, Result, TxId};

use crate::csr::CsrGraph;

/// Purger-Implementierung für [`CsrGraph`].
#[deprecated(
    note = "Entscheidung D-E1: Löschschnittstellen werden nach contextra-erasure-spi migriert; neutrale Wartungs-API verwenden"
)]
pub struct GraphPurger {
    graph: Arc<CsrGraph>,
}

impl GraphPurger {
    /// Erzeugt einen neuen [`GraphPurger`].
    pub fn new(graph: Arc<CsrGraph>) -> Self {
        Self { graph }
    }
}

/// Erzeugt einen `Arc<dyn Purger>` für einen gegebenen [`CsrGraph`].
#[deprecated(
    note = "Entscheidung D-E1: Löschschnittstellen werden nach contextra-erasure-spi migriert; neutrale Wartungs-API verwenden"
)]
pub fn graph_purger(graph: Arc<CsrGraph>) -> Arc<dyn Purger> {
    Arc::new(GraphPurger::new(graph))
}

impl Purger for GraphPurger {
    fn purge(&self, _ctx: &RequestContext, target: &ScanTarget) -> Result<PurgeStats> {
        let mut total_purged = 0u64;
        let mut bytes_freed = 0u64;

        // Versuche target_id als DocId zu parsen
        let doc_id = target
            .target_id
            .parse::<u64>()
            .map(DocId::new)
            .or_else(|_| DocId::from_key(&target.target_id));

        if let Ok(doc) = doc_id {
            let edge_ids = self.graph.edges_for_doc(doc);
            if !edge_ids.is_empty() {
                let tx = TxId::new(u64::MAX);
                let (count, _, _) = self.graph.tombstone_edges_direct(&edge_ids, tx)?;
                total_purged += count as u64;
                bytes_freed += (count * std::mem::size_of::<(u32, u32, f32)>()) as u64;
                self.graph.doc_edge_index.remove_doc(doc);
            }
        }

        Ok(PurgeStats {
            purged_units: total_purged,
            bytes_freed,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use contextra_ports::GraphIndex;
    use contextra_types::{Edge, Entity, EntityId};

    #[tokio::test]
    async fn test_graph_purger_empty_target() {
        let graph = Arc::new(CsrGraph::new());
        let purger = graph_purger(graph);
        let ctx = RequestContext::default();
        let target = ScanTarget::new("12345", [0u8; 32]);

        let stats = purger.purge(&ctx, &target).expect("purge should succeed");
        assert_eq!(stats.purged_units, 0);
        assert_eq!(stats.bytes_freed, 0);
    }

    #[tokio::test]
    async fn test_graph_purger_existing_edges() {
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
        edge.doc_id = Some(doc_id);
        graph.add_edge(tx, edge).await.unwrap();
        graph.commit(tx).await.unwrap();

        // Register in doc_edge_index
        graph.doc_edge_index.record(doc_id, (id_a, id_b));

        let purger = graph_purger(graph.clone());
        let ctx = RequestContext::default();
        let target = ScanTarget::new("42", [0u8; 32]);

        let stats = purger.purge(&ctx, &target).expect("purge should succeed");
        assert_eq!(stats.purged_units, 1);
        assert!(stats.bytes_freed > 0);
    }
}
