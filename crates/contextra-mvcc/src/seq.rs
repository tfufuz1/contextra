//! # Sequenzallokator (`SeqAllocator`)
//!
//! Verwaltet monoton steigende Sequenznummern für Transaktionen und Versionen.
//!
//! ### Invarianten und Garantien
//! - **I-20 / MV-06**: `next_seq = max(current_seq, ledger_floor) + 1`. Die Sequenz bleibt
//!   über Restores streng monoton und läuft niemals rückwärts.
//! - **MV-10**: Checked-Arithmetik, Überlaufbehandlung ohne Wrap und ohne Panics.
//! - **Concurrency**: Thread-sichere atomare Sequenzvergabe via CAS-Schleife (Compare-And-Swap).
//! - **Ring 0**: Keine Kenntnis über Festplattenlayouts, keine I/O-Operationen.

use crate::error::MvccError;
use std::sync::atomic::{AtomicU64, Ordering};

/// Atomarer Sequenzallokator mit Restore-Sicherheit.
///
/// Garantiert, dass generierte Sequenznummern sowohl strikt größer als die bisherige
/// `current_seq` als auch strikt größer als der konfigurierte `ledger_floor` sind.
#[derive(Debug)]
pub struct SeqAllocator {
    current_seq: AtomicU64,
    ledger_floor: AtomicU64,
}

impl SeqAllocator {
    /// Erzeugt einen neuen `SeqAllocator` mit initialen Sequenz- und Floor-Werten.
    #[inline]
    pub fn new(initial_seq: u64, ledger_floor: u64) -> Self {
        Self {
            current_seq: AtomicU64::new(initial_seq),
            ledger_floor: AtomicU64::new(ledger_floor),
        }
    }

    /// Liefert die zuletzt vergebene oder gesetzte `current_seq`.
    #[inline]
    pub fn current_seq(&self) -> u64 {
        self.current_seq.load(Ordering::Acquire)
    }

    /// Liefert den aktuellen `ledger_floor`.
    #[inline]
    pub fn ledger_floor(&self) -> u64 {
        self.ledger_floor.load(Ordering::Acquire)
    }

    /// Passt den `ledger_floor` an (z. B. nach Wiederherstellung oder Ledger-Scan).
    ///
    /// Nutzt `fetch_max`, damit der Floor niemals verkleinert werden kann.
    #[inline]
    pub fn set_ledger_floor(&self, floor: u64) {
        self.ledger_floor.fetch_max(floor, Ordering::Release);
    }

    /// Setzt nach einem Restore-Ereignis den Zustand neu.
    ///
    /// Setzt `current_seq` auf den Stand des restaurierten Snapshots und `ledger_floor`
    /// auf den Audit-Ledger-Wasserstand.
    pub fn reset_after_restore(&self, current: u64, floor: u64) {
        self.current_seq.store(current, Ordering::Release);
        self.ledger_floor.store(floor, Ordering::Release);
    }

    /// Berechnet und allokiert atomar die nächste Sequenznummer.
    ///
    /// ### Algorithmus (MV-06 / I-20):
    /// 1. Lade `current_seq` und `ledger_floor`.
    /// 2. Berechne `base = max(current_seq, ledger_floor)`.
    /// 3. Berechne `next = base.checked_add(1)`.
    /// 4. Bei Überlauf (`None`): Gebe `Err(MvccError::InvalidSequence(base))` zurück (kein Sentinel-Wert).
    /// 5. Atomarer CAS auf `current_seq`: Setze auf `next`. Bei Konflikt wiederhole ab Schritt 1.
    pub fn next_seq(&self) -> Result<u64, MvccError> {
        let mut curr = self.current_seq.load(Ordering::Acquire);
        loop {
            let floor = self.ledger_floor.load(Ordering::Acquire);
            let base = curr.max(floor);
            let next = base
                .checked_add(1)
                .ok_or(MvccError::InvalidSequence(base))?;

            match self.current_seq.compare_exchange_weak(
                curr,
                next,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return Ok(next),
                Err(actual) => curr = actual,
            }
        }
    }
}

impl Default for SeqAllocator {
    #[inline]
    fn default() -> Self {
        Self::new(0, 0)
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
    use super::*;

    #[test]
    fn test_seq_allocator_basic() {
        let alloc = SeqAllocator::new(0, 0);
        assert_eq!(alloc.current_seq(), 0);
        assert_eq!(alloc.ledger_floor(), 0);

        let s1 = alloc.next_seq().expect("valid next seq");
        assert_eq!(s1, 1);
        assert_eq!(alloc.current_seq(), 1);

        let s2 = alloc.next_seq().expect("valid next seq");
        assert_eq!(s2, 2);
    }

    #[test]
    fn test_seq_allocator_ledger_floor_dominates() {
        let alloc = SeqAllocator::new(5, 50);
        let s = alloc.next_seq().expect("valid next seq");
        assert_eq!(s, 51);
        assert_eq!(alloc.current_seq(), 51);
    }

    #[test]
    fn test_seq_allocator_overflow_fail_closed() {
        let alloc = SeqAllocator::new(u64::MAX, 0);
        assert!(alloc.next_seq().is_err());

        let alloc2 = SeqAllocator::new(0, u64::MAX);
        assert!(alloc2.next_seq().is_err());
    }

    #[test]
    fn test_seq_allocator_set_ledger_floor_monotonic() {
        let alloc = SeqAllocator::new(10, 20);
        alloc.set_ledger_floor(15);
        assert_eq!(alloc.ledger_floor(), 20);

        alloc.set_ledger_floor(30);
        assert_eq!(alloc.ledger_floor(), 30);
    }
}
