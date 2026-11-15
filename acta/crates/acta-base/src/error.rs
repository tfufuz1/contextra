//! Fehlerarchitektur (Spec 3.5) und Fehlertyp von acta-base (HR-4).

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

/// Fehler von acta-base (ein Fehlertyp je Crate, kein `Other(String)`).
///
/// Alle Varianten sind Eingabefehler der kanonischen Kodierung. Ob ein solcher Fehler beim
/// Aufrufer einen Integritaetsfehler bedeutet (etwa beim Lesen einer Platte), entscheidet der
/// Aufrufer; die Klasse hier ist `Invalid`. Die Texte enthalten weder Werte noch Laengen (Z15).
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[non_exhaustive]
pub enum BaseError {
    /// Weniger Bytes vorhanden als das Feld verlangt.
    #[error("canonical input truncated")]
    Truncated,
    /// Laengenfeld groesser als die zulaessige Obergrenze oder als der Rest der Eingabe.
    #[error("canonical length out of bounds")]
    LengthOutOfBounds,
    /// Nach dem letzten erwarteten Feld sind Bytes uebrig.
    #[error("canonical input has trailing bytes")]
    TrailingBytes,
    /// Wert liegt ausserhalb des erlaubten Bereichs.
    #[error("value out of range")]
    OutOfRange,
}

impl ClassifiedError for BaseError {
    fn class(&self) -> ErrorClass {
        ErrorClass::Invalid
    }

    fn code(&self) -> &'static str {
        match self {
            Self::Truncated => "base.canon.truncated",
            Self::LengthOutOfBounds => "base.canon.length",
            Self::TrailingBytes => "base.canon.trailing",
            Self::OutOfRange => "base.range",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_are_stable_and_class_is_invalid() {
        let all = [
            (BaseError::Truncated, "base.canon.truncated"),
            (BaseError::LengthOutOfBounds, "base.canon.length"),
            (BaseError::TrailingBytes, "base.canon.trailing"),
            (BaseError::OutOfRange, "base.range"),
        ];
        for (e, code) in all {
            assert_eq!(e.code(), code);
            assert_eq!(e.class(), ErrorClass::Invalid);
        }
    }
}
