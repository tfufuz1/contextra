//! # Kindprozess-Hilfsmodul für exklusive Verzeichnis-Lock-Tests (T-LOCK-1)
//!
//! Zweck: Ausführen eines zweiten Prozesses, der versucht, ein bereits gesperrtes
//! Datenverzeichnis zu öffnen, und den Beendigungscode 42 meldet, wenn der feste
//! Fehlertext `"Datenverzeichnis bereits in Benutzung"` auftritt.

#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::disallowed_methods,
    clippy::exit
)]

use contextra_durable_fs::dir_lock::{DirLock, DIR_LOCKED_MSG};
use std::path::Path;

/// Umgebungsvariable zur Signalisierung des Kindprozess-Modus.
pub const ENV_LOCK_CHILD_MODE: &str = "CONTEXTRA_TEST_LOCK_CHILD";
/// Umgebungsvariable für den Pfad des zu sperrenden Datenverzeichnisses.
pub const ENV_LOCK_DIR_PATH: &str = "CONTEXTRA_TEST_LOCK_DIR";

/// Beendigungscode bei erwartetem Sperrkonflikt (DF-05).
pub const EXIT_LOCK_CONFLICT: i32 = 42;
/// Beendigungscode bei unerwartetem Fehler.
pub const EXIT_UNEXPECTED_ERROR: i32 = 1;
/// Beendigungscode bei erfolgreichem Lock-Erwerb.
pub const EXIT_LOCK_ACQUIRED: i32 = 0;

/// Führt die Sperrprüfung aus, falls der aktuelle Prozess als Lock-Child gestartet wurde.
pub fn run_child_if_requested() {
    if std::env::var(ENV_LOCK_CHILD_MODE).is_err() {
        return;
    }

    let dir_str = match std::env::var(ENV_LOCK_DIR_PATH) {
        Ok(val) => val,
        Err(_) => std::process::exit(EXIT_UNEXPECTED_ERROR),
    };

    let dir = Path::new(&dir_str);
    match DirLock::acquire(dir) {
        Ok(_guard) => {
            // Lock erfolgreich erhalten
            std::process::exit(EXIT_LOCK_ACQUIRED);
        }
        Err(err) => {
            let msg = err.to_string();
            if msg.contains(DIR_LOCKED_MSG) {
                // Erwarteter Sperrkonflikt (DF-05 / T-LOCK-1)
                std::process::exit(EXIT_LOCK_CONFLICT);
            } else {
                eprintln!("Unerwarteter Fehler im Kindprozess: {err}");
                std::process::exit(EXIT_UNEXPECTED_ERROR);
            }
        }
    }
}
