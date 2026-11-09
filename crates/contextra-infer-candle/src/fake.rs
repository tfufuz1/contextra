//! Fake/Mock Inferenz- und Embedding-Backend für autarken Betrieb ohne `candle`.
//!
//! # Invarianten
//! - Deterministische Ausgabe: Bitreproduzierbare Embeddings und Tokens bei gegebenem Input/Seed.
//! - Zero-Panic: Keine Panics bei unvorhergesehenen Eingaben.
//! - Standalone: Funktioniert vollständig ohne GPU oder externe C-Bibliotheken.

use crate::error::Result;
use crate::traits::{Embedder, GenerationConfig, TextGenerator};

// ANCHOR[contextra-infer-candle:FAKE-001]

/// Deterministischer Fake-Embedder für Tests und Standalone-Nutzung ohne Candle.
#[derive(Debug, Clone)]
pub struct FakeEmbedder {
    dimension: usize,
}

impl FakeEmbedder {
    /// Erstellt eine neue Instanz mit der gewünschten Vektordimension.
    pub fn new(dimension: usize) -> Self {
        Self { dimension }
    }
}

impl Default for FakeEmbedder {
    fn default() -> Self {
        Self::new(384)
    }
}

impl Embedder for FakeEmbedder {
    fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let mut vec = vec![0.0f32; self.dimension];
        let bytes = text.as_bytes();
        if bytes.is_empty() {
            return Ok(vec);
        }

        // Deterministische Hash-Akkumulation über Dimensionen
        let mut acc = 0x811c9dc5u32;
        for (i, b) in bytes.iter().enumerate() {
            acc = acc.wrapping_mul(0x01000193) ^ (*b as u32);
            let idx = i % self.dimension;
            vec[idx] += ((acc & 0xFF) as f32) / 255.0;
        }

        // L2-Normalisierung
        let mut norm_sq = 0.0f32;
        for val in &vec {
            norm_sq += *val * *val;
        }
        if norm_sq > 0.0 {
            let norm = norm_sq.sqrt();
            for val in &mut vec {
                *val /= norm;
            }
        }

        Ok(vec)
    }

    fn dimension(&self) -> usize {
        self.dimension
    }
}

/// Deterministischer Fake-Text-Generator für Tests und Standalone-Nutzung.
#[derive(Debug, Clone, Default)]
pub struct FakeTextGenerator {
    prefix: String,
}

impl FakeTextGenerator {
    /// Erstellt eine neue Instanz mit optionalem Präfix für generierte Antworten.
    pub fn new(prefix: impl Into<String>) -> Self {
        Self {
            prefix: prefix.into(),
        }
    }
}

impl TextGenerator for FakeTextGenerator {
    fn generate(&self, prompt: &str, config: &GenerationConfig) -> Result<String> {
        let max_tokens = config.max_new_tokens.min(1024);
        let mut output = if self.prefix.is_empty() {
            format!("{prompt} -> Generated Completion")
        } else {
            format!("{}: {prompt}", self.prefix)
        };

        // Kürzen auf simulierte max_new_tokens falls nötig
        if output.len() > max_tokens.saturating_mul(4) {
            output.truncate(max_tokens.saturating_mul(4));
        }

        Ok(output)
    }
}
