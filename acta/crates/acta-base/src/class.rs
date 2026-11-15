//! Datenklassen (Regel R-CLS, Spec 2.2). Jeder Schreibvorgang deklariert eine Klasse.

/// Datenklasse eines Schreibvorgangs.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum DataClass {
    /// Ebene K: Sachinhalte ohne Personenbezug.
    Wissen,
    /// Ebene K: Entscheidungsprotokolle ohne Personenbezug.
    Beleg,
    /// Ebene P: personenbezogene Daten.
    Personal,
    /// Ebene P: Daten nach Art. 9 und 10 DSGVO.
    Besonders,
    /// Ebene D: Indizes, Caches, Embeddings, Sitzungen.
    Abgeleitet,
    /// Ebene P: Eingang ohne Entscheidung oder Betroffenenzuordnung.
    Unklassifiziert,
}

impl DataClass {
    /// `true`, wenn die Klasse zur Ebene P gehoert (verschluesselt, loeschbar).
    #[must_use]
    pub fn is_personal_layer(self) -> bool {
        matches!(
            self,
            Self::Personal | Self::Besonders | Self::Unklassifiziert
        )
    }
}
