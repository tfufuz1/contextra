//! Fehler-Kompatibilität für `GuardError`.
//!
//! Gemäß Invariante I3 verbleibt die Fehlerabbildung isoliert in `compat/error.rs`.

use crate::error::GuardError;

impl GuardError {
    /// Liefert die Fehlermeldung als String-Repräsentation.
    pub fn error_message(&self) -> String {
        self.to_string()
    }
}
