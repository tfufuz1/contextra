//! Zustandsmaschine des Löschablaufs (SPEC §11.2, ER-18).
//!
//! Zustände:
//!   Requested → Gated → KeyDestroyed → Purging → Verifying → Completed
//!      └──────────┴──────────┴── Pending(reason) (idempotent, wiederholbar)
//!   Refused  (terminal, vor Gate)

use crate::types_local::EraseTarget as PortEraseTarget;
use crate::types_local::ErasureId;
use crate::types_local::PendingReason;
use serde::{Deserialize, Serialize};

pub use self::EraseRequest as SagaEraseRequest;
pub use crate::types_local::{EraseMode as SagaMode, EraseTarget as SagaEraseTarget};

/// Anfrage zur Löschung (Kompatibilität mit Ports & Tests).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EraseRequest {
    /// Ziel der Löschung
    pub target: PortEraseTarget,
    /// Eindeutige Lösch-ID
    pub erasure_id: [u8; 16],
}

/// Saga-Zustand (SPEC §11.2, §7.4, D-124 Abs. 5, ER-29).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug, Serialize, Deserialize)]
pub enum SagaState {
    /// Löschauftrag eingegangen.
    Requested,
    /// Gate gesetzt und Schlüssel vernichtet – Daten unlesbar.
    Gated,
    /// Schlüssel vernichtet (Intermediate-Zustand, Purge noch offen).
    KeyDestroyed,
    /// Purge-Phase läuft.
    Purging,
    /// Verifikations-Phase läuft.
    Verifying,
    /// Löschung vollständig abgeschlossen.
    Completed,
    /// Zugriffsbeschränkung aktiv (Art. 18 DSGVO, D-116).
    Restricted,
    /// Zugriffsbeschränkung aufgehoben (D-116).
    Lifted,
    /// Temporär verzögert (mit Grund).
    Pending,
    /// Endgültig verweigert (terminal, vor Gate).
    Refused,
}

impl SagaState {
    /// Prüft ob dieser Zustand ein Endzustand ist.
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Completed | Self::Refused | Self::Lifted)
    }

    /// Prüft ob ein Übergang zu `next` erlaubt ist (SPEC §11.2, I-4, FW-9).
    pub fn can_transition_to(self, next: SagaState) -> bool {
        use SagaState::*;
        match (self, next) {
            // Normalpfad Löschung
            (Requested, Gated) => true,
            (Requested, Restricted) => true,
            (Requested, Refused) => true,
            (Requested, Pending) => true,
            (Gated, KeyDestroyed) => true,
            (Gated, Purging) => true, // D-117: Superseded Version ohne KeyDestroy direkt zu Purging
            (Gated, Pending) => true,
            (KeyDestroyed, Purging) => true,
            (KeyDestroyed, Pending) => true,
            (Purging, Verifying) => true,
            (Purging, Pending) => true,
            (Verifying, Completed) => true,
            (Verifying, Pending) => true,
            // Restriktion (D-116)
            (Restricted, Lifted) => true,
            (Restricted, Gated) => true, // Erase hebt Restriktion durch Löschung auf (D-15, D-119)
            (Restricted, Pending) => true,
            // Pending kann nur zu den aktiven Schritten fortsetzen, NIE direkt zu Completed (FW-9, I-4)
            (Pending, Gated) => true,
            (Pending, KeyDestroyed) => true,
            (Pending, Purging) => true,
            (Pending, Verifying) => true,
            // Idempotenz: gleicher Zustand ist kein Fehler
            (a, b) if a == b => true,
            _ => false,
        }
    }
}

impl std::fmt::Display for SagaState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Requested => write!(f, "Requested"),
            Self::Gated => write!(f, "Gated"),
            Self::KeyDestroyed => write!(f, "KeyDestroyed"),
            Self::Purging => write!(f, "Purging"),
            Self::Verifying => write!(f, "Verifying"),
            Self::Completed => write!(f, "Completed"),
            Self::Restricted => write!(f, "Restricted"),
            Self::Lifted => write!(f, "Lifted"),
            Self::Pending => write!(f, "Pending"),
            Self::Refused => write!(f, "Refused"),
        }
    }
}

/// Zustandsmaschine für einen einzelnen Löschvorgang.
#[derive(Debug, Clone)]
pub struct EraseSaga {
    /// Eindeutige Kennung.
    erasure_id: ErasureId,
    /// Aktueller Zustand.
    state: SagaState,
    /// Letzter Pending-Grund (falls vorhanden).
    pending_reason: Option<PendingReason>,
}

impl Default for EraseSaga {
    fn default() -> Self {
        Self::new(ErasureId::new([0u8; 16]))
    }
}

impl EraseSaga {
    /// Erstellt eine neue Saga mit gegebener ErasureId im Zustand `Requested`.
    pub fn new(erasure_id: ErasureId) -> Self {
        Self {
            erasure_id,
            state: SagaState::Requested,
            pending_reason: None,
        }
    }

    /// Erstellt eine neue Saga mit gegebener ErasureId im Zustand `Requested`.
    pub fn with_id(erasure_id: ErasureId) -> Self {
        Self::new(erasure_id)
    }

    /// Erstellt eine Saga mit einem vorgegebenen Zustand (Recovery-Pfad).
    pub fn with_state(erasure_id: ErasureId, state: SagaState) -> Self {
        Self {
            erasure_id,
            state,
            pending_reason: None,
        }
    }

    /// Gibt die Lösch-ID zurück.
    pub fn erasure_id(&self) -> ErasureId {
        self.erasure_id
    }

    /// Gibt den aktuellen Zustand zurück.
    pub fn state(&self) -> SagaState {
        self.state
    }

    /// Gibt den letzten Pending-Grund zurück (falls vorhanden).
    pub fn pending_reason(&self) -> Option<&PendingReason> {
        self.pending_reason.as_ref()
    }

    /// Führt einen einfachen Zustandsübergang durch (idempotent).
    pub fn transition_to(&mut self, next: SagaState) -> Result<(), crate::error::ErasureError> {
        self.transition_with_reason(next, None)
    }

    /// Führt einen Zustandsübergang mit Begründung durch (ER-18: idempotent).
    ///
    /// Gibt `Err` zurück wenn der Übergang nicht erlaubt ist.
    pub fn transition_with_reason(
        &mut self,
        next: SagaState,
        reason: Option<PendingReason>,
    ) -> Result<(), crate::error::ErasureError> {
        if !self.state.can_transition_to(next) {
            return Err(crate::error::ErasureError::InvalidTransition {
                from: self.state.to_string(),
                to: next.to_string(),
            });
        }
        self.state = next;
        self.pending_reason = reason;
        Ok(())
    }

    /// Prüft ob die Saga abgeschlossen ist.
    pub fn is_terminal(&self) -> bool {
        self.state.is_terminal()
    }
}
