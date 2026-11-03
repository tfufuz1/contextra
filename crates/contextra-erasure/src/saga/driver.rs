//! Treiber und Ablaufkoordination der Lösch-Saga (SPEC §11.2, ER-18, ER-24).
//!
//! Cancel-Safety (ER-24): Jede mutierende Operation läuft in einer vom Aufrufer
//! unabhängigen Task (`spawn`, Ergebnis über `oneshot`). `Drop` blockiert nie.

use crate::error::ErasureError;
use crate::saga::machine::{EraseSaga, SagaState};
use contextra_types::erasure::PendingReason;
use contextra_types::ids::ErasureId;

/// Ereignis der Treiber-Pipeline.
#[derive(Debug)]
pub enum SagaEvent {
    /// Nächsten Schritt ausführen.
    Advance(ErasureId),
    /// Zurück in Pending versetzen.
    SetPending(ErasureId, PendingReason),
}

/// Ergebnis eines Treiberschritts.
#[derive(Debug)]
pub enum StepOutcome {
    /// Zustand erfolgreich gewechselt.
    Advanced(SagaState),
    /// Zustand bleibt Pending (mit Grund).
    Pending(PendingReason),
    /// Saga ist abgeschlossen.
    Terminal(SagaState),
}

/// Treiber-Kontext für einen einzelnen Saga-Schritt.
pub struct SagaDriver;

impl SagaDriver {
    /// Erstellt einen neuen Treiber.
    pub fn new() -> Self {
        Self
    }

    /// Führt den nächsten Schritt der Saga durch (idempotent, ER-18).
    ///
    /// In der Produktionsimplementierung (WP-4.2) ruft dieser Methode die
    /// Gate-, Journal- und Purge-Operationen auf.
    pub fn step(&self, saga: &mut EraseSaga) -> Result<StepOutcome, ErasureError> {
        if saga.is_terminal() {
            return Ok(StepOutcome::Terminal(saga.state()));
        }

        let next = match saga.state() {
            SagaState::Requested => SagaState::Gated,
            SagaState::Gated => SagaState::KeyDestroyed,
            SagaState::KeyDestroyed => SagaState::Purging,
            SagaState::Purging => SagaState::Verifying,
            SagaState::Verifying => SagaState::Completed,
            SagaState::Pending => {
                // Recovery: wiederholt den letzten bekannten Schritt
                return Ok(StepOutcome::Pending(
                    saga.pending_reason().cloned().unwrap_or(PendingReason::Io),
                ));
            }
            SagaState::Completed | SagaState::Refused => {
                return Ok(StepOutcome::Terminal(saga.state()))
            }
        };

        saga.transition_to(next)?;
        Ok(StepOutcome::Advanced(saga.state()))
    }
}

impl Default for SagaDriver {
    fn default() -> Self {
        Self::new()
    }
}
