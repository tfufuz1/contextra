//! Modul `types` im Ring 0 (Graph-Subsystem) von Contextra.
//!
//! Stellt Kernkomponenten für Graphtraversierung, CSR-Repräsentationen und
//! Analysemethoden bereit. Wird von Integrationsschichten und Algorithmenmodulen verwendet.

use std::collections::HashSet;

#[derive(Debug, Clone)]
// ANCHOR[contextra-graph:DELETEDVIEW-0305]
/// Struktur `DeletedView` für die Graphverarbeitung.
///
/// # Zweck
/// Kapselt Funktionalität im Ring 0 (Graph) ohne externe I/O-Seiteneffekte.
///
/// # Ablauf
/// 1. Parameter und Vorbedingungen im MVCC-/CSR-Graphkontext validieren.
/// 2. Speichereffiziente Berechnung oder Datenzugriff durchführen.
pub struct DeletedView {
    deleted_nodes: HashSet<usize>,
}

impl DeletedView {
    // ANCHOR[contextra-graph:FROM_NODES-0306]
    /// Funktion `from_nodes` für die Graphverarbeitung.
    ///
    /// # Zweck
    /// Kapselt Funktionalität im Ring 0 (Graph) ohne externe I/O-Seiteneffekte.
    ///
    /// # Ablauf
    /// 1. Parameter und Vorbedingungen im MVCC-/CSR-Graphkontext validieren.
    /// 2. Speichereffiziente Berechnung oder Datenzugriff durchführen.
    pub(crate) fn from_nodes(deleted_nodes: HashSet<usize>) -> Self {
        Self { deleted_nodes }
    }

    #[cfg(test)]
    // ANCHOR[contextra-graph:EMPTY-0307]
    /// Funktion `empty` für die Graphverarbeitung.
    ///
    /// # Zweck
    /// Kapselt Funktionalität im Ring 0 (Graph) ohne externe I/O-Seiteneffekte.
    ///
    /// # Ablauf
    /// 1. Parameter und Vorbedingungen im MVCC-/CSR-Graphkontext validieren.
    /// 2. Speichereffiziente Berechnung oder Datenzugriff durchführen.
    pub(crate) fn empty() -> Self {
        Self {
            deleted_nodes: HashSet::new(),
        }
    }

    #[inline]
    // ANCHOR[contextra-graph:CONTAINS-0308]
    /// Funktion `contains` für die Graphverarbeitung.
    ///
    /// # Zweck
    /// Kapselt Funktionalität im Ring 0 (Graph) ohne externe I/O-Seiteneffekte.
    ///
    /// # Ablauf
    /// 1. Parameter und Vorbedingungen im MVCC-/CSR-Graphkontext validieren.
    /// 2. Speichereffiziente Berechnung oder Datenzugriff durchführen.
    pub fn contains(&self, idx: usize) -> bool {
        self.deleted_nodes.contains(&idx)
    }

    #[inline]
    // ANCHOR[contextra-graph:LEN-0309]
    /// Funktion `len` für die Graphverarbeitung.
    ///
    /// # Zweck
    /// Kapselt Funktionalität im Ring 0 (Graph) ohne externe I/O-Seiteneffekte.
    ///
    /// # Ablauf
    /// 1. Parameter und Vorbedingungen im MVCC-/CSR-Graphkontext validieren.
    /// 2. Speichereffiziente Berechnung oder Datenzugriff durchführen.
    pub fn len(&self) -> usize {
        self.deleted_nodes.len()
    }

    #[inline]
    // ANCHOR[contextra-graph:IS_EMPTY-0310]
    /// Funktion `is_empty` für die Graphverarbeitung.
    ///
    /// # Zweck
    /// Kapselt Funktionalität im Ring 0 (Graph) ohne externe I/O-Seiteneffekte.
    ///
    /// # Ablauf
    /// 1. Parameter und Vorbedingungen im MVCC-/CSR-Graphkontext validieren.
    /// 2. Speichereffiziente Berechnung oder Datenzugriff durchführen.
    pub fn is_empty(&self) -> bool {
        self.deleted_nodes.is_empty()
    }
}

#[derive(Debug, Clone, Default)]
// ANCHOR[contextra-graph:PPRCONTEXT-0311]
/// Struktur `PprContext` für die Graphverarbeitung.
///
/// # Zweck
/// Kapselt Funktionalität im Ring 0 (Graph) ohne externe I/O-Seiteneffekte.
///
/// # Ablauf
/// 1. Parameter und Vorbedingungen im MVCC-/CSR-Graphkontext validieren.
/// 2. Speichereffiziente Berechnung oder Datenzugriff durchführen.
pub struct PprContext {
    pub(crate) valid_seeds: Vec<usize>,
    pub(crate) out_weight_sums: Vec<f32>,
    pub(crate) ranks: Vec<f32>,
    pub(crate) next_ranks: Vec<f32>,
}

impl PprContext {
    // ANCHOR[contextra-graph:NEW-0312]
    /// Funktion `new` für die Graphverarbeitung.
    ///
    /// # Zweck
    /// Kapselt Funktionalität im Ring 0 (Graph) ohne externe I/O-Seiteneffekte.
    ///
    /// # Ablauf
    /// 1. Parameter und Vorbedingungen im MVCC-/CSR-Graphkontext validieren.
    /// 2. Speichereffiziente Berechnung oder Datenzugriff durchführen.
    pub fn new() -> Self {
        Self::default()
    }

    // ANCHOR[contextra-graph:PREPARE-0313]
    /// Funktion `prepare` für die Graphverarbeitung.
    ///
    /// # Zweck
    /// Kapselt Funktionalität im Ring 0 (Graph) ohne externe I/O-Seiteneffekte.
    ///
    /// # Ablauf
    /// 1. Parameter und Vorbedingungen im MVCC-/CSR-Graphkontext validieren.
    /// 2. Speichereffiziente Berechnung oder Datenzugriff durchführen.
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
