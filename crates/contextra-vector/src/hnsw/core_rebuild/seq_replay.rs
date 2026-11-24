// FILE-CONTEXT
// ZWECK: Phase-2 Delta-Replay via MVCC-Sequenzlog und atomarer Swap unter write_mutex.
// INVARIANTEN: INV-INDEX-SNAPSHOT-1 (born_seq/died_seq Filterung); SSI-Determinismus; Swap unter write_mutex.
// HOTSPOTS: replay_delta_and_swap, ReplayDecision, compute_replay_decisions
// STAND: TS:2026-11-11T02:00:00Z

//! Sequenzlog-Wiedergabe für den HNSW-Rebuild (Phase 2).
//!
//! #[derive(Debug, Clone)]Invariante INV-INDEX-SNAPSHOT-1:
//! Für jedes Dokument im Index gilt ein wohldefinierter Transaktions-Sichtbarkeitsbereich:
//! - `born_seq`: Transaktionssequenznummer der Einfügung.
//! - `died_seq`: Transaktionssequenznummer der Löschung (Tombstone).
//!
//! Änderungen im MVCC-Sequenzlog (`contextra_mvcc::SeqLogChange`) seit dem Transaktionszeitpunkt
//! des Snapshots (`snapshot_tx`) werden in strikter WAL-Reihenfolge nachgespielt, bevor die
//! neuen Speicherstrukturen atomar unter `write_mutex` ausgetauscht werden.

use roaring::RoaringTreemap;
use std::sync::atomic::Ordering;

use crate::error::{Result, VectorError};
use crate::hnsw::types::{HnswIndex, HnswIndexCore, VectorData};
use crate::types::DocId;

/// Repräsentiert eine isolierte Insert- oder Delete-Entscheidung aus dem Sequenzlog.
#[derive(Debug, Clone)]
pub(super) enum ReplayDecision {
    Insert {
        doc_id: DocId,
        vector: VectorData,
        seq: u64,
    },
    Delete {
        doc_id: DocId,
    },
}

/// Extrahiert und filtert Sequenzlog-Änderungen in chronologischer WAL-Reihenfolge.
pub(super) fn compute_replay_decisions(
    core: &HnswIndexCore,
    snapshot_tx: u64,
) -> Result<Vec<ReplayDecision>> {
    let delta_changes = core.cold.seq_log.read().changes_since(snapshot_tx);
    let mut decisions = Vec::with_capacity(delta_changes.len());

    for change in delta_changes {
        match change {
            contextra_mvcc::SeqLogChange::Insert { doc_id, seq } => {
                let vector_opt = {
                    let doc_map = core.hot.doc_to_node.read();
                    let mmap_count = core
                        .cold
                        .mmap_index
                        .read()
                        .as_ref()
                        .map(|m| m.header.node_count() as usize)
                        .unwrap_or(0);
                    if let Some(&global_idx) = doc_map.get(&doc_id.inner()) {
                        if global_idx >= mmap_count {
                            let ram_idx = global_idx - mmap_count;
                            let nodes = core.hot.nodes.read();
                            nodes.get(ram_idx).map(|n| n.vector.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                };

                if let Some(vector) = vector_opt {
                    decisions.push(ReplayDecision::Insert {
                        doc_id,
                        vector,
                        seq,
                    });
                }
            }
            contextra_mvcc::SeqLogChange::Delete { doc_id, .. } => {
                decisions.push(ReplayDecision::Delete { doc_id });
            }
        }
    }

    Ok(decisions)
}

/// Führt Phase 2 des Rebuilds aus: Wendet Änderungen nach dem Snapshot-Tx an und tauscht Kern-Puffer aus.
///
/// Invariante (INV-INDEX-SNAPSHOT-1 & SSI-Determinismus):
/// Der Puffer-Swap erfolgt atomar unter gehaltener `write_mutex`. Leser behalten deterministische Sicht.
pub(super) fn replay_delta_and_swap(
    core: &HnswIndexCore,
    new_index: HnswIndex,
    snapshot_tx: u64,
) -> Result<()> {
    let _write_lock = core.hot.write_mutex.lock();

    let decisions = compute_replay_decisions(core, snapshot_tx)?;

    for decision in decisions {
        match decision {
            ReplayDecision::Insert {
                doc_id,
                vector,
                seq,
            } => {
                let f32_vec = match vector {
                    VectorData::F32(v) => v,
                    VectorData::U8(v) => {
                        let q_guard = core.cold.quantizer.read();
                        let q = q_guard.as_ref().ok_or_else(|| {
                            VectorError::Index(
                                "Quantizer missing during rebuild phase 2 delta replay".into(),
                            )
                        })?;
                        q.dequantize(&v)?
                    }
                };

                new_index.inner.do_insert(doc_id, &f32_vec, seq)?;
            }
            ReplayDecision::Delete { doc_id } => {
                new_index.inner.do_delete(doc_id)?;
            }
        }
    }

    {
        let mut nodes = core.hot.nodes.write();
        let mut doc_to_node = core.hot.doc_to_node.write();
        let mut deleted_nodes = core.cold.deleted_nodes.write();
        let mut offsets = core.hot.arena.offsets.write();
        let mut capacities = core.hot.arena.capacities.write();
        let mut count_offsets = core.hot.arena.count_offsets.write();
        let mut counts = core.hot.arena.counts.write();
        let mut arena = core.hot.arena.arena.write();

        let new_nodes = std::mem::take(&mut *new_index.inner.hot.nodes.write());
        let new_doc_to_node = std::mem::take(&mut *new_index.inner.hot.doc_to_node.write());
        let new_entry_point = new_index.inner.hot.get_entry_point();
        let new_ram_entry_point = new_index.inner.hot.get_ram_entry_point();

        let new_offsets = std::mem::take(&mut *new_index.inner.hot.arena.offsets.write());
        let new_capacities = std::mem::take(&mut *new_index.inner.hot.arena.capacities.write());
        let new_count_offsets =
            std::mem::take(&mut *new_index.inner.hot.arena.count_offsets.write());
        let new_counts = std::mem::take(&mut *new_index.inner.hot.arena.counts.write());
        let new_arena = std::mem::take(&mut *new_index.inner.hot.arena.arena.write());

        let new_quantizer = new_index.inner.cold.quantizer.write().take();
        if new_quantizer.is_some() {
            *core.cold.quantizer.write() = new_quantizer;
        }

        *nodes = new_nodes;
        *doc_to_node = new_doc_to_node;
        core.hot.set_entry_point(new_entry_point);
        core.hot.set_ram_entry_point(new_ram_entry_point);
        *offsets = new_offsets;
        *capacities = new_capacities;
        *count_offsets = new_count_offsets;
        *counts = new_counts;
        *arena = new_arena;

        core.hot.max_layer.store(
            new_index.inner.hot.max_layer.load(Ordering::SeqCst),
            Ordering::SeqCst,
        );

        let mmap_count = core
            .cold
            .mmap_index
            .read()
            .as_ref()
            .map(|m| m.header.node_count() as usize)
            .unwrap_or(0);
        let mut new_deleted = RoaringTreemap::new();
        for del_idx in deleted_nodes.iter() {
            if (del_idx as usize) < mmap_count {
                new_deleted.insert(del_idx);
            }
        }
        for del_idx in new_index.inner.cold.deleted_nodes.read().iter() {
            new_deleted.insert(del_idx);
        }

        *deleted_nodes = new_deleted;
        core.hot
            .deleted_count
            .store(deleted_nodes.len(), Ordering::SeqCst);
    }

    Ok(())
}
