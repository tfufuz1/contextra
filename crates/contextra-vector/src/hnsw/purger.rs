// FILE-CONTEXT
// ZWECK: Fail-closed Hard-Purge für HNSW Vektor-Indizes (Purger Trait, Welle 2/3).
// INVARIANTEN: Zeroize von Vektoren (f32/u8) und Arena-Slots vor Slot-Freigabe; physische Bereinigung.
// NICHT-OFFENSICHTLICH: Graph-Reparatur entfernt alle eingehenden/ausgehenden Kanten und wählt Entry-Point neu.

use std::sync::atomic::Ordering;
use std::sync::Arc;

use contextra_ports::scanner::{PurgeStats, Purger, ScanTarget};
use contextra_types::{DocId, RequestContext, Result};
use zeroize::Zeroize;

use super::types::{HnswIndex, VectorData};

/// Purger-Implementierung für `WitnessLayer::HnswIndex`.
pub struct HnswPurger {
    index: Arc<HnswIndex>,
}

impl HnswPurger {
    /// Erzeugt einen neuen `HnswPurger`.
    pub fn new(index: Arc<HnswIndex>) -> Self {
        Self { index }
    }
}

/// Erzeugt einen `Arc<dyn Purger>` für einen HNSW Index.
pub fn hnsw_purger(index: Arc<HnswIndex>) -> Arc<dyn Purger> {
    Arc::new(HnswPurger::new(index))
}

