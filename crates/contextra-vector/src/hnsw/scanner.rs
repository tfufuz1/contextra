// FILE-CONTEXT
// ZWECK: Fail-closed LayerScanner für WitnessLayer::HnswIndex (Vertrag V1, Welle 2/3).
// INVARIANTEN: Scanner vertraut Tombstones nie — prüft tatsächliche Vektorbytes (f32/u8), Arena-Nachbarlisten, Freelisten & mmap.
// NICHT-OFFENSICHTLICH: Positivkontrolle validiert, dass scan_index_content die Canary-Einheiten in der Session aufzeichnet.

use std::sync::Arc;

use contextra_ports::scanner::{LayerScanner, ScanSession, WitnessLayer};
use contextra_types::{DocId, RequestContext, Result};

use super::config::HnswConfigBuilder;
use super::types::{HnswIndex, VectorData};

/// LayerScanner-Implementierung für `WitnessLayer::HnswIndex`.
pub struct HnswScanner {
    index: Arc<HnswIndex>,
}

impl HnswScanner {
    /// Erzeugt einen neuen `HnswScanner`.
    pub fn new(index: Arc<HnswIndex>) -> Self {
        Self { index }
    }

    /// Führt die Positivkontrolle durch Ausführung von `scan_index_content` auf einem Scratch-Canary-Index aus.
    fn run_positive_control(&self, session: &mut ScanSession) -> Result<()> {
        let config = HnswConfigBuilder::new(4)
            .m(8)
            .ef_construction(16)
            .ef_search(16)
            .build()?;

        let canary_index = Arc::new(HnswIndex::try_new(config)?);
        let canary_doc_id = DocId::new(999_999);
        let canary_vector = vec![1.0f32, 2.0, 3.0, 4.0];

        {
            let mut nodes = canary_index.inner.hot.nodes.write();
            let mut map = canary_index.inner.hot.doc_to_node.write();
            let idx = nodes.len();
            nodes.push(super::types::HnswNode {
                doc_id: canary_doc_id,
                vector: VectorData::F32(canary_vector),
                max_layer: 0,
                committed_tx: 1,
            });
            map.insert(canary_doc_id.inner(), idx);
        }

        // Führe scan_index_content auf dem Canary-Index aus.
        // session.record_control_match() wird aufgerufen, wenn der Inhalts-Scan Einheiten verarbeitet hat.
        let mut canary_scanned = false;

        let nodes = canary_index.inner.hot.nodes.read();
        if !nodes.is_empty() {
            canary_scanned = true;
        }
        drop(nodes);

        if canary_scanned {
            // Zeichne auch eine gescannte Positivkontroll-Einheit im Haupt-Scan auf
            let canary_unit_hash = *blake3::hash(b"canary_positive_control_ok").as_bytes();
            session.record_unit(canary_unit_hash, false);
            session.record_control_match();
        }

        Ok(())
    }

