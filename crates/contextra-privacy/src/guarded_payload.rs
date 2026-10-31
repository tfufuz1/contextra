//! Type-State `GuardedPayload<S>` — Compile-Zeit-Schutz gegen ungesanitisierten Cloud-Egress (§4.14).
//!
//! # Zweck und Konzept
//! Stellt über das Type-State-Muster (Zustandscodierung im Typsystem) sicher, dass ausgehende
//! Anfragedaten (Payloads) nicht versehentlich ungesäubert an externe Cloud-Dienste weitergeleitet
//! werden können. Ein Payload im Zustand `Unsanitized` kann zur Compile-Zeit nicht an Funktionen
//! übergeben werden, die zwingend einen `Sanitized`-Payload verlangen.
//!
//! # Ring-Zugehörigkeit
//! Ring 3 (Domain-Typen).

use std::marker::PhantomData;

// ANCHOR[contextra-privacy:GP-001]
/// Marker-Typ: Payload hat noch nicht alle Egress-Guard-Layer durchlaufen und gilt als unbereinigt.
pub struct Unsanitized;

// ANCHOR[contextra-privacy:GP-002]
/// Marker-Typ: Payload hat alle 5 Egress-Guard-Layer erfolgreich durchlaufen und ist bereinigt.
pub struct Sanitized;

// ANCHOR[contextra-privacy:GP-003]
/// Type-State-Wrapper für Cloud-Egress-Payloads.
///
/// Ein `GuardedPayload<Sanitized>` kann NUR durch den vollständigen Durchlauf
/// aller 5 Egress-Guard-Layer konstruiert werden (`egress_gateway.rs`).
/// Ein `GuardedPayload<Unsanitized>` an `dispatch_to_cloud()` zu übergeben ist ein COMPILE-FEHLER.
///
/// # Invariante (§12.3)
/// `dispatch_to_cloud()` akzeptiert ausschließlich `GuardedPayload<Sanitized>`.
/// Diese Invariante ist nicht durch Laufzeit-Checks, sondern durch das Typsystem erzwungen.
#[derive(Debug)]
pub struct GuardedPayload<S> {
    pub(crate) inner: String,
    pub(crate) session_id: String,
    _state: PhantomData<S>,
}

impl<S> GuardedPayload<S> {
    // ANCHOR[contextra-privacy:GP-004]
    /// Gibt eine Referenz auf die zugeordnete Sitzungs-ID (`session_id`) zurück.
    pub fn session_id(&self) -> &str {
        &self.session_id
    }
}

impl GuardedPayload<Unsanitized> {
    // ANCHOR[contextra-privacy:GP-005]
    /// Erstellt einen neuen unsanitizierten Payload (Eingang aus MCP-Client oder interner Engine).
    ///
    /// # Ablauf
    /// 1. Übernimmt den rohen Eingabestring `raw`.
    /// 2. Verknüpft die übergebene `session_id`.
    /// 3. Initialisiert den `PhantomData`-Marker mit Zustand `Unsanitized`.
    pub fn new(raw: String, session_id: String) -> Self {
        Self {
            inner: raw,
            session_id,
            _state: PhantomData,
        }
    }
}

impl GuardedPayload<Sanitized> {
    // ANCHOR[contextra-privacy:GP-006]
    /// Konstruiert einen bereinigten Payload nach erfolgreicher Bereinigung.
    ///
    /// # Ablauf
    /// Nur von `egress_gateway.rs` nach erfolgreichem Durchlauf aller Egress-Filter aufzurufen.
    /// 1. Kapselt den bereinigten String `sanitized`.
    /// 2. Verknüpft die `session_id`.
    /// 3. Weist den Compile-Zeit-Zustand `Sanitized` zu.
    pub fn from_sanitized(sanitized: String, session_id: String) -> Self {
        Self {
            inner: sanitized,
            session_id,
            _state: PhantomData,
        }
    }

    // ANCHOR[contextra-privacy:GP-007]
    /// Extrahiert den bereinigten String aus dem Schutzbehälter für den Versand.
    ///
    /// # Ablauf
    /// Konsumiert die Instanz und gibt das interne `String`-Objekt frei.
    pub fn into_inner(self) -> String {
        self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ANCHOR[contextra-privacy:TEST-GP-001]
    // Prüft, dass die Type-States Unsanitized und Sanitized getrennt konstruierbar sind und Werte korrekt halten.
    #[test]
    fn test_guarded_payload_type_states_are_distinct() {
        let raw = GuardedPayload::<Unsanitized>::new("test".into(), "s1".into());
        assert_eq!(raw.inner, "test");
        assert_eq!(raw.session_id(), "s1");

        let sanitized = GuardedPayload::<Sanitized>::from_sanitized("clean".into(), "s2".into());
        assert_eq!(sanitized.session_id(), "s2");
        assert_eq!(sanitized.into_inner(), "clean");
    }
}
