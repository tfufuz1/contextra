//! Zustände eines Datums nach SPEC §5.2 mit der normativen Übergangstabelle.

/// Grund einer wiederholbaren Verzögerung (SPEC §5.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum PendingReason {
    /// Plattenreserve unter 1 GB.
    DiskReserveLow,
    /// Kompaktion läuft.
    CompactionRunning,
    /// Transienter I/O-Fehler, Wiederholung mit Backoff.
    IoBackoff,
}

/// Saga-Zustand (SPEC §5.2). `Live` ist der Zustand ohne laufende Löschung und hat keinen Eintrag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum SagaState {
    /// Auftrag erfasst, Intent im Journal (fsync #1).
    Requested,
    /// Gate aktiv, Schlüssel noch vorhanden (fsync #2). Irreversibel (INV-10).
    Gated,
    /// Schlüssel vernichtet und verifiziert (fsync #3). Ab hier kryptographisch unlesbar.
    KeyRevoked,
    /// Physische Bereinigung läuft.
    Purging,
    /// Messung läuft.
    Verifying,
    /// Abgeschlossen, Bericht signiert, Ledger geschrieben (terminal, INV-9).
    Completed,
    /// Art.-18-Einschränkung aktiv: Lesen nur mit Audit.
    Restricted,
    /// Einschränkung aufgehoben (terminal, INV-9).
    Lifted,
    /// Blockiert mit Grund; wird vom Hintergrundläufer fortgesetzt.
    Pending(PendingReason),
    /// Wegen aktiver Aufbewahrungspflicht verweigert (terminal, INV-9).
    Refused,
}

impl SagaState {
    /// Terminale Zustände (INV-9).
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Lifted | Self::Refused)
    }

    /// Ob Lesezugriffe in diesem Zustand blockiert sind (SPEC §5.2, Tabelle „Lesen“).
    pub const fn blocks_reads(self) -> bool {
        matches!(
            self,
            Self::Gated
                | Self::KeyRevoked
                | Self::Purging
                | Self::Verifying
                | Self::Completed
                | Self::Pending(_)
        )
    }

    /// Normative Übergangstabelle (SPEC §5.2). Terminale Zustände haben keine Nachfolger.
    pub const fn can_transition_to(self, next: SagaState) -> bool {
        use SagaState::*;
        if self.is_terminal() {
            return false;
        }
        match (self, next) {
            (Requested, Gated | Restricted | Refused) => true,
            (Restricted, Gated | Lifted) => true,
            (Gated, KeyRevoked) => true,
            (KeyRevoked, Purging) => true,
            (Purging, Verifying) => true,
            (Verifying, Completed) => true,
            // Wiederholbare Verzögerung: aus jedem nicht-terminalen Zustand, nie aus Pending in Pending.
            (from, Pending(_)) => !matches!(from, Pending(_)),
            // Rückkehr aus Pending in den gespeicherten Zustand (Resume-Ziel wird vom Aufrufer geprüft).
            (Pending(_), Requested | Restricted | Gated | KeyRevoked | Purging | Verifying) => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use SagaState::*;

    #[test]
    fn terminale_zustaende_haben_keine_nachfolger() {
        let all = [
            Requested,
            Gated,
            KeyRevoked,
            Purging,
            Verifying,
            Completed,
            Restricted,
            Lifted,
            Pending(PendingReason::IoBackoff),
            Refused,
        ];
        for terminal in [Completed, Lifted, Refused] {
            assert!(terminal.is_terminal());
            for next in all {
                assert!(
                    !terminal.can_transition_to(next),
                    "{terminal:?} -> {next:?} verboten"
                );
            }
        }
    }

    #[test]
    fn gate_ist_irreversibel() {
        // INV-10: Gated darf nie zurück zu Restricted oder Requested.
        assert!(!Gated.can_transition_to(Restricted));
        assert!(!Gated.can_transition_to(Requested));
        assert!(!KeyRevoked.can_transition_to(Gated));
    }

    #[test]
    fn normaler_pfad_ist_erlaubt() {
        assert!(Requested.can_transition_to(Gated));
        assert!(Gated.can_transition_to(KeyRevoked));
        assert!(KeyRevoked.can_transition_to(Purging));
        assert!(Purging.can_transition_to(Verifying));
        assert!(Verifying.can_transition_to(Completed));
    }

    #[test]
    fn restrict_pfad() {
        assert!(Requested.can_transition_to(Restricted));
        assert!(Restricted.can_transition_to(Gated));
        assert!(Restricted.can_transition_to(Lifted));
        assert!(!Restricted.can_transition_to(Purging));
    }

    #[test]
    fn pending_aus_jedem_nicht_terminalen_zustand() {
        for from in [Requested, Gated, KeyRevoked, Purging, Verifying, Restricted] {
            assert!(from.can_transition_to(Pending(PendingReason::DiskReserveLow)));
        }
        assert!(
            !Pending(PendingReason::IoBackoff).can_transition_to(Pending(PendingReason::IoBackoff))
        );
    }

    #[test]
    fn lesesperre_nach_tabelle() {
        assert!(!Requested.blocks_reads());
        assert!(!Restricted.blocks_reads());
        assert!(Gated.blocks_reads());
        assert!(KeyRevoked.blocks_reads());
        assert!(Completed.blocks_reads());
        assert!(!Lifted.blocks_reads());
    }
}
