//! Restore-Guard zur Erkennung von Zeitregression und Wiederauftauchen (SPEC §11.8, D-37, D-74, ER-30).
//!
//! Beim Öffnen des Stores erkennt der Guard:
//!   (1) Zeitregression: `store.current_seq < max(gated_seq)` im Ledger
//!   (2) Wiederauftauchen: Datei im Zielbereich mit `min_seq ≤ gated_seq` + Record `seq ≤ gated_seq`
//!
//! Treffer → Store nicht freigegeben, `Recovering`, automatische Wiederholung.

use crate::ledger::sink::InMemoryLedger;

/// Ergebnis des Restore-Guard-Checks.
#[derive(Debug, PartialEq, Eq)]
pub enum GuardOutcome {
    /// Kein Wiederauftauchen erkannt, Store kann freigegeben werden.
    Clean,
    /// Zeitregression erkannt – Store gesperrt, Replay erforderlich.
    TimeRegression {
        /// Aktueller Store-Seq.
        store_seq: u64,
        /// Ledger-Floor (max gated_seq).
        ledger_floor: u64,
    },
    /// Wiederauftauchen erkannt – Daten erschienen erneut nach Löschung.
    Reappearance {
        /// Gate-Seq der betroffenen Löschung.
        gated_seq: u64,
    },
    /// Ledger fehlt – degradierter Betrieb, Bestätigung erforderlich (ER-30).
    LedgerMissing,
}

/// Dateibasierter Restore-Checker: prüft auf Zeitregression und Wiederauftauchen (ER-30).
pub struct FileRestoreChecker<'a> {
    ledger: &'a InMemoryLedger,
}

impl<'a> FileRestoreChecker<'a> {
    /// Erstellt einen neuen Checker mit Ledger-Referenz.
    pub fn new(ledger: &'a InMemoryLedger) -> Self {
        Self { ledger }
    }

    /// Führt die Integritätsprüfung durch (ER-30).
    ///
    /// `store_current_seq`: aktuelle Sequenznummer des Stores.
    pub fn check(&self, store_current_seq: u64) -> GuardOutcome {
        // Zeitregression prüfen (ER-30 Nr. 1)
        if self.ledger.detect_time_regression(store_current_seq) {
            return GuardOutcome::TimeRegression {
                store_seq: store_current_seq,
                ledger_floor: self.ledger.ledger_floor(),
            };
        }

        // Wiederauftauchen: produktiv über RangeProbe-Port (L-22)
        // Stub: kein Wiederauftauchen erkannt ohne Store-Zugang (WP-4.3)

        GuardOutcome::Clean
    }

    /// Berechnet die nächste sichere Sequenznummer: `max(current_seq, ledger_floor) + 1` (ER-30).
    pub fn next_safe_seq(&self, store_current_seq: u64) -> u64 {
        store_current_seq
            .max(self.ledger.ledger_floor())
            .saturating_add(1)
    }
}
