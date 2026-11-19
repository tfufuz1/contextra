// FILE-CONTEXT
// ZWECK: DSGVO-Löschkette Phase 4e: physische HNSW-Reparatur und Slot-Nullung (SPEC contextra-vector §3).
// INVARIANTEN: INV-VEC-CONNECTIVITY-1 (Konnektivität nach Entfernung erhalten, sonst Fehler);
//              INV-VEC-ZEROIZE-1 (Vektorslots sind nach Erfolg vollständig genullt);
//              Fail-Closed: mmap-gestützte Knoten werden NICHT stillschweigend übersprungen.
// NICHT-OFFENSICHTLICH: Die Validierung läuft vollständig vor der ersten Mutation; nach einem Fehler
//              während der Entfernung ist der Index in einem Teilzustand und muss verworfen werden.
// HOTSPOTS: VectorPurger::purge_vectors
// STAND: 2026-11-19

//! Physische Vektor-Bereinigung für die DSGVO-Löschkette.

use std::collections::BTreeSet;
use std::path::Path;

use contextra_durable_fs::flush_dir;
use zeroize::Zeroize;

use crate::error::{Result, VectorError};
use crate::hnsw::{HnswIndex, VectorData};
use crate::types::DocId;

// ANCHOR[contextra-vector:PURGE-OUTCOME]
/// Ergebnis einer physischen Vektor-Bereinigung.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VectorPurgeOutcome {
    /// Anzahl tatsächlich entfernter Vektoren (bereits entfernte IDs zählen nicht).
    pub vectors_purged: u64,
    /// Anzahl reparierter Nachbarkanten.
    ///
    /// Bleibt immer null. VETO-F02 verbietet jede Neuvernetzung beim
    /// Löschen; die Topologie wird ausschließlich durch einen
    /// generationsweisen Neuaufbau ersetzt.
    pub edges_repaired: u64,
    /// Freigegebene Vektor-Bytes (Slots wurden genullt).
    pub bytes_reclaimed: u64,
}

/// Zustandslose Bereinigungsroutine für Vektoren (Phase 4e).
pub struct VectorPurger;

