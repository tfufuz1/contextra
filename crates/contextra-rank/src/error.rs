//! Fehlertypen für `contextra-rank`.

use thiserror::Error;

/// Fehler bei der Validierung von Eingaben und Berechnung der Reciprocal Rank Fusion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum RankError {
    /// Eine Kandidatenliste überschreitet die Obergrenze von 1.000 Einträgen (D-87).
    #[error("Kandidatenliste überschreitet die Obergrenze von 1000 Einträgen")]
    TooLarge,

    /// Ein Dokument kommt mehrfach innerhalb derselben Eingabeliste vor.
    #[error("Duplikat innerhalb einer Kandidatenliste")]
    Duplicate,

    /// Ganzzahlüberlauf bei der Festkomma-Akkumulation.
    #[error("Ganzzahlüberlauf bei der Festkomma-Akkumulation")]
    Overflow,
}
