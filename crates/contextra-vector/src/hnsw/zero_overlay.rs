// FILE-CONTEXT
// STAND: 2026-10-09
// ZWECK: In-Memory-Overlay zur Identifikation logisch gelöschter/genullter mmap-residenter HNSW-Knotenindizes.
// INVARIANTEN: Zero-Panic; thread-sichere lesende/schreibende Zugriffe auf das Overlay.
// NICHT-OFFENSICHTLICH: dieses Overlay ändert ohne Verdrahtung in Lesepfade noch kein Verhalten;
// WAL-Persistenz und Lesepfad-Verdrahtung folgen in separaten Arbeitspaketen.

//! In-Memory-Overlay: markiert mmap-residente HNSW-Knotenindizes als logisch gelöscht.
//!
//! mmap-residente HNSW-Knoten werden beim Löschen NICHT direkt in der mmap-Datei genullt,
//! da die mmap-Datei während laufenden Betriebs (wegen paralleler Leser) nicht direkt
//! beschrieben werden darf.
//!
//! Lesepfade MÜSSEN (künftiges Arbeitspaket) vor jedem Zugriff auf einen mmap-Vektor
//! `is_zeroed` prüfen und bei `true` Nullen statt der echten mmap-Bytes zurückgeben.
//! Diese Struktur allein ändert noch kein Leseverhalten.
//!
//! **Hinweis zu künftigen Arbeitspaketen:**
//! - Punkt (b) Persistenz über WAL (Write-Ahead Log) ist bewusst **nicht** Teil dieses
//!   Arbeitspakets und folgt in einem späteren Arbeitspaket.
//! - Punkt (c) Verdrahtung in Lesepfade (core_search.rs, core_rebuild.rs, vector_index_impl.rs)
//!   ist bewusst **nicht** Teil dieses Arbeitspakets und folgt in einem späteren Arbeitspaket.

use std::collections::HashSet;
use parking_lot::RwLock;

/// In-Memory-Overlay: markiert mmap-residente HNSW-Knotenindizes als
/// logisch gelöscht.
pub struct MmapZeroOverlay {
    zeroed: RwLock<HashSet<u32>>,
}

impl MmapZeroOverlay {
    /// Erstellt ein neues leeres `MmapZeroOverlay`.
    pub fn new() -> Self {
        Self {
            zeroed: RwLock::new(HashSet::new()),
        }
    }

    /// Markiert den angegebenen mmap-Knotenindex als gelöscht / genullt.
    /// Die Operation ist idempotent.
    pub fn mark_zeroed(&self, mmap_idx: u32) {
        self.zeroed.write().insert(mmap_idx);
    }

    /// Prüft, ob der angegebene mmap-Knotenindex als gelöscht / genullt markiert ist.
    pub fn is_zeroed(&self, mmap_idx: u32) -> bool {
        self.zeroed.read().contains(&mmap_idx)
    }

    /// Entfernt die Genullt-Markierung für einen mmap-Knotenindex
    /// (z. B. für Slot-Wiederverwendung nach Rebuild).
    /// Wenn der Index nie markiert war, geschieht nichts (kein Panic).
    pub fn unmark(&self, mmap_idx: u32) {
        self.zeroed.write().remove(&mmap_idx);
    }

    /// Liefert die Anzahl der aktuell markierten mmap-Indizes (Audit-Metrik).
    pub fn zeroed_count(&self) -> usize {
        self.zeroed.read().len()
    }
}

impl Default for MmapZeroOverlay {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mark_is_zeroed_unmark_count() {
        let overlay = MmapZeroOverlay::new();
        assert_eq!(overlay.zeroed_count(), 0);
        assert!(!overlay.is_zeroed(42));

        overlay.mark_zeroed(42);
        assert!(overlay.is_zeroed(42));
        assert_eq!(overlay.zeroed_count(), 1);
        assert!(!overlay.is_zeroed(100));

        overlay.unmark(42);
        assert!(!overlay.is_zeroed(42));
        assert_eq!(overlay.zeroed_count(), 0);
    }

    #[test]
    fn test_idempotent_mark_zeroed() {
        let overlay = MmapZeroOverlay::new();

        overlay.mark_zeroed(15);
        overlay.mark_zeroed(15);
        overlay.mark_zeroed(15);

        assert!(overlay.is_zeroed(15));
        assert_eq!(overlay.zeroed_count(), 1);
    }

    #[test]
    fn test_unmark_unmarked_index_no_panic() {
        let overlay = MmapZeroOverlay::new();

        // unmark auf nie markiertem Index darf nicht paniken
        overlay.unmark(999);
        assert!(!overlay.is_zeroed(999));
        assert_eq!(overlay.zeroed_count(), 0);
    }

    #[test]
    fn test_default_impl() {
        let overlay = MmapZeroOverlay::default();
        assert_eq!(overlay.zeroed_count(), 0);
    }
}
