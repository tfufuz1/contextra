//! Fehlerarchitektur (Spec 3.5).

/// Fehlerklasse. `Integrity` und `Crypto` loesen Vergiftung aus (15.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ErrorClass {
    /// Wiederholung mit Backoff.
    Transient,
    /// Integritaetsfehler (Vergiftung).
    Integrity,
    /// Kryptofehler (Vergiftung).
    Crypto,
    /// Nur aus Gate oder Registerbeleg, nie aus Abwesenheit eines Schluessels (Z10).
    Erased,
    /// Ablehnung (Rolle, Zugriff, Freigabe).
    Policy,
    /// Ablehnung der Eingabe; nichts wird geschrieben.
    Invalid,
}

/// Klassifizierbarer Fehler. `Display` und `code()` enthalten keine Personendaten (Z15).
pub trait ClassifiedError: std::error::Error {
    /// Fehlerklasse.
    fn class(&self) -> ErrorClass;
    /// Stabiler Code ohne Nutzdaten und Kennungen.
    fn code(&self) -> &'static str;
}
