//! Treiber und Ablaufkoordination der Lösch-Saga (SPEC §11.2, ER-18, ER-24).
//!
//! Cancel-Safety (ER-24): Jede mutierende Operation läuft in einer vom Aufrufer
//! unabhängigen Task (`spawn`, Ergebnis über `oneshot`). `Drop` blockiert nie.

use crate::error::ErasureError;
use crate::saga::machine::{EraseSaga, SagaState};
use crate::types_local::ErasureId;
use crate::types_local::PendingReason;
use std::sync::Arc;

/// Ereignis der Treiber-Pipeline (SPEC §11.2).
#[derive(Debug, Clone)]
pub enum SagaEvent {
    /// Nächsten Schritt ausführen.
    Advance(ErasureId),
    /// Zurück in Pending versetzen mit Grund.
    SetPending(ErasureId, PendingReason),
    /// Saga abschließen.
    Complete(ErasureId),
}

/// Ergebnis eines Treiberschritts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepOutcome {
    /// Zustand erfolgreich gewechselt.
    Advanced(SagaState),
    /// Zustand bleibt Pending (mit Grund).
    Pending(PendingReason),
    /// Saga ist abgeschlossen oder verweigert.
    Terminal(SagaState),
}

/// Treiber-Kontext für einen einzelnen Saga-Schritt.
#[derive(Default)]
pub struct SagaDriver {
    core: Option<Arc<super::core::ErasureCore>>,
}

impl SagaDriver {
    /// Erstellt einen neuen Treiber.
    pub fn new() -> Self {
        Self { core: None }
    }

    /// Erstellt einen Treiber mit Anbindung an den `ErasureCore`.
    pub fn with_core(core: Arc<super::core::ErasureCore>) -> Self {
        Self { core: Some(core) }
    }

    /// Liefert die Core-Referenz, falls vorhanden.
    pub fn core(&self) -> Option<&Arc<super::core::ErasureCore>> {
        self.core.as_ref()
    }

    /// Führt den nächsten Schritt der Saga durch (idempotent, ER-18).
    ///
    /// Validiert den Übergang und vollzieht den Schritt in der Zustandsmaschine.
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
            SagaState::Completed | SagaState::Refused | SagaState::Lifted => {
                return Ok(StepOutcome::Terminal(saga.state()))
            }
            SagaState::Restricted => {
                // Restricted wartet auf Lifted oder Erase (D-116, D-119).
                return Ok(StepOutcome::Terminal(SagaState::Restricted));
            }
        };

        saga.transition_to(next)?;
        Ok(StepOutcome::Advanced(saga.state()))
    }

    /// Asynchrone Ausführung des nächsten Treiberschritts unter Nutzung des `ErasureCore`.
    pub async fn step_async(&self, saga: &mut EraseSaga) -> Result<StepOutcome, ErasureError> {
        if let Some(core) = &self.core {
            let id = saga.erasure_id();
            if let Ok(st) = core.status(id).await {
                saga.transition_to(st.state)?;
                if saga.is_terminal() {
                    return Ok(StepOutcome::Terminal(st.state));
                }
                return Ok(StepOutcome::Advanced(st.state));
            }
        }
        self.step(saga)
    }

    /// Verarbeitet ein Ereignis aus der Treiber-Pipeline (SPEC §11.2).
    pub fn process_event(
        &self,
        saga: &mut EraseSaga,
        event: SagaEvent,
    ) -> Result<StepOutcome, ErasureError> {
        match event {
            SagaEvent::Advance(id) => {
                if id != saga.erasure_id() {
                    return Err(ErasureError::Internal("erasure id mismatch".into()));
                }
                self.step(saga)
            }
            SagaEvent::SetPending(id, reason) => {
                if id != saga.erasure_id() {
                    return Err(ErasureError::Internal("erasure id mismatch".into()));
                }
                saga.transition_with_reason(SagaState::Pending, Some(reason.clone()))?;
                Ok(StepOutcome::Pending(reason))
            }
            SagaEvent::Complete(id) => {
                if id != saga.erasure_id() {
                    return Err(ErasureError::Internal("erasure id mismatch".into()));
                }
                saga.transition_to(SagaState::Completed)?;
                Ok(StepOutcome::Terminal(SagaState::Completed))
            }
        }
    }
}
