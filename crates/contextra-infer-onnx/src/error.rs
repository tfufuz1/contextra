//! Fehlerdefinitionen für `contextra-infer-onnx`.
//!
//! Entspricht Klasse-A-Anforderung B4: Genau ein primärer Fehlertyp `Error`
//! mit `thiserror` und `#[non_exhaustive]`.

use thiserror::Error;

/// Primärer Fehlertyp für Inferenz-, Modelllade- und Validierungsoperationen.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// Das angeforderte Modell oder die Tokenizer-Datei wurde nicht gefunden.
    #[error("Modell oder Tokenizer nicht gefunden: {0}")]
    ModelNotFound(String),

    /// Verletzung von Sicherheitsrichtlinien (z. B. Path-Traversal).
    #[error("Sicherheitsverletzung: {0}")]
    SecurityViolation(String),

    /// Prüfsummenabweichung beim Laden eines Modells oder Tokenizers.
    #[error("Hash-Prüfsummenfehler für '{path}': erwartet {expected}, berechnet {actual}")]
    HashMismatch {
        /// Pfad der betroffenen Datei.
        path: String,
        /// Erwarteter Hash-Hexstring.
        expected: String,
        /// Tatsächlich ermittelter Hash-Hexstring.
        actual: String,
    },

    /// Ungültige oder unerwartete Vektordimension.
    #[error("Ungültige Dimension: erwartet {expected}, erhalten {actual}")]
    InvalidDimension {
        /// Erwartete Dimension.
        expected: usize,
        /// Tatsächliche Dimension.
        actual: usize,
    },

    /// Fehler während der Tokenisierung.
    #[error("Tokenisierungsfehler: {0}")]
    Tokenization(String),

    /// Fehler während der ONNX Runtime Inferenz.
    #[error("Inferenzfehler: {0}")]
    Inference(String),

    /// I/O-Fehler beim Lesen oder Schreiben lokaler Dateien.
    #[error("I/O-Fehler: {0}")]
    Io(#[from] std::io::Error),

    /// Ungültige Konfiguration oder Parameter.
    #[error("Ungültige Konfiguration: {0}")]
    InvalidConfig(String),

    /// Feature oder Fähigkeit ist in diesem Build nicht aktiviert.
    #[error("Fähigkeit nicht verfügbar: {0}")]
    Unsupported(String),
}

/// Kanonischer Result-Typ für `contextra-infer-onnx`.
pub type Result<T> = std::result::Result<T, Error>;
