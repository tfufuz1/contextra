//! Konfiguration und Modellsicherheit.
//!
//! Stellt sichere Pfadprüfung gegen Traversal-Attacken und kryptografische
//! Modellverifikation (BLAKE3 / SHA-256) bereit.

use crate::error::{Error, Result};
use std::path::{Component, Path, PathBuf};

/// Unterstützte kryptografische Hash-Verfahren zur Modellverifikation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelHash {
    /// BLAKE3 Prüfsumme als Hexstring.
    Blake3(String),
    /// SHA-256 Prüfsumme als Hexstring.
    Sha256(String),
}

impl ModelHash {
    /// Überprüft den Hash einer Datei am angegebenen Pfad.
    ///
    /// # Errors
    /// Gibt `Error::HashMismatch` zurück, wenn die Prüfsumme nicht übereinstimmt,
    /// oder `Error::Io` bei Lesefehlern.
    pub fn verify_file(&self, path: &Path) -> Result<()> {
        let bytes = std::fs::read(path)?;
        match self {
            Self::Blake3(expected) => {
                let actual = blake3::hash(&bytes).to_hex().to_string();
                if !actual.eq_ignore_ascii_case(expected) {
                    return Err(Error::HashMismatch {
                        path: path.to_string_lossy().to_string(),
                        expected: expected.clone(),
                        actual,
                    });
                }
            }
            Self::Sha256(expected) => {
                #[cfg(feature = "onnx")]
                {
                    use sha2::{Digest, Sha256};
                    let mut hasher = Sha256::new();
                    hasher.update(&bytes);
                    let actual = format!("{:x}", hasher.finalize());
                    if !actual.eq_ignore_ascii_case(expected) {
                        return Err(Error::HashMismatch {
                            path: path.to_string_lossy().to_string(),
                            expected: expected.clone(),
                            actual,
                        });
                    }
                }
                #[cfg(not(feature = "onnx"))]
                {
                    let _ = expected;
                    return Err(Error::Unsupported(
                        "SHA-256 Verifikation erfordert das onnx-Feature".into(),
                    ));
                }
            }
        }
        Ok(())
    }
}

/// Prüft, ob ein übergebener Zielpfad sicher innerhalb eines Basisverzeichnisses liegt.
///
/// Schützt vor Path-Traversal (`..`, absolute Pfade außerhalb des Root).
///
/// # Errors
/// Gibt `Error::SecurityViolation` zurück, falls der Pfad das Basisverzeichnis verlässt.
pub fn verify_path_safety(base: &Path, user_path: &Path) -> Result<PathBuf> {
    for comp in user_path.components() {
        if matches!(comp, Component::ParentDir) {
            return Err(Error::SecurityViolation(format!(
                "Path traversal erkannt im Pfad: {}",
                user_path.display()
            )));
        }
    }

    let combined = if user_path.is_absolute() {
        user_path.to_path_buf()
    } else {
        base.join(user_path)
    };

    Ok(combined)
}

/// Pooling-Modus für Embeddings aus Transformer-Hidden-States.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PoolingMode {
    /// Mittelwert über alle nicht maskierten Token-Vektoren.
    #[default]
    Mean,
    /// Repräsentation des ersten Tokens (`[CLS]`).
    Cls,
}

/// Konfiguration für einen ONNX-Embedder.
#[derive(Debug, Clone)]
pub struct EmbedderConfig {
    /// Pfad zum ONNX-Modell.
    pub model_path: PathBuf,
    /// Pfad zur `tokenizer.json`.
    pub tokenizer_path: PathBuf,
    /// Erwartete Ausgabedimension.
    pub expected_dimension: usize,
    /// Maximale Sequenzlänge (Tokenanzahl).
    pub max_sequence_length: usize,
    /// Gewählter Pooling-Modus.
    pub pooling: PoolingMode,
    /// Ob L2-Normalisierung durchgeführt werden soll.
    pub normalize_l2: bool,
    /// Anzahl an Inferenz-Threads.
    pub num_threads: usize,
    /// Optionaler erwarteter Hash für die Modelldatei.
    pub model_hash: Option<ModelHash>,
    /// Optionaler erwarteter Hash für die Tokenizerdatei.
    pub tokenizer_hash: Option<ModelHash>,
}

impl EmbedderConfig {
    /// Erstellt eine Basiskonfiguration mit sicheren Standardwerten.
    #[must_use]
    pub fn new(model_path: PathBuf, tokenizer_path: PathBuf, expected_dimension: usize) -> Self {
        Self {
            model_path,
            tokenizer_path,
            expected_dimension,
            max_sequence_length: 512,
            pooling: PoolingMode::Mean,
            normalize_l2: true,
            num_threads: 1,
            model_hash: None,
            tokenizer_hash: None,
        }
    }

    /// Validiert die Konfiguration und prüft ggf. Dateiprüfsummen.
    ///
    /// # Errors
    /// Gibt Fehler bei ungültigen Dimensionen, Thread-Zahlen oder Hash-Abweichungen zurück.
    pub fn validate(&self) -> Result<()> {
        if self.expected_dimension == 0 {
            return Err(Error::InvalidConfig("Dimension darf nicht 0 sein".into()));
        }
        if self.max_sequence_length == 0 {
            return Err(Error::InvalidConfig(
                "Max-Sequenzlänge darf nicht 0 sein".into(),
            ));
        }
        if self.num_threads == 0 {
            return Err(Error::InvalidConfig("Threadanzahl muss >= 1 sein".into()));
        }
        if let Some(hash) = &self.model_hash {
            hash.verify_file(&self.model_path)?;
        }
        if let Some(hash) = &self.tokenizer_hash {
            hash.verify_file(&self.tokenizer_path)?;
        }
        Ok(())
    }
}

/// Konfiguration für einen ONNX Cross-Encoder Reranker.
#[derive(Debug, Clone)]
pub struct RerankerConfig {
    /// Pfad zum ONNX-Modell.
    pub model_path: PathBuf,
    /// Pfad zur `tokenizer.json`.
    pub tokenizer_path: PathBuf,
    /// Maximale Tokenanzahl je Paar.
    pub max_length: usize,
    /// Anzahl an Inferenz-Threads.
    pub num_threads: usize,
    /// Optionaler Modell-Hash.
    pub model_hash: Option<ModelHash>,
    /// Optionaler Tokenizer-Hash.
    pub tokenizer_hash: Option<ModelHash>,
}

impl RerankerConfig {
    /// Erstellt eine Basiskonfiguration für den Reranker.
    #[must_use]
    pub fn new(model_path: PathBuf, tokenizer_path: PathBuf) -> Self {
        Self {
            model_path,
            tokenizer_path,
            max_length: 512,
            num_threads: 1,
            model_hash: None,
            tokenizer_hash: None,
        }
    }

    /// Validiert die Reranker-Konfiguration.
    ///
    /// # Errors
    /// Meldet Konfigurationsfehler oder Hash-Mismatches.
    pub fn validate(&self) -> Result<()> {
        if self.max_length == 0 {
            return Err(Error::InvalidConfig("Max-Länge darf nicht 0 sein".into()));
        }
        if self.num_threads == 0 {
            return Err(Error::InvalidConfig("Threadanzahl muss >= 1 sein".into()));
        }
        if let Some(hash) = &self.model_hash {
            hash.verify_file(&self.model_path)?;
        }
        if let Some(hash) = &self.tokenizer_hash {
            hash.verify_file(&self.tokenizer_path)?;
        }
        Ok(())
    }
}
