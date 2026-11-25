// FILE-CONTEXT
// STAND: 2026-10-05 (SESSION: provenance_delete_cascade)
// ZWECK: Herkunftsnachweis für Graph-Kanten (DocEdgeIndex & EdgeProvenance)
// INVARIANTEN: INV-GRAPH-PROV-1: Jede CSR-Kante ordnet sich ihren Quelldokumenten zu.
// SIEHE AUCH: crates/contextra-graph/src/cascade.rs

use crate::consistency_enforcement::EdgeId;
use ahash::{AHashMap, AHashSet};
use contextra_types::{DocId, TxId};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

/// Herkunftsnachweis für eine Graph-Kante.
/// INV-GRAPH-PROV-1: Jede aktive CSR-Kante trägt einen EdgeProvenance-Eintrag,
/// der ihre Quell-Dokumente zurückverfolgbar macht.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeProvenance {
    /// Identifikator der Kante (from_entity_id, to_entity_id).
    pub edge_id: EdgeId,
    /// Dokumente deren Inhalt zu dieser Kante beigetragen hat.
    pub source_doc_ids: Vec<DocId>,
    /// Transaktions-ID bei der Erstellung der Kante.
    pub created_at_tx: TxId,
}

impl EdgeProvenance {
    pub fn new(edge_id: EdgeId, source_doc_ids: Vec<DocId>, created_at_tx: TxId) -> Self {
        Self {
            edge_id,
            source_doc_ids,
            created_at_tx,
        }
    }
}

/// Rückverfolgung DocId -> betroffene Kanten, für Cascading-Invalidation (INV-GRAPH-PROV-1).
/// Speichert sowohl die Zuordnung DocId -> Kanten als auch EdgeId -> EdgeProvenance.
#[derive(Debug, Default)]
pub struct DocEdgeIndex {
    doc_to_edges: RwLock<AHashMap<DocId, AHashSet<EdgeId>>>,
    edge_provenance: RwLock<AHashMap<EdgeId, EdgeProvenance>>,
}

impl DocEdgeIndex {
    pub fn new() -> Self {
        Self {
            doc_to_edges: RwLock::new(AHashMap::new()),
            edge_provenance: RwLock::new(AHashMap::new()),
        }
    }

    /// Registriert die Abhängigkeit einer Kante von einem Dokument.
    pub fn record(&self, doc_id: DocId, edge_id: EdgeId) {
        let prov = EdgeProvenance::new(edge_id, vec![doc_id], TxId::new(0));
        self.record_provenance(&prov);
    }

    /// Registriert einen EdgeProvenance-Eintrag und indiziert alle darin enthaltenen source_doc_ids.
    pub fn record_provenance(&self, provenance: &EdgeProvenance) {
        let mut doc_guard = self.doc_to_edges.write();
        let mut prov_guard = self.edge_provenance.write();

        for &doc_id in &provenance.source_doc_ids {
            doc_guard
                .entry(doc_id)
                .or_default()
                .insert(provenance.edge_id);
        }

        if let Some(existing) = prov_guard.get_mut(&provenance.edge_id) {
            for &doc_id in &provenance.source_doc_ids {
                if !existing.source_doc_ids.contains(&doc_id) {
                    existing.source_doc_ids.push(doc_id);
                }
            }
        } else {
            prov_guard.insert(provenance.edge_id, provenance.clone());
        }
    }

    /// Gibt alle Kanten zurück, die von diesem Dokument abhängen.
    pub fn edges_for_doc(&self, doc_id: DocId) -> Vec<EdgeId> {
        let guard = self.doc_to_edges.read();
        guard
            .get(&doc_id)
            .map(|set| set.iter().copied().collect())
            .unwrap_or_default()
    }

    /// Gibt das `EdgeProvenance` einer Kante zurück, falls vorhanden.
    pub fn provenance_for_edge(&self, edge_id: EdgeId) -> Option<EdgeProvenance> {
        let guard = self.edge_provenance.read();
        guard.get(&edge_id).cloned()
    }

