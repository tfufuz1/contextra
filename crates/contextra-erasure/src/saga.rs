//! Saga-Zustandsmaschine (SPEC §11.2, ER-15..ER-24).
//!
//! Zustände: `Requested ─▶ Gated ─▶ KeyDestroyed ─▶ Purging ─▶ Verifying ─▶ Completed`
//! Fehlerzustände: `Pending(reason)`, `Refused(reason)`.

use crate::error::ErasureError;
use contextra_ports::erasure::EraseTarget;
use serde::{Deserialize, Serialize};

/// Zustände der Lösch-Saga (SPEC §11.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SagaState {
    /// Löschung angefragt
    Requested,
    /// Tor geschlossen, Lesepfade blockiert
    Gated,
    /// Schlüsselmaterial im Tresor vernichtet
    KeyDestroyed,
    /// Physischer Purge auf den Schichten läuft
    Purging,
    /// Residuenprüfung und Zeugenausstellung
    Verifying,
    /// Vollständig abgeschlossen und quittiert
    Completed,
    /// Ausstehend (z. B. SLA, I/O-Fehler, DiskReserve)
    Pending,
    /// Endgültig abgewiesen (z. B. gesetzliche Aufbewahrungspflicht)
    Refused,
}

/// Anfrage zur Löschung.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EraseRequest {
    /// Ziel der Löschung
    pub target: EraseTarget,
    /// Eindeutige Lösch-ID
    pub erasure_id: [u8; 16],
}

/// Zustandsmaschine zur Ausführung und Verfolgung einer Lösch-Saga.
#[derive(Debug, Clone)]
pub struct EraseSaga {
    state: SagaState,
}

impl Default for EraseSaga {
    fn default() -> Self {
        Self::new()
    }
}

impl EraseSaga {
    /// Erstellt eine neue Saga im Zustand `Requested`.
    pub fn new() -> Self {
        Self {
            state: SagaState::Requested,
        }
    }

    /// Aktueller Zustand.
    pub fn state(&self) -> SagaState {
        self.state
    }

    /// Idempotenter Zustandsübergang (ER-18).
    pub fn transition_to(&mut self, next: SagaState) -> Result<(), ErasureError> {
        if self.state == next {
            return Ok(());
        }

        let valid = matches!(
            (self.state, next),
            (SagaState::Requested, SagaState::Gated)
                | (SagaState::Requested, SagaState::Refused)
                | (SagaState::Gated, SagaState::KeyDestroyed)
                | (SagaState::KeyDestroyed, SagaState::Purging)
                | (SagaState::Purging, SagaState::Verifying)
                | (SagaState::Verifying, SagaState::Completed)
                | (_, SagaState::Pending)
        );

        if valid {
            self.state = next;
            Ok(())
        } else {
            Err(ErasureError::Internal(format!(
                "Ungültiger Zustandsübergang von {:?} nach {:?}",
                self.state, next
            )))
        }
    }
}
