// FILE-CONTEXT
// ZWECK: In-Memory-Overlay zur Identifikation logisch genullter mmap-residenter HNSW-Knotenindizes.
// HINWEIS: (b) Persistenz über WAL und (c) Verdrahtung in Lesepfade sind bewusst NICHT Teil dieses Arbeitspakets
// (Referenz auf künftige Arbeitspakete).

use parking_lot::RwLock;
use std::collections::HashSet;

/// In-Memory-Overlay: markiert mmap-residente HNSW-Knotenindizes als
/// logisch gelöscht. Lesepfade MÜSSEN (künftiges Arbeitspaket) vor jedem
/// Zugriff auf einen mmap-Vektor `is_zeroed` prüfen und bei `true` Nullen
/// statt der echten mmap-Bytes zurückgeben. Diese Struktur allein ändert
/// noch kein Leseverhalten.
pub struct MmapZeroOverlay {
    zeroed: RwLock<HashSet<u32>>,
}

impl MmapZeroOverlay {
    /// Erstellt eine neue Instanz von `MmapZeroOverlay`.
    pub fn new() -> Self {
        Self {
            zeroed: RwLock::new(HashSet::new()),
        }
    }

    /// Markiert einen mmap-residente Index als logisch genullt.
    pub fn mark_zeroed(&self, mmap_idx: u32) {
        self.zeroed.write().insert(mmap_idx);
    }

    /// Prüft, ob der angegebene mmap-Index logisch genullt ist.
    pub fn is_zeroed(&self, mmap_idx: u32) -> bool {
        self.zeroed.read().contains(&mmap_idx)
    }

    /// Entfernt die Markierung für den mmap-Index (z. B. für Slot-Wiederverwendung nach Rebuild).
    pub fn unmark(&self, mmap_idx: u32) {
        self.zeroed.write().remove(&mmap_idx);
    }

    /// Liefert die Anzahl aktuell als genullt markierter Indizes (Audit-Metrik).
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
    fn test_mmap_zero_overlay_mark_is_zeroed_unmark() {
        let overlay = MmapZeroOverlay::new();
        assert_eq!(overlay.zeroed_count(), 0);
        assert!(!overlay.is_zeroed(42));

        // Mark index
        overlay.mark_zeroed(42);
        assert!(overlay.is_zeroed(42));
        assert_eq!(overlay.zeroed_count(), 1);

        // Idempotent double mark
        overlay.mark_zeroed(42);
        assert!(overlay.is_zeroed(42));
        assert_eq!(overlay.zeroed_count(), 1);

        // Mark another index
        overlay.mark_zeroed(100);
        assert!(overlay.is_zeroed(100));
        assert_eq!(overlay.zeroed_count(), 2);

        // Unmark index
        overlay.unmark(42);
        assert!(!overlay.is_zeroed(42));
        assert!(overlay.is_zeroed(100));
        assert_eq!(overlay.zeroed_count(), 1);

        // Unmark index that was never marked (no panic)
        overlay.unmark(999);
        assert_eq!(overlay.zeroed_count(), 1);
    }

    #[test]
    fn test_mmap_zero_overlay_default() {
        let overlay = MmapZeroOverlay::default();
        assert_eq!(overlay.zeroed_count(), 0);
    }
}
