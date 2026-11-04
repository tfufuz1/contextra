//! Behandlungslogik für Poisoned-Zustände nach I/O- oder fsync-Fehlern (SPEC §4.2 D-55, §13.1 I-10).

use contextra_types::{ContextraError, Result};
use std::sync::atomic::{AtomicBool, Ordering};

/// Atomarer Poison-Status für Speichertriebwerke.
#[derive(Debug, Default)]
pub struct PoisonState {
    poisoned: AtomicBool,
}

impl PoisonState {
    /// Erzeugt einen neuen unvergifteten Zustand.
    pub fn new() -> Self {
        Self {
            poisoned: AtomicBool::new(false),
        }
    }

    /// Vergiftet den Speicherzustand dauerhaft nach einem fsync- oder I/O-Fehler (D-55).
    pub fn poison(&self) {
        self.poisoned.store(true, Ordering::SeqCst);
    }

    /// Prüft, ob der Speicher vergiftet ist.
    pub fn is_poisoned(&self) -> bool {
        self.poisoned.load(Ordering::SeqCst)
    }

    /// Weist mutierende Operationen mit `ContextraError::Io` ab, wenn vergiftet (D-55).
    pub fn check_healthy(&self) -> Result<()> {
        if self.is_poisoned() {
            Err(ContextraError::Storage(
                "Storage engine is poisoned due to a previous fsync or I/O failure (SPEC D-55)".into(),
            ))
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poison_state_lifecycle() {
        let state = PoisonState::new();
        assert!(!state.is_poisoned());
        assert!(state.check_healthy().is_ok());

        state.poison();
        assert!(state.is_poisoned());
        assert!(state.check_healthy().is_err());
    }
}
