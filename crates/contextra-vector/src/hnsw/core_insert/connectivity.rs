// FILE-CONTEXT
// ZWECK: Konnektivitäts-Prüfung und Score-Berechnung (INV-VEC-CONNECTIVITY-1).
// INVARIANTEN: Zero-Panic; BFS auf Layer 0; Fail-Closed bei isolierten Knoten.
// HOTSPOTS: connectivity_score, check_connectivity
// STAND: 2026-11-12

//! Konnektivitäts- und Graph-Integritätsprüfungen für HNSW.

use std::sync::atomic::Ordering;

use crate::error::Result;

use super::super::types::HnswIndexCore;

impl HnswIndexCore {
    pub fn connectivity_score(&self) -> f64 {
        let deleted = self.hot.deleted_count.load(Ordering::SeqCst);
        let mmap_count = self
            .cold
            .mmap_index
            .read()
            .as_ref()
            .map(|m| m.header.node_count() as usize)
            .unwrap_or(0);
        let total = mmap_count + self.hot.nodes.read().len();
        if total == 0 {
            return 1.0;
        }
        (1.0_f64 - deleted as f64 / total as f64).max(0.0_f64)
    }

    /// Prüft die Graph-Konnektivität (`INV-VEC-CONNECTIVITY-1`):
    /// Verifiziert mittels BFS auf Layer 0, dass alle aktiven (nicht gelöschten) Knoten
    /// vom aktuellen Entry-Point aus erreichbar sind.
    ///
    /// # Errors
    /// Gibt [`VectorError::Conflict`][crate::error::VectorError::Conflict]
    /// mit Fehlermeldung `"connectivity"` zurück, falls der Graph isolierte aktive Knoten
    /// aufweist oder kein Einstiegspunkt für vorhandene Knoten existiert.
    // ANCHOR[contextra-vector:HNSW-CHECK-CONNECTIVITY]
    pub fn check_connectivity(&self) -> crate::error::Result<()> {
        let deleted = self.cold.deleted_nodes.read();
        let doc_map = self.hot.doc_to_node.read();

        let active_nodes: ahash::AHashSet<usize> = doc_map
            .values()
            .copied()
            .filter(|&idx| !deleted.contains(idx as u64))
            .collect();

        if active_nodes.is_empty() {
            return Ok(());
        }

        let ep = match self.hot.get_entry_point() {
            Some(ep) if !deleted.contains(ep as u64) => ep,
            _ => {
                return Err(crate::error::VectorError::Conflict(
                    "connectivity: no valid entry point found for active nodes".to_string(),
                ));
            }
        };

        let mmap_count = self
            .cold
            .mmap_index
            .read()
            .as_ref()
            .map(|m| m.header.node_count() as usize)
            .unwrap_or(0);
        let m = self.cold.config.m;

        let mut reachable = ahash::AHashSet::with_capacity(active_nodes.len());
        let mut queue = std::collections::VecDeque::new();

        reachable.insert(ep);
        queue.push_back(ep);

        while let Some(current) = queue.pop_front() {
            let conns = if current < mmap_count {
                let mmap_guard = self.cold.mmap_index.read();
                if let Some(mmap) = mmap_guard.as_ref() {
                    if let Ok(rec) = mmap.get_node_record(current) {
                        mmap.get_connections(&rec, 0).unwrap_or_default()
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                }
            } else {
                let ram_idx = current - mmap_count;
                self.hot.arena.get_ram_node_connections(ram_idx, 0, m)
            };

            for n in conns {
                let n_usize = n as usize;
                if reachable.insert(n_usize) {
                    queue.push_back(n_usize);
                }
            }
        }

        for &node_idx in &active_nodes {
            if !reachable.contains(&node_idx) {
                return Err(crate::error::VectorError::Conflict(
                    "connectivity: isolated active node unreachable from entry point".to_string(),
                ));
            }
        }

        Ok(())
    }
}
