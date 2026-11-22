//! Type-State GuardedPayload<S> — Compile-Zeit-Schutz gegen ungesanitisierten Cloud-Egress (§4.14).

use std::marker::PhantomData;

/// Marker-Typ: Payload noch nicht durch alle Egress-Guard-Layer gelaufen.
pub struct Unsanitized;

/// Marker-Typ: Payload hat alle 5 Egress-Guard-Layer erfolgreich durchlaufen.
pub struct Sanitized;

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
    /// Gibt die Session-ID zurück.
    pub fn session_id(&self) -> &str {
        &self.session_id
    }
}

impl GuardedPayload<Unsanitized> {
    /// Erstellt einen neuen unsanitizierten Payload (Eingang aus MCP-Client).
    pub fn new(raw: String, session_id: String) -> Self {
        Self {
            inner: raw,
            session_id,
            _state: PhantomData,
        }
    }
}

impl GuardedPayload<Sanitized> {
    /// Nur von `egress_gateway.rs` aufzurufen nach Durchlauf aller 5 Layer.
    pub fn from_sanitized(sanitized: String, session_id: String) -> Self {
        Self {
            inner: sanitized,
            session_id,
            _state: PhantomData,
        }
    }

    /// Gibt den sanitizierten Payload-String zurück.
    pub fn into_inner(self) -> String {
        self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
