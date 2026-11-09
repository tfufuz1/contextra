//! Lokale Kern-Traits für `contextra-infer-candle`.
//!
//! # Zweck
//! Definiert standardisierte, pure-Rust Traits für Text-Embedding und Text-Generierung
//! ohne Abhängigkeit von externen Frameworks oder Workspace-Crates (Klasse A).

use crate::error::Result;

// ANCHOR[contextra-infer-candle:TRAITS-001]

/// Lokaler Trait für Vektor-Embeddings (z.B. BERT, MiniLM).
pub trait Embedder: Send + Sync {
    /// Erzeugt einen Embedding-Vektor für den gegebenen Text.
    fn embed(&self, text: &str) -> Result<Vec<f32>>;

    /// Erzeugt Embedding-Vektoren für einen Batch von Texten.
    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let mut results = Vec::with_capacity(texts.len());
        for text in texts {
            results.push(self.embed(text)?);
        }
        Ok(results)
    }

    /// Gibt die Dimension der erzeugten Einbettungen zurück.
    fn dimension(&self) -> usize;
}

/// Optionen für die Textgenerierung.
#[derive(Debug, Clone, PartialEq)]
pub struct GenerationConfig {
    /// Maximale Anzahl neu zu generierender Tokens.
    pub max_new_tokens: usize,
    /// Sampling-Temperatur (0.0 = strikt deterministisches Greedy Sampling).
    pub temperature: f64,
    /// Top-p Nucleus-Sampling Schwelle.
    pub top_p: Option<f64>,
    /// Optionaler deterministischer Seed für Reproduzierbarkeit.
    pub seed: Option<u64>,
    /// Stopp-Sequenzen für den Abbruch.
    pub stop_tokens: Vec<String>,
}

impl Default for GenerationConfig {
    fn default() -> Self {
        Self {
            max_new_tokens: 128,
            temperature: 0.0,
            top_p: None,
            seed: Some(42),
            stop_tokens: Vec::new(),
        }
    }
}

/// Lokaler Trait für synchrone Textgenerierung.
pub trait TextGenerator: Send + Sync {
    /// Generiert Text auf Basis des angegebenen Prompts und Konfiguration.
    fn generate(&self, prompt: &str, config: &GenerationConfig) -> Result<String>;
}

/// Lokaler Trait für KV-State-Senke (Export von Attention- oder KV-Zuständen).
pub trait KvStateSink: Send + Sync {
    /// Speichert ein KV-Zustandssegment unter einem opaken Schlüssel.
    fn store_segment(&mut self, segment_id: &[u8], payload: &[u8]) -> Result<()>;
}

/// Lokaler Trait für KV-State-Quelle (Wiederherstellung von Attention- oder KV-Zuständen).
pub trait KvStateSource: Send + Sync {
    /// Liest ein KV-Zustandssegment anhand des opaken Schlüssels.
    fn load_segment(&self, segment_id: &[u8]) -> Result<Option<Vec<u8>>>;
}