    /// Entfernt ein Dokument aus dem Herkunftsindex (DSGVO-Kaskaden-Löschung).
    ///
    /// # Semantik (INV-GRAPH-PROV-1)
    /// Beim Löschen von Dokument D:
    /// (a) D wird aus allen `source_doc_ids` der betroffenen Kanten entfernt.
    /// (b) Kanten, deren `source_doc_ids` danach LEER sind, werden in einer `Vec<EdgeId>` zurückgegeben,
    ///     damit der Aufrufer sie im Graph tombstonieren/löschen kann.
    /// (c) Kanten mit weiteren Quell-Dokumenten bleiben im Graph und Index bestehen.
    ///
    /// Ein mehrfacher Aufruf mit derselben `DocId` oder eine unbekannte `DocId` ist ein idempotent safe No-Op
    /// und liefert einen leeren Vektor zurück.
    pub fn remove_doc(&self, doc_id: DocId) -> Vec<EdgeId> {
        let mut doc_guard = self.doc_to_edges.write();
        let mut prov_guard = self.edge_provenance.write();

        let mut tombstone_candidates = Vec::new();

        if let Some(edges) = doc_guard.remove(&doc_id) {
            for edge_id in edges {
                if let Some(prov) = prov_guard.get_mut(&edge_id) {
                    prov.source_doc_ids.retain(|&d| d != doc_id);
                    if prov.source_doc_ids.is_empty() {
                        tombstone_candidates.push(edge_id);
                        prov_guard.remove(&edge_id);
                    }
                } else {
                    // Fallback: Kante war in doc_to_edges registriert, hatte aber sonst keine Dokumente
                    tombstone_candidates.push(edge_id);
                }
            }
        }

        tombstone_candidates
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use contextra_types::EntityId;

    #[test]
    fn test_doc_edge_index_record_and_retrieve() {
        let index = DocEdgeIndex::new();
        let doc1 = DocId::new(100);
        let doc2 = DocId::new(200);
        let edge1 = (EntityId::new(1), EntityId::new(2));
        let edge2 = (EntityId::new(2), EntityId::new(3));

        index.record(doc1, edge1);
        index.record(doc1, edge2);
        index.record(doc2, edge2);

        let edges_doc1 = index.edges_for_doc(doc1);
        assert_eq!(edges_doc1.len(), 2);
        assert!(edges_doc1.contains(&edge1));
        assert!(edges_doc1.contains(&edge2));

        let edges_doc2 = index.edges_for_doc(doc2);
        assert_eq!(edges_doc2.len(), 1);
        assert!(edges_doc2.contains(&edge2));

        let tombstoned_doc1 = index.remove_doc(doc1);
        // edge1 had only doc1 -> tombstoned
        // edge2 had doc1 and doc2 -> remaining doc2, not tombstoned
        assert_eq!(tombstoned_doc1, vec![edge1]);
        assert!(index.edges_for_doc(doc1).is_empty());
        assert_eq!(index.edges_for_doc(doc2).len(), 1);
    }

    #[test]
    fn test_record_provenance() {
        let index = DocEdgeIndex::new();
        let doc1 = DocId::new(10);
        let doc2 = DocId::new(20);
        let edge = (EntityId::new(5), EntityId::new(6));

        let prov = EdgeProvenance::new(edge, vec![doc1, doc2], TxId::new(1));
        index.record_provenance(&prov);

        assert_eq!(index.edges_for_doc(doc1), vec![edge]);
        assert_eq!(index.edges_for_doc(doc2), vec![edge]);

        let stored_prov = index.provenance_for_edge(edge).expect("provenance exists");
        assert_eq!(stored_prov.source_doc_ids, vec![doc1, doc2]);
    }

    #[test]
    fn test_record_provenance_empty_docs() {
        let index = DocEdgeIndex::new();
        let edge = (EntityId::new(1), EntityId::new(2));
        let prov = EdgeProvenance::new(edge, vec![], TxId::new(100));

        index.record_provenance(&prov);
        assert!(index.edges_for_doc(DocId::new(999)).is_empty());
    }

    #[test]
    fn test_doc_edge_index_nonexistent_doc() {
        let index = DocEdgeIndex::new();
        assert!(index.edges_for_doc(DocId::new(404)).is_empty());

        let res = index.remove_doc(DocId::new(404)); // must not panic and return empty
        assert!(res.is_empty());
        assert!(index.edges_for_doc(DocId::new(404)).is_empty());
    }

    #[test]
    fn test_doc_edge_index_duplicate_records() {
        let index = DocEdgeIndex::new();
        let doc = DocId::new(1);
        let edge = (EntityId::new(10), EntityId::new(20));

        index.record(doc, edge);
        index.record(doc, edge);

        let edges = index.edges_for_doc(doc);
        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0], edge);
    }

    #[test]
    fn test_edge_provenance_serde_roundtrip() {
        let edge = (EntityId::new(42), EntityId::new(84));
        let prov = EdgeProvenance::new(
            edge,
            vec![DocId::new(1001), DocId::new(1002)],
            TxId::new(500),
        );

        let json = serde_json::to_string(&prov).expect("serialization failed");
        let deserialized: EdgeProvenance =
            serde_json::from_str(&json).expect("deserialization failed");

        assert_eq!(deserialized.edge_id, prov.edge_id);
        assert_eq!(deserialized.source_doc_ids, prov.source_doc_ids);
        assert_eq!(deserialized.created_at_tx, prov.created_at_tx);
    }

    #[test]
    fn test_remove_doc_multi_sources_cascade_rules() {
        let index = DocEdgeIndex::new();
        let doc1 = DocId::new(1);
        let doc2 = DocId::new(2);
        let doc3 = DocId::new(3);

        let edge_a = (EntityId::new(10), EntityId::new(20)); // Doc1 only
        let edge_b = (EntityId::new(20), EntityId::new(30)); // Doc1 + Doc2
        let edge_c = (EntityId::new(30), EntityId::new(40)); // Doc2 only

        index.record_provenance(&EdgeProvenance::new(edge_a, vec![doc1], TxId::new(1)));
        index.record_provenance(&EdgeProvenance::new(edge_b, vec![doc1, doc2], TxId::new(1)));
        index.record_provenance(&EdgeProvenance::new(edge_c, vec![doc2], TxId::new(1)));

        // Remove doc1: Edge A is tombstoned, Edge B remains (doc2)
        let removed_doc1 = index.remove_doc(doc1);
        assert_eq!(removed_doc1, vec![edge_a]);

        let prov_b = index.provenance_for_edge(edge_b).expect("edge_b exists");
        assert_eq!(prov_b.source_doc_ids, vec![doc2]);

        // Remove unknown doc3: No-op
        let removed_doc3 = index.remove_doc(doc3);
        assert!(removed_doc3.is_empty());

        // Repeat remove doc1: Idempotent
        let removed_doc1_again = index.remove_doc(doc1);
        assert!(removed_doc1_again.is_empty());

        // Remove doc2: Edge B and Edge C are tombstoned
        let mut removed_doc2 = index.remove_doc(doc2);
        removed_doc2.sort();
        let mut expected = vec![edge_b, edge_c];
        expected.sort();
        assert_eq!(removed_doc2, expected);
    }
}
