//! Lokale Typdefinitionen für Autarkie von `contextra-rank` (B1/B3 Konformität).

use serde::{Deserialize, Serialize};

/// Maximale Anzahl an Suchergebnissen für Top-K-Buffer.
pub const MAX_SEARCH_K: usize = 10_000;

/// Opake Dokumentidentität für `contextra-rank`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DocId(pub u64);

impl DocId {
    /// Erzeugt eine `DocId` aus einer hexadezimalen oder numerischen Zeichenkette.
    pub fn from_key(key: &str) -> Result<Self, String> {
        if let Ok(val) = key.parse::<u64>() {
            return Ok(DocId(val));
        }
        if let Some(stripped) = key
            .strip_prefix("doc_")
            .or_else(|| key.strip_prefix("doc-"))
        {
            if let Ok(val) = stripped.parse::<u64>() {
                return Ok(DocId(val));
            }
        }
        // Fallback: Hash des Keys als stabiler Integer
        let mut hasher = ahash::AHasher::default();
        std::hash::Hash::hash(key, &mut hasher);
        use std::hash::Hasher;
        Ok(DocId(hasher.finish()))
    }
}

impl std::fmt::Display for DocId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for DocId {
    fn from(val: u64) -> Self {
        DocId(val)
    }
}

impl From<DocId> for u64 {
    fn from(id: DocId) -> Self {
        id.0
    }
}

#[cfg(feature = "types-compat")]
impl From<contextra_types::DocId> for DocId {
    fn from(id: contextra_types::DocId) -> Self {
        DocId(id.0)
    }
}

#[cfg(feature = "types-compat")]
impl From<DocId> for contextra_types::DocId {
    fn from(id: DocId) -> Self {
        contextra_types::DocId(id.0)
    }
}

/// Konfigurations-Fingerprint zur Validierung und Cache-Invalidierung.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConfigFingerprint {
    /// Identifikator oder Hash der Modell-/Promptkonfiguration.
    pub hash: [u8; 32],
}

impl ConfigFingerprint {
    /// Null-Fingerprint.
    pub const ZERO: Self = ConfigFingerprint { hash: [0u8; 32] };

    /// Erzeugt einen Fingerprint aus Modell, Quantisierung, Template und Temperatur.
    pub fn new(
        model_id: impl Into<String>,
        quantization: impl Into<String>,
        prompt_template: &str,
        temperature: f32,
    ) -> Self {
        let mut hasher = ahash::AHasher::default();
        std::hash::Hash::hash(&model_id.into(), &mut hasher);
        std::hash::Hash::hash(&quantization.into(), &mut hasher);
        std::hash::Hash::hash(prompt_template, &mut hasher);
        std::hash::Hash::hash(&temperature.to_bits(), &mut hasher);
        use std::hash::Hasher;
        let h1 = hasher.finish();
        let mut hash = [0u8; 32];
        hash[..8].copy_from_slice(&h1.to_le_bytes());
        hash[8..16].copy_from_slice(&h1.rotate_left(13).to_le_bytes());
        hash[16..24].copy_from_slice(&h1.rotate_left(29).to_le_bytes());
        hash[24..32].copy_from_slice(&h1.rotate_left(47).to_le_bytes());
        ConfigFingerprint { hash }
    }

    /// Erzeugt einen Fingerprint aus einem 32-Byte-Array.
    pub const fn from_bytes(bytes: [u8; 32]) -> Self {
        ConfigFingerprint { hash: bytes }
    }

    /// Gibt die rohen Bytes des Fingerprints zurück.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.hash
    }
}

#[cfg(feature = "types-compat")]
impl From<contextra_types::ConfigFingerprint> for ConfigFingerprint {
    fn from(fp: contextra_types::ConfigFingerprint) -> Self {
        ConfigFingerprint {
            hash: fp.prompt_template_hash,
        }
    }
}

/// Fusionsstrategie für hybride Suchen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum FusionStrategy {
    /// Standard Reciprocal Rank Fusion mit k=60.
    #[default]
    Rrf,
    /// Score-normalisierte gewichtete lineare Kombination.
    ScoreNormalized,
}