impl VectorPurger {
    /// Tombstoned die Vektoren `doc_ids` und nullt ihre Arena-Slots.
    ///
    /// 1. Validierung: Jede ID wird geprüft; mmap-gestützte Knoten führen zu `Unsupported`,
    ///    da ein Nur-Lese-Mapping nicht genullt werden kann. Vor der ersten Mutation.
    /// 2. Je ID: Slot nullen und tombstonen. Kanten bleiben unverändert
    ///    (VETO-F02); nur ein Rebuild der nächsten Generation repariert die
    ///    Topologie.
    ///    Bereits entfernte oder unbekannte IDs sind idempotent und werden übersprungen.
    /// 3. Zeroize-Prüfung (INV-VEC-ZEROIZE-1): Alle betroffenen Slots sind vollständig Null.
    /// 4. Falls `persist_to` gesetzt: Persistieren und Verzeichnis-Sync via `contextra-durable-fs`.
    ///
    /// # Errors
    /// - `Unsupported`, wenn ein Knoten mmap-gestützt ist.
    /// - `Index`, wenn ein Slot nach dem Nullen nicht Null ist.
    /// - `Io`/`Storage` bei Persistenzfehlern.
    // ANCHOR[contextra-vector:PURGE-VECTORS]
    pub fn purge_vectors(
        index: &mut HnswIndex,
        doc_ids: &[DocId],
        persist_to: Option<&Path>,
    ) -> Result<VectorPurgeOutcome> {
        let mmap_count = mmap_node_count(index);
        let unique: BTreeSet<DocId> = doc_ids.iter().copied().collect();

        // 1. Validierung vor jeder Mutation (Fail-Closed).
        let mut pending: Vec<DocId> = Vec::with_capacity(unique.len());
        for &doc_id in &unique {
            match index.node_idx_for_doc(doc_id) {
                None => {} // bereits entfernt oder nie vorhanden: idempotent
                Some(idx) if idx < mmap_count => {
                    return Err(VectorError::Unsupported(format!(
                        "mmap-backed vector for doc {doc_id:?} cannot be zeroized in place; index rebuild required"
                    )));
                }
                Some(_) => pending.push(doc_id),
            }
        }

        let dim = index.inner.cold.config.dimension;
        let slot_bytes = if index.inner.cold.config.quantize {
            dim
        } else {
            dim.saturating_mul(std::mem::size_of::<f32>())
        };

        // 2. Ausschließlich tombstonen und zeroizen. Das Entfernen oder
        // Ersetzen eingehender Kanten wäre Rewiring und ist durch VETO-F02
        // verboten. Die Slotnummer wird von `do_delete` nicht freigegeben.
        let mut outcome = VectorPurgeOutcome::default();
        let mut purged_slots: Vec<usize> = Vec::with_capacity(pending.len());
        for doc_id in pending {
            let Some(slot) = index.node_idx_for_doc(doc_id) else {
                continue;
            };
            let Some(ram_idx) = slot.checked_sub(mmap_count) else {
                return Err(VectorError::Unsupported(
                    "mmap-backed vector cannot be zeroized in place; generation rebuild required"
                        .to_string(),
                ));
            };

            let max_layer = {
                let mut nodes = index.inner.hot.nodes.write();
                let node = nodes.get_mut(ram_idx).ok_or_else(|| {
                    VectorError::CorruptedStorage("slot is absent from vector arena".to_string())
                })?;
                let layer = node.max_layer;
                match &mut node.vector {
                    VectorData::F32(values) => values.zeroize(),
                    VectorData::U8(values) => values.zeroize(),
                }
                node.doc_id = DocId::MIN;
                layer
            };
            index
                .inner
                .hot
                .arena
                .zeroize_node_connections(ram_idx, max_layer, index.inner.cold.config.m);
            index.inner.do_delete(doc_id)?;
            outcome.vectors_purged += 1;
            outcome.bytes_reclaimed += slot_bytes as u64;
            purged_slots.push(slot);
        }

        // 3. Zeroize-Verifikation (INV-VEC-ZEROIZE-1).
        for &idx in &purged_slots {
            if !slot_is_zero(index, idx, mmap_count) {
                return Err(VectorError::Index(format!(
                    "zeroize verification failed for slot {idx}"
                )));
            }
        }

        // 4. Persistenz mit Verzeichnis-Sync.
        if let Some(path) = persist_to {
            index.save_sync(path)?;
            if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
                flush_dir(parent).map_err(|e| VectorError::Storage(e.to_string()))?;
            }
        }

        Ok(outcome)
    }
}

/// Anzahl der mmap-gestützten Knoten (Knoten mit Index unterhalb dieser Grenze).
fn mmap_node_count(index: &HnswIndex) -> usize {
    index
        .inner
        .cold
        .mmap_index
        .read()
        .as_ref()
        .map(|m| usize::try_from(m.header.node_count()).unwrap_or(usize::MAX))
        .unwrap_or(0)
}

