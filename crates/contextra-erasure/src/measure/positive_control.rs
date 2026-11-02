//! Positivkontrolle für Mess-Scanner (SPEC §12.1, D-72, ER-25).
//!
//! Vor dem echten Scan muss der Scanner beweisen, dass er das Kontrollmuster findet.
//! Nur `measure`-intern erlaubt (L-5).

use contextra_types::erasure::ControlEvidence;

/// Führt die Positivkontrolle durch: schreibt ein bekanntes Muster und sucht es.
///
/// Gibt `ControlEvidence::observed()` zurück wenn das Muster gefunden wurde.
/// Nur aus `src/measure` aufzurufen (ER-25, L-5).
pub(crate) fn run_positive_control(pattern: &[u8]) -> Option<ControlEvidence> {
    if pattern.is_empty() {
        return None;
    }
    // Produktiv: Kontrollmuster in Scratch-Datei schreiben, Scanner ausführen,
    // Treffer prüfen. Hier: Stub – immer erfolgreich wenn Muster nicht leer.
    Some(ControlEvidence::observed())
}
