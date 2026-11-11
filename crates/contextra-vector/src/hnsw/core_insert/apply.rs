// FILE-CONTEXT
// ZWECK: Übernahme vorbereiteter Einfügungen (PreparedInsert) in Graph-Arena und HotState.
// INVARIANTEN: Zero-Panic; Entry-Point Aktualisierung; Transaktionsatomarität.
// HOTSPOTS: apply_insert, do_insert
// STAND: 2026-11-12

//! Übernahme (Apply) vorbereiteter Knoten in den HNSW-Index.

use std::sync::atomic::Ordering;

use crate::error::Result;
use crate::types::DocId;

use super::super::batch::PreparedInsert;
use super::super::types::{HnswIndexCore, HnswNode};

impl HnswIndexCore {
    // ANCHOR[contextra-vector:HNSW-APPLY-INSERT]
    pub fn apply_insert(&self, prepared: PreparedInsert, tx_id: u64) {
        let node = HnswNode {
            doc_id: prepared.doc_id,
            vector: prepared.vector_data,
            max_layer: prepared.new_layer,
            committed_tx: tx_id,
        };

        let m = self.cold.config.m;
        self.hot
            .arena
            .allocate_node(prepared.new_layer, m, &prepared.final_connections)
            .ok();

        for backlink in prepared.neighbor_backlinks {
            self.hot
                .arena
                .update_backlink(
                    backlink.neighbor_ram_idx,
                    backlink.layer,
                    m,
                    &backlink.updated_connections,
                )
                .ok();
        }

        self.hot.nodes.write().push(node);
        self.hot
            .doc_to_node
            .write()
            .insert(prepared.doc_id.inner(), prepared.new_idx);

        let current_ep = self.hot.get_entry_point();
        if prepared.should_update_entry_point || current_ep.is_none() {
            self.hot.set_entry_point(Some(prepared.new_idx));
            let current_max = self.hot.max_layer.load(Ordering::Acquire) as usize;
            if prepared.new_layer > current_max || current_ep.is_none() {
                self.hot
                    .max_layer
                    .store(prepared.new_layer as u64, Ordering::Release);
            }
        }

        if prepared.should_update_ram_entry_point || self.hot.get_ram_entry_point().is_none() {
            self.hot.set_ram_entry_point(Some(prepared.new_idx));
        }
    }

    pub(crate) fn do_insert(&self, id: DocId, vector: &[f32], tx_id: u64) -> Result<()> {
        let prepared = self.compute_insert(id, vector)?;
        self.apply_insert(prepared, tx_id);
        Ok(())
    }
}