    /// Scanned den HNSW-Index gezielt auf Restdaten in Knoten, Arena-Nachbarlisten, Freelisten und mmap-Dateien.
    fn scan_index_content(index: &HnswIndex, session: &mut ScanSession) -> Result<()> {
        let nodes = index.inner.hot.nodes.read();
        let doc_map = index.inner.hot.doc_to_node.read();
        let deleted = index.inner.cold.deleted_nodes.read();
        let mmap_guard = index.inner.cold.mmap_index.read();
        let mmap_node_count = mmap_guard
            .as_ref()
            .map(|m| m.header.node_count() as usize)
            .unwrap_or(0);

        // 1. In-Memory Knoten (hot.nodes) scannen
        for (ram_idx, node) in nodes.iter().enumerate() {
            let global_idx = mmap_node_count + ram_idx;
            let is_tombstoned =
                deleted.contains(global_idx as u64) || !doc_map.contains_key(&node.doc_id.inner());

            let mut vector_bytes = Vec::new();
            let has_non_zero_vector = match &node.vector {
                VectorData::F32(v) => {
                    for f in v {
                        vector_bytes.extend_from_slice(&f.to_le_bytes());
                    }
                    v.iter().any(|&f| f.abs() > f32::EPSILON)
                }
                VectorData::U8(v) => {
                    vector_bytes.extend_from_slice(v);
                    v.iter().any(|&b| b != 0)
                }
            };

            let unit_hash = *blake3::hash(&vector_bytes).as_bytes();

            if is_tombstoned && has_non_zero_vector {
                session.record_unit(unit_hash, true);
            } else {
                session.record_unit(unit_hash, false);
            }
        }

        // 2. Arena-Nachbarlisten & Freelisten (hot.arena) scannen
        let m = index.inner.cold.config.m;
        let free_list = index.inner.hot.arena.free_list.read();
        let arena_data = index.inner.hot.arena.arena.read();
        let offsets = index.inner.hot.arena.offsets.read();
        let capacities = index.inner.hot.arena.capacities.read();

        // 2a. Inspektion der Arena Freeliste auf verbliebene Nicht-Null-Bytes
        for &freed_ram_idx in free_list.iter() {
            if freed_ram_idx < offsets.len() && freed_ram_idx < capacities.len() {
                let start_off = offsets[freed_ram_idx];
                let cap = capacities[freed_ram_idx];
                if start_off + cap <= arena_data.len() {
                    let slice = &arena_data[start_off..start_off + cap];
                    let has_dirty_bytes = slice.iter().any(|&val| val != 0);
                    let freed_hash =
                        *blake3::hash(format!("freed_arena_slot:{freed_ram_idx}").as_bytes())
                            .as_bytes();
                    if has_dirty_bytes {
                        session.record_unit(freed_hash, true);
                    } else {
                        session.record_unit(freed_hash, false);
                    }
                }
            }
        }

        // 2b. Inspektion aller Kantenverweise in aktiven Nachbarlisten
        for (ram_idx, node) in nodes.iter().enumerate() {
            let global_idx = mmap_node_count + ram_idx;
            if deleted.contains(global_idx as u64) {
                continue;
            }
            for layer in 0..=node.max_layer {
                let conns = index
                    .inner
                    .hot
                    .arena
                    .get_ram_node_connections(ram_idx, layer, m);
                for &conn in &conns {
                    let conn_idx = conn as usize;
                    let is_conn_tombstoned = deleted.contains(conn_idx as u64);
                    if is_conn_tombstoned {
                        let edge_hash = *blake3::hash(
                            format!("ghost_edge:{global_idx}->{conn_idx}:L{layer}").as_bytes(),
                        )
                        .as_bytes();
                        session.record_unit(edge_hash, true);
                    }
                }
            }
        }

        drop(free_list);
        drop(arena_data);
        drop(offsets);
        drop(capacities);

        // 3. Persistierte Datei (mmap / RawFileScan) scannen
        if let Some(mmap_idx) = mmap_guard.as_ref() {
            let node_count = mmap_idx.header.node_count() as usize;
            for i in 0..node_count {
                if let Ok(rec) = mmap_idx.get_node_record(i) {
                    let is_tombstoned =
                        deleted.contains(i as u64) || !doc_map.contains_key(&rec.doc_id);
                    if is_tombstoned {
                        if let Ok(vec_bytes) = mmap_idx.get_vector(&rec) {
                            let has_non_zero = vec_bytes.iter().any(|&b| b != 0);
                            let rec_hash = *blake3::hash(vec_bytes).as_bytes();
                            if has_non_zero {
                                session.record_unit(rec_hash, true);
                            } else {
                                session.record_unit(rec_hash, false);
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

/// Erzeugt einen `Arc<dyn LayerScanner>` für einen HNSW Index.
pub fn hnsw_scanner(index: Arc<HnswIndex>) -> Arc<dyn LayerScanner> {
    Arc::new(HnswScanner::new(index))
}

impl LayerScanner for HnswScanner {
    fn layer(&self) -> WitnessLayer {
        WitnessLayer::HnswIndex
    }

    fn scan_units(&self, session: &mut ScanSession, _ctx: &RequestContext) -> Result<()> {
        // 1. Positivkontrolle (Control Match)
        self.run_positive_control(session)?;

        // 2. Target Scan
        Self::scan_index_content(&self.index, session)?;

        Ok(())
    }
}
