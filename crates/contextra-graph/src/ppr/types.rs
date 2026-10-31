//! Basistypen, Kontextstrukturen und Vektorpuffer für PPR-Berechnungen.
//!
//! # Zweck
//! Definiert `PprContext` und `DeletedView` zur effizienten Wiederverwendung von Arbeitsvektoren.

use std::collections::HashSet;

#[derive(Debug, Clone)]
// ANCHOR[contextra-graph:SRC-PPR-TYPES-001]
pub struct DeletedView {
    deleted_nodes: HashSet<usize>,
}

impl DeletedView {
    pub(crate) fn from_nodes(deleted_nodes: HashSet<usize>) -> Self {
        Self { deleted_nodes }
    }

    #[cfg(test)]
    pub(crate) fn empty() -> Self {
        Self {
            deleted_nodes: HashSet::new(),
        }
    }

    #[inline]
// ANCHOR[contextra-graph:SRC-PPR-TYPES-002]
    pub fn contains(&self, idx: usize) -> bool {
        self.deleted_nodes.contains(&idx)
    }

    #[inline]
// ANCHOR[contextra-graph:SRC-PPR-TYPES-003]
    pub fn len(&self) -> usize {
        self.deleted_nodes.len()
    }

    #[inline]
// ANCHOR[contextra-graph:SRC-PPR-TYPES-004]
    pub fn is_empty(&self) -> bool {
        self.deleted_nodes.is_empty()
    }
}

#[derive(Debug, Clone, Default)]
// ANCHOR[contextra-graph:SRC-PPR-TYPES-005]
pub struct PprContext {
    pub(crate) valid_seeds: Vec<usize>,
    pub(crate) out_weight_sums: Vec<f32>,
    pub(crate) ranks: Vec<f32>,
    pub(crate) next_ranks: Vec<f32>,
}

impl PprContext {
// ANCHOR[contextra-graph:SRC-PPR-TYPES-006]
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn prepare(&mut self, n: usize) {
        self.valid_seeds.clear();

        if self.out_weight_sums.len() < n {
            self.out_weight_sums.resize(n, 0.0);
            self.ranks.resize(n, 0.0);
            self.next_ranks.resize(n, 0.0);
        }
        if let Some(slice) = self.out_weight_sums.get_mut(..n) {
            slice.fill(0.0);
        }
        if let Some(slice) = self.ranks.get_mut(..n) {
            slice.fill(0.0);
        }
        if let Some(slice) = self.next_ranks.get_mut(..n) {
            slice.fill(0.0);
        }
    }
}