impl Purger for HnswPurger {
    fn purge(&self, _ctx: &RequestContext, target: &ScanTarget) -> Result<PurgeStats> {
        let mut total_purged = 0u64;
        let mut bytes_freed = 0u64;

        let _write_lock = self.index.inner.hot.write_mutex.lock();

        let mmap_node_count = self
            .index
            .inner
            .cold
            .mmap_index
            .read()
            .as_ref()
            .map(|m| m.header.node_count() as usize)
            .unwrap_or(0);

        // 1. Identifiziere Ziel-DocIDs/Knoten-Indizes
        let mut targets_to_purge: Vec<(DocId, usize)> = Vec::new();

        {
            let doc_map = self.index.inner.hot.doc_to_node.read();
            let nodes_read = self.index.inner.hot.nodes.read();

            let parsed_target = target
                .target_id
                .parse::<u64>()
                .map(DocId::new)
                .or_else(|_| DocId::from_key(&target.target_id));

            for (ram_idx, node) in nodes_read.iter().enumerate() {
                let global_idx = mmap_node_count + ram_idx;
                let matches_doc_id = if let Ok(td) = parsed_target {
                    node.doc_id == td
                } else {
                    node.doc_id.inner().to_string() == target.target_id
                        || blake3::hash(&node.doc_id.inner().to_le_bytes()).as_bytes()
                            == &target.target_hash
                };

                if matches_doc_id {
                    if !targets_to_purge.iter().any(|(_, idx)| *idx == global_idx) {
                        targets_to_purge.push((node.doc_id, global_idx));
                    }
                }
            }

            if let Ok(td) = parsed_target {
                if let Some(&idx) = doc_map.get(&td.inner()) {
                    if !targets_to_purge.iter().any(|(_, i)| *i == idx) {
                        targets_to_purge.push((td, idx));
                    }
                }
            }
        }

        if targets_to_purge.is_empty() {
            return Ok(PurgeStats {
                purged_units: 0,
                bytes_freed: 0,
            });
        }

        let m = self.index.inner.cold.config.m;
        let dim = self.index.inner.cold.config.dimension;

        for (doc_id, target_idx) in targets_to_purge {
            let target_max_layer = {
                let nodes_read = self.index.inner.hot.nodes.read();
                if target_idx < mmap_node_count {
                    0
                } else {
                    let ram_idx = target_idx - mmap_node_count;
                    nodes_read.get(ram_idx).map(|n| n.max_layer).unwrap_or(0)
                }
            };

            // 2. Nachbarschafts-Kantenbereinigung über alle Ebenen
            let nodes_read = self.index.inner.hot.nodes.read();
            let total_nodes = mmap_node_count + nodes_read.len();

            for layer in 0..=target_max_layer {
                for neighbor_idx in 0..total_nodes {
                    if neighbor_idx == target_idx {
                        continue;
                    }
                    if neighbor_idx >= mmap_node_count {
                        let neighbor_ram_idx = neighbor_idx - mmap_node_count;
                        let conns = self.index.inner.hot.arena.get_ram_node_connections(
                            neighbor_ram_idx,
                            layer,
                            m,
                        );
                        if conns.contains(&(target_idx as u32)) {
                            let updated: Vec<u32> = conns
                                .into_iter()
                                .filter(|&c| c != target_idx as u32)
                                .collect();
                            let _ = self.index.inner.hot.arena.update_backlink(
                                neighbor_ram_idx,
                                layer,
                                m,
                                &updated,
                            );
                        }
                    }
                }
            }
            drop(nodes_read);

            // 3. Zeroize der Vektordaten im Knotenslot
            if target_idx >= mmap_node_count {
                let ram_idx = target_idx - mmap_node_count;
                let mut nodes_write = self.index.inner.hot.nodes.write();
                if ram_idx < nodes_write.len() {
                    let node = &mut nodes_write[ram_idx];
                    match &mut node.vector {
                        VectorData::F32(ref mut v) => {
                            v.iter_mut().for_each(|val| *val = 0.0);
                            v.zeroize();
                        }
                        VectorData::U8(ref mut v) => {
                            v.zeroize();
                        }
                    }
                    node.doc_id = DocId::MIN;
                    bytes_freed += (dim * std::mem::size_of::<f32>()) as u64;
                }
                drop(nodes_write);

                // Zeroize arena connection slots und Freigabe erst NACH Zeroize
                self.index
                    .inner
                    .hot
                    .arena
                    .zeroize_and_free_node(ram_idx, target_max_layer, m);
            }

            // 4. Entfernung aus ID-Map
            self.index
                .inner
                .hot
                .doc_to_node
                .write()
                .remove(&doc_id.inner());

            // 5. Eintrag in Tombstones
            self.index
                .inner
                .cold
                .deleted_nodes
                .write()
                .insert(target_idx as u64);
            self.index
                .inner
                .hot
                .deleted_count
                .fetch_add(1, Ordering::SeqCst);

            // 6. Neuwahl des Entry-Points falls gelöschter Knoten betroffen war
            let current_ep = self.index.inner.hot.get_entry_point();
            if current_ep == Some(target_idx) {
                let deleted = self.index.inner.cold.deleted_nodes.read();
                let nodes_read = self.index.inner.hot.nodes.read();
                let total = mmap_node_count + nodes_read.len();
                let new_ep = (0..total).find(|&i| !deleted.contains(i as u64));
                self.index.inner.hot.set_entry_point(new_ep);
            }

            let current_ram_ep = self.index.inner.hot.get_ram_entry_point();
            if current_ram_ep == Some(target_idx) {
                let deleted = self.index.inner.cold.deleted_nodes.read();
                let nodes_read = self.index.inner.hot.nodes.read();
                let total = mmap_node_count + nodes_read.len();
                let new_ep = (0..total).find(|&i| !deleted.contains(i as u64));
                self.index.inner.hot.set_ram_entry_point(new_ep);
            }

            total_purged += 1;
        }

        // 7. Persistierte Datei neu schreiben & neu mmappen falls mmap_index aktiv
        let mmap_path = {
            let mmap_guard = self.index.inner.cold.mmap_index.read();
            mmap_guard.as_ref().and_then(|m| m.path.clone())
        };

        if let Some(path) = mmap_path {
            self.index.save_internal(&path)?;
            let new_mmap = crate::persistence::MmapIndex::open(&path)?;
            self.index.load_mmap_from_instance(new_mmap)?;
        }

        Ok(PurgeStats {
            purged_units: total_purged,
            bytes_freed,
        })
    }
}
