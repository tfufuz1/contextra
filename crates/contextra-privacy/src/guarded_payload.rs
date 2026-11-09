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
//!
//! # Typ-Test (compile_fail)
//! ```compile_fail
//! // Direkte externe Feld-Konstruktion ist verboten:
//! use contextra_privacy::guarded_payload::{GuardedPayload, Sanitized};
//! let _p = GuardedPayload { inner: "geheim".to_string(), session_id: "s1".to_string(), _state: std::marker::PhantomData };
//! ```
//!
//! ```compile_fail
//! // Mutierender Zugriff ist verboten:
//! use contextra_privacy::guarded_payload::{GuardedPayload, Sanitized};
//! fn tamper(p: &mut GuardedPayload<Sanitized>) {
//!     p.clear(); // Fehler: kein DerefMut
//! }
//! ```
//!
//! ```compile_fail
//! // Unsanitized kann nicht an Sanitized-Empfänger übergeben werden:
//! use contextra_privacy::guarded_payload::{GuardedPayload, Sanitized, Unsanitized};
//! fn send_to_cloud(_p: GuardedPayload<Sanitized>) {}
//! fn run() {
//!     let raw = GuardedPayload::<Unsanitized>::new("raw".into(), "s1".into());
//!     send_to_cloud(raw); // Typfehler
//! }
//! ```

use std::fmt;
use std::marker::PhantomData;
use std::ops::Deref;

// ANCHOR[contextra-privacy:GP-001]
/// Marker-Typ: Payload hat noch nicht alle Egress-Guard-Layer durchlaufen und gilt als unbereinigt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Unsanitized;

// ANCHOR[contextra-privacy:GP-002]
/// Marker-Typ: Payload hat alle 5 Egress-Guard-Layer erfolgreich durchlaufen und ist bereinigt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
#[derive(Clone, PartialEq, Eq)]
pub struct GuardedPayload<S = Sanitized> {
    pub(crate) inner: String,
    pub(crate) session_id: String,
    pub(crate) _state: PhantomData<S>,
}

impl<S> GuardedPayload<S> {
    // ANCHOR[contextra-privacy:GP-004]
    /// Gibt eine Referenz auf die zugeordnete Sitzungs-ID (`session_id`) zurück.
    #[must_use]
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Gibt eine Referenz auf den Textinhalt zurück.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.inner
    }

    /// Gibt die Bytelänge des Textinhalts zurück.
    #[must_use]
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Gibt zurück, ob die Nutzlast leer ist.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }
}

impl GuardedPayload<Unsanitized> {
    // ANCHOR[contextra-privacy:GP-005]
    /// Erstellt einen neuen unsanitizierten Payload (Eingang aus MCP-Client oder interner Engine).
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
    pub(crate) fn from_sanitized(sanitized: String, session_id: String) -> Self {
        Self {
            inner: sanitized,
            session_id,
            _state: PhantomData,
        }
    }

    /// Extrahiert den bereinigten String aus dem Schutzbehälter für den Versand.
    #[must_use]
    pub fn into_inner(self) -> String {
        self.inner
    }

    #[cfg(any(test, feature = "test-doubles"))]
    #[doc(hidden)]
    pub fn from_sanitized_for_test(sanitized: String, session_id: String) -> Self {
        Self::from_sanitized(sanitized, session_id)
    }
}

impl<S> Deref for GuardedPayload<S> {
    type Target = str;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<S> AsRef<str> for GuardedPayload<S> {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.inner
    }
}

impl<S> AsRef<[u8]> for GuardedPayload<S> {
    #[inline]
    fn as_ref(&self) -> &[u8] {
        self.inner.as_bytes()
    }
}

impl<S> fmt::Debug for GuardedPayload<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("GuardedPayload")
            .field("session_id", &self.session_id)
            .field("bytes_len", &self.inner.len())
            .field("is_empty", &self.inner.is_empty())
            .finish()
    }
}

impl<S> fmt::Display for GuardedPayload<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ANCHOR[contextra-privacy:TEST-GP-001]
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