/// Prüft, ob der Vektorslot eines RAM-Knotens vollständig aus Nullen besteht.
fn slot_is_zero(index: &HnswIndex, idx: usize, mmap_count: usize) -> bool {
    let Some(ram_idx) = idx.checked_sub(mmap_count) else {
        return false;
    };
    let nodes = index.inner.hot.nodes.read();
    match nodes.get(ram_idx).map(|n| &n.vector) {
        Some(VectorData::F32(v)) => v.iter().all(|x| x.to_bits() == 0),
        Some(VectorData::U8(v)) => v.iter().all(|&b| b == 0),
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hnsw::HnswConfigBuilder;
    use crate::types::MAX_SEARCH_K;
    use crate::vector_index::VectorIndex;
    use roaring::RoaringBitmap;

    const DIM: usize = 8;
    const N: u32 = 120;

    /// Deterministischer Pseudo-Zufall (LCG) ohne externe Abhängigkeit.
    fn vector_for(seed: u32) -> Vec<f32> {
        let mut state = u64::from(seed).wrapping_mul(6_364_136_223_846_793_005).wrapping_add(1);
        (0..DIM)
            .map(|_| {
                state = state
                    .wrapping_mul(6_364_136_223_846_793_005)
                    .wrapping_add(1_442_695_040_888_963_407);
                ((state >> 40) as f32 / (1u64 << 24) as f32) * 2.0 - 1.0
            })
            .collect()
    }

    fn build_index() -> HnswIndex {
        let cfg = HnswConfigBuilder::new(DIM)
            .max_elements(1024)
            .m(8)
            .ef_construction(64)
            .ef_search(64)
            .build()
            .expect("valid test config");
        let index = HnswIndex::try_new(cfg).expect("index");
        for i in 0..N {
            index
                .inner
                .do_insert(DocId::new(u64::from(i)), &vector_for(i), 1)
                .expect("insert");
        }
        index
    }

    fn purge_set() -> Vec<DocId> {
        (0..N).step_by(8).map(|i| DocId::new(u64::from(i))).collect()
    }

    #[test]
    fn purge_does_not_rewire_the_graph() {
        let mut index = build_index();
        let purge = purge_set();
        let outcome = VectorPurger::purge_vectors(&mut index, &purge, None).expect("purge");

        assert_eq!(outcome.vectors_purged, purge.len() as u64);
        assert_eq!(outcome.edges_repaired, 0, "VETO-F02 forbids rewiring");
        assert_eq!(outcome.bytes_reclaimed, (purge.len() * DIM * 4) as u64);

        // A delete is allowed to degrade graph connectivity. The deterministic
        // rebuild, not deletion, is the only repair path (VETO-F02).
        for removed in purge {
            assert_eq!(index.node_idx_for_doc(removed), None);
        }
    }

    #[test]
    fn purged_slots_are_zeroized_and_never_returned() {
        let mut index = build_index();
        let purge = purge_set();
        VectorPurger::purge_vectors(&mut index, &purge, None).expect("purge");

        let mmap = mmap_node_count(&index);
        for doc in &purge {
            assert_eq!(index.node_idx_for_doc(*doc), None, "doc must be removed from map");
        }
        for slot in (0..N as usize).filter(|i| i % 8 == 0) {
            assert!(slot_is_zero(&index, slot, mmap), "slot {slot} not zeroized");
        }

        for i in 0..N {
            let hits = index.search_knn(&vector_for(i), 10, None).expect("search");
            assert!(hits.iter().all(|h| !purge.contains(&h.doc_id)));
        }
    }

    #[test]
    fn purge_is_idempotent_and_ignores_unknown_ids() {
        let mut index = build_index();
        let purge = purge_set();
        VectorPurger::purge_vectors(&mut index, &purge, None).expect("first purge");

        let mut again = purge.clone();
        again.push(DocId::new(999_999));
        let outcome = VectorPurger::purge_vectors(&mut index, &again, None).expect("second purge");
        assert_eq!(outcome, VectorPurgeOutcome::default());
    }

    #[test]
    fn search_knn_honours_exclusion_mask() {
        let index = build_index();
        let mask: RoaringBitmap = (0..40u32).collect();
        let hits = index
            .search_knn(&vector_for(3), 10, Some(&mask))
            .expect("search");
        assert_eq!(hits.len(), 10, "must backfill to k");
        assert!(hits.iter().all(|h| !mask.contains(u32::try_from(h.doc_id.inner()).unwrap_or(0))));
    }

    #[test]
    fn search_knn_rejects_bad_input() {
        let index = build_index();
        assert!(index.search_knn(&[0.0; DIM + 1], 1, None).is_err());
        let mut nan = vector_for(1);
        nan[0] = f32::NAN;
        assert!(index.search_knn(&nan, 1, None).is_err());
        assert!(index.search_knn(&vector_for(1), MAX_SEARCH_K + 1, None).is_err());
        assert!(index.search_knn(&vector_for(1), 0, None).expect("k=0").is_empty());
    }

    #[test]
    fn persist_writes_file_and_syncs_dir() {
        let mut index = build_index();
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("index.hnsw");
        VectorPurger::purge_vectors(&mut index, &purge_set(), Some(&path)).expect("purge+persist");
        assert!(path.exists());
    }
}
