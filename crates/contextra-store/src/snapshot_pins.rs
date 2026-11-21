//! # Snapshot-Pins (Ring 1)
//!
//! Der Store kennt nur eine Frage der Versionierung: *Welche Sequenznummer darf die
//! Kompaktion frühestens verwerfen?* Katalog, Sichtbarkeit und Snapshot-Verwaltung
//! gehören `contextra-version` (SPEC 3.4, ADR-32). `SnapshotPins` ist der minimale
//! Port-Ersatz: ein Referenzzähler je gepinnter Sequenznummer mit lock-frei lesbarem
//! Minimum.

use contextra_ids::TOMBSTONE_BIT;
use parking_lot::Mutex;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};

/// Referenzgezählte Menge gepinnter Sequenznummern.
#[derive(Debug)]
pub struct SnapshotPins {
    pins: Mutex<BTreeMap<u64, u32>>,
    min_pinned: AtomicU64,
}

impl Default for SnapshotPins {
    fn default() -> Self {
        Self::new()
    }
}

impl SnapshotPins {
    /// Erzeugt eine leere Pin-Menge (`min_active_seqno() == u64::MAX`).
    #[must_use]
    pub fn new() -> Self {
        Self {
            pins: Mutex::new(BTreeMap::new()),
            min_pinned: AtomicU64::new(u64::MAX),
        }
    }

    /// Pinnt `seq_no`. Jeder `pin` braucht genau einen [`SnapshotPins::unpin`].
    pub fn pin(&self, seq_no: u64) {
        let seq = seq_no & !TOMBSTONE_BIT;
        let mut pins = self.pins.lock();
        let count = pins.entry(seq).or_insert(0);
        *count = count.saturating_add(1);
        self.publish_min(&pins);
    }

    /// Gibt einen Pin frei. Ein `unpin` ohne passenden `pin` ist wirkungslos.
    pub fn unpin(&self, seq_no: u64) {
        let seq = seq_no & !TOMBSTONE_BIT;
        let mut pins = self.pins.lock();
        if let Some(count) = pins.get_mut(&seq) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                pins.remove(&seq);
            }
        }
        self.publish_min(&pins);
    }

    /// Kleinste gepinnte Sequenznummer, `u64::MAX` wenn nichts gepinnt ist.
    #[must_use]
    #[inline]
    pub fn min_active_seqno(&self) -> u64 {
        self.min_pinned.load(Ordering::Acquire)
    }

    fn publish_min(&self, pins: &BTreeMap<u64, u32>) {
        let min = pins.keys().next().copied().unwrap_or(u64::MAX);
        self.min_pinned.store(min, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_is_max() {
        assert_eq!(SnapshotPins::new().min_active_seqno(), u64::MAX);
    }

    #[test]
    fn min_follows_refcounts() {
        let p = SnapshotPins::new();
        p.pin(10);
        p.pin(5);
        p.pin(5);
        assert_eq!(p.min_active_seqno(), 5);
        p.unpin(5);
        assert_eq!(p.min_active_seqno(), 5, "zweiter Pin hält die Marke");
        p.unpin(5);
        assert_eq!(p.min_active_seqno(), 10);
        p.unpin(10);
        assert_eq!(p.min_active_seqno(), u64::MAX);
    }

    #[test]
    fn stray_unpin_is_harmless() {
        let p = SnapshotPins::new();
        p.unpin(7);
        assert_eq!(p.min_active_seqno(), u64::MAX);
    }

    #[test]
    fn tombstone_bit_is_masked() {
        let p = SnapshotPins::new();
        p.pin(3 | TOMBSTONE_BIT);
        assert_eq!(p.min_active_seqno(), 3);
    }
}
