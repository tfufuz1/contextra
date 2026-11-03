//! Zeroisierende Geheimnis-Wrapper (SPEC 7.1, TY-11).
//!
//! `Secret` implementiert weder `Debug` noch `Display` noch `Clone`.
//! Beim Verlassen des Gültigkeitsbereichs (`Drop`) werden die Bytes genullt (`zeroize`).

use core::fmt;
use zeroize::{Zeroize, ZeroizeOnDrop};

/// Zeroisierender Speicherpuffer für Schlüsselmaterial und sensible Daten (SPEC 7.1, TY-11).
/// Kein Debug mit Klartext, kein Display, kein Clone.
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct Secret {
    bytes: Vec<u8>,
}

impl Secret {
    /// Erzeugt ein neues `Secret` aus einem Byte-Vektor.
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { bytes }
    }

    /// Gewährt schreibgeschützten Einblick in die geschützten Bytes.
    pub fn expose_secret(&self) -> &[u8] {
        &self.bytes
    }

    /// Liefert die Länge des geschützten Speichers.
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// Prüft, ob das Secret leer ist.
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}

impl fmt::Debug for Secret {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Secret([REDACTED {} bytes])", self.bytes.len())
    }
}