#[cfg(feature = "types-compat")]
impl From<contextra_types::FusionStrategy> for FusionStrategy {
    fn from(s: contextra_types::FusionStrategy) -> Self {
        match s {
            contextra_types::FusionStrategy::Rrf => FusionStrategy::Rrf,
            contextra_types::FusionStrategy::ScoreNormalized => FusionStrategy::ScoreNormalized,
        }
    }
}

#[cfg(feature = "types-compat")]
impl From<FusionStrategy> for contextra_types::FusionStrategy {
    fn from(s: FusionStrategy) -> Self {
        match s {
            FusionStrategy::Rrf => contextra_types::FusionStrategy::Rrf,
            FusionStrategy::ScoreNormalized => contextra_types::FusionStrategy::ScoreNormalized,
        }
    }
}

/// Richtlinie bei Signal-Fehlern.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SignalFailurePolicy {
    /// Bricht die Suche ab.
    #[default]
    Fail,
    /// Setzt die Suche mit reduzierten Signalen fort.
    Degrade,
}

#[cfg(feature = "types-compat")]
impl From<contextra_types::OnSignalFailure> for SignalFailurePolicy {
    fn from(p: contextra_types::OnSignalFailure) -> Self {
        match p {
            contextra_types::OnSignalFailure::Fail => SignalFailurePolicy::Fail,
            contextra_types::OnSignalFailure::Degrade => SignalFailurePolicy::Degrade,
        }
    }
}

#[cfg(feature = "types-compat")]
impl From<SignalFailurePolicy> for contextra_types::OnSignalFailure {
    fn from(p: SignalFailurePolicy) -> Self {
        match p {
            SignalFailurePolicy::Fail => contextra_types::OnSignalFailure::Fail,
            SignalFailurePolicy::Degrade => contextra_types::OnSignalFailure::Degrade,
        }
    }
}

/// Signal-Fusionsstrategien pro Kanal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SignalFusionStrategies {
    /// Strategie für Vektor.
    pub vector: FusionStrategy,
    /// Strategie für Text.
    pub text: FusionStrategy,
    /// Strategie für Graph.
    pub graph: FusionStrategy,
}

impl SignalFusionStrategies {
    /// Gibt die Fusionsstrategie für ein bestimmtes Signal ("vector", "text", "graph") zurück.
    pub fn strategy_for_signal(&self, signal_name: &str) -> FusionStrategy {
        match signal_name {
            "vector" => self.vector,
            "text" => self.text,
            "graph" => self.graph,
            _ => FusionStrategy::Rrf,
        }
    }
}

impl From<FusionStrategy> for SignalFusionStrategies {
    fn from(s: FusionStrategy) -> Self {
        Self {
            vector: s,
            text: s,
            graph: s,
        }
    }
}

#[cfg(feature = "types-compat")]
impl From<contextra_types::SignalFusionStrategies> for SignalFusionStrategies {
    fn from(s: contextra_types::SignalFusionStrategies) -> Self {
        SignalFusionStrategies {
            vector: s.vector.into(),
            text: s.text.into(),
            graph: s.graph.into(),
        }
    }
}

#[cfg(feature = "types-compat")]
impl From<SignalFusionStrategies> for contextra_types::SignalFusionStrategies {
    fn from(s: SignalFusionStrategies) -> Self {
        contextra_types::SignalFusionStrategies {
            vector: s.vector.into(),
            text: s.text.into(),
            graph: s.graph.into(),
        }
    }
}

/// Fusionsgewichte für die Normalisierung.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FusionWeights {
    /// Vektor-Gewicht.
    pub vector: f32,
    /// Text-Gewicht.
    pub text: f32,
    /// Graph-Gewicht.
    pub graph: f32,
    /// Zusätzliche Gewichte.
    pub metadata: f32,
}

impl Default for FusionWeights {
    fn default() -> Self {
        Self {
            vector: 1.0 / 3.0,
            text: 1.0 / 3.0,
            graph: 1.0 / 3.0,
            metadata: 0.0,
        }
    }
}

#[cfg(feature = "types-compat")]
impl From<contextra_types::FusionWeights> for FusionWeights {
    fn from(w: contextra_types::FusionWeights) -> Self {
        FusionWeights {
            vector: w.vector(),
            text: w.text(),
            graph: w.graph(),
            metadata: 0.0,
        }
    }
}
