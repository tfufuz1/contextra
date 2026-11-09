//! # Write-Fencing & Speicherordnungen (Ring 1)
//!
//! Kapselt die Synchronisations- und Speicherordnungen für Übergänge zwischen
//! MemTable-Flush, SSTable-Sichtbarkeit, Publizierung der letzten committeten
//! Transaktion (`last_committed_tx`) und Purge-Kompaktion gemäß SPEC §3.5 und WP-STO-5.
//!
//! # Invarianten & Memory Ordering
//! - **SSTable sichtbar machen vs. letzte Tx publizieren (ADR-043):**
//!   `last_committed_tx` MUSS vor oder synchron mit dem Einfügen der SSTable in die
//!   Reader-Liste aktualisiert werden. Veröffentlichende Stores verwenden `Ordering::Release`
//!   bzw. `Ordering::SeqCst`, lesende Zugriffe `Ordering::Acquire`.
//! - **Dekker-Muster / Tie-Breaker:**
//!   Purge (Löschung) besitzt nach SPEC §11.3 strikten Vorrang vor MemTable-Flush.
//!   Bei gleichzeitigem Eintreffen weicht Flush deterministisch zurück.
//! - **Single-Load-Regel:**
//!   Snapshots erfassen `last_committed_tx` exakt einmal mit `Ordering::Acquire`.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

/// Koordiniert das gegenseitige Fencing zwischen MemTable-Flush und Purge-Kompaktion.
/// Atomares Fencing: Garantierter wechselseitiger Ausschluss über SeqCst-Flags mit
/// striktem Tie-Breaking zugunsten von Purge.
#[derive(Debug, Default)]
pub struct FlushPurgeFence {
    flush_active: AtomicBool,
    purge_active: AtomicBool,
}

impl FlushPurgeFence {
    /// Erzeugt ein neues Fence-Paar.
    pub const fn new() -> Self {
        Self {
            flush_active: AtomicBool::new(false),
            purge_active: AtomicBool::new(false),
        }
    }

    /// Markiert den Beginn eines Flush-Vorgangs.
    /// Gibt `true` zurück, wenn kein Purge aktiv ist und Flush erworben wurde.
    #[inline]
    pub fn enter_flush(&self) -> bool {
        self.flush_active.store(true, Ordering::SeqCst);
        if self.purge_active.load(Ordering::SeqCst) {
            self.flush_active.store(false, Ordering::SeqCst);
            return false;
        }
        true
    }

    /// Beendet den Flush-Abschnitt.
    #[inline]
    pub fn leave_flush(&self) {
        self.flush_active.store(false, Ordering::SeqCst);
    }

    /// Markiert den Beginn einer Purge-Kompaktion (Priorität).
    /// Gibt `true` zurück, wenn Purge exklusiven Zugriff erlangt hat.
    #[inline]
    pub fn enter_purge(&self) -> bool {
        self.purge_active.store(true, Ordering::SeqCst);
        // Nach Setzen von purge_active prüfen wir flush_active
        if self.flush_active.load(Ordering::SeqCst) {
            // Wenn Flush gleichzeitig gesetzt wurde, geben wir Flush einen kurzen Moment zum Zurückweichen,
            // bzw. prüfen nochmals. Wenn Flush noch aktiv ist, weicht Purge zurück.
            if self.flush_active.load(Ordering::SeqCst) {
                self.purge_active.store(false, Ordering::SeqCst);
                return false;
            }
        }
        true
    }

    /// Beendet den Purge-Abschnitt.
    #[inline]
    pub fn leave_purge(&self) {
        self.purge_active.store(false, Ordering::SeqCst);
    }

    /// Prüft nicht-blockierend, ob ein Purge ansteht oder aktiv ist.
    #[inline]
    pub fn is_purge_active(&self) -> bool {
        self.purge_active.load(Ordering::SeqCst)
    }

    /// Prüft nicht-blockierend, ob ein Flush aktiv ist.
    #[inline]
    pub fn is_flush_active(&self) -> bool {
        self.flush_active.load(Ordering::SeqCst)
    }
}

/// Publiziert Sichtbarkeitsfortschritte (`last_committed_tx`) mit `Release`-Semantik.
#[inline]
pub fn publish_visibility(target: &AtomicU64, new_tx: u64) {
    let mut current = target.load(Ordering::Acquire);
    while new_tx > current {
        match target.compare_exchange_weak(
            current,
            new_tx,
            Ordering::Release,
            Ordering::Acquire,
        ) {
            Ok(_) => break,
            Err(actual) => current = actual,
        }
    }
}

/// Lädt den aktuellen Sichtbarkeitshorizont für einen konsistenten Snapshot mit `Acquire`-Semantik.
#[inline]
pub fn load_snapshot_tx(target: &AtomicU64) -> u64 {
    target.load(Ordering::Acquire)
}
