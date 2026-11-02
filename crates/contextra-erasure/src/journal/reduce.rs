//! Journal-Reduktion nach Completed (SPEC §11.6, R13, ER-29).
//!
//! Nach `Completed` werden Zwischenzustände aus dem Journal entfernt.

use super::record::{JournalEntry, SagaState};

/// Reduziert Journal-Einträge: Behält nur den letzten Eintrag je `erasure_id`
/// nach `Completed` (R13).
pub fn reduce(entries: &[JournalEntry]) -> Vec<JournalEntry> {
    let mut result: Vec<JournalEntry> = Vec::new();

    for entry in entries {
        if entry.state == SagaState::Completed {
            // Alle früheren Einträge für diese ID entfernen
            result.retain(|e| e.erasure_id != entry.erasure_id);
            result.push(entry.clone());
        } else {
            result.push(entry.clone());
        }
    }

    result
}
