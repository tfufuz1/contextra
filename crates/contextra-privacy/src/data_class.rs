//! Kanonische Datenklassen (SPEC §3.1) und Typestate-Payload-Guard (D-02, D-03).

use crate::egress_vault::EntityRecognizer;

/// Die vier normativen Datenklassen des Contextra-Systems (SPEC §3.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum DataClass {
    /// Klasse K: Unternehmenswissen (Verträge, Handbücher, Richtlinien).
    /// Verschlüsselt mit Collection-KEK. Löschbar nur per Collection-Drop.
    K,

    /// Klasse P: Personenbezogene Daten natürlicher Personen (Name, E-Mail, Interaktionen).
    /// Adressiert über `SubjectRef`, verschlüsselt mit individuellem SubjectKey.
    /// Atomar löschbar in O(1) via Crypto-Shredding.
    P,

    /// Klasse D: Abgeleitete Daten (Embeddings, Postings, Graph-Knoten, Caches).
    /// Tragen verpflichtend `LineageRef` zum Quelldatum für kaskadierte Bereinigung.
    D,

    /// Klasse E: Löschmaterial (Journal, Ledger, signierte Berichte, Attestierungen).
    /// Verschlüsselt mit separatem ErasureKey; eigenständig nach Aufbewahrungsfrist bereinigt.
    E,
}

/// Typestate-Wrapper: Bezeugt, dass der innere Payload erfolgreich gegen
/// PII-Muster geprüft wurde und keine personenbezogenen Daten enthält (SPEC §3.1, D-03).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Validated<T>(T);

impl<T> Validated<T> {
    /// Erzeugt ein Validated<T> direkt (nur für interne verifizierte Konstruktoren).
    #[inline]
    pub const fn new_unchecked(val: T) -> Self {
        Self(val)
    }

    /// Liefert eine Referenz auf die validierten Daten.
    #[inline]
    pub fn inner(&self) -> &T {
        &self.0
    }

    /// Verbraucht den Wrapper und gibt die validierten Daten frei.
    #[inline]
    pub fn into_inner(self) -> T {
        self.0
    }
}

/// Prüft einen Text auf PII. Bei Fund schlägt die Validierung fehl.
pub fn validate_k_payload<T: AsRef<str>>(
    payload: T,
    recognizer: &dyn EntityRecognizer,
) -> Result<Validated<T>, PersonalDataFound> {
    let matches = recognizer.recognize(payload.as_ref());
    if let Some((range, category)) = matches.into_iter().next() {
        return Err(PersonalDataFound {
            entity_type: category,
            byte_range: range,
        });
    }
    Ok(Validated(payload))
}

/// Fehler: Personenbezogene Daten in Klasse-K-Payload entdeckt.
#[derive(Debug, Clone, thiserror::Error)]
#[error("Personal data detected in Class K payload: entity '{entity_type}' at range {byte_range:?}")]
pub struct PersonalDataFound {
    /// Typ der erkannten Entität (z.B. "IBAN", "EMAIL").
    pub entity_type: String,
    /// Byte-Bereich im geprüften Text.
    pub byte_range: std::ops::Range<usize>,
}
