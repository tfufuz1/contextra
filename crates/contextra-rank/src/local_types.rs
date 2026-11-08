//! Lokale Typdefinitionen für Autarkie von `contextra-rank` (B1/B3 Konformität).

use serde::{Deserialize, Serialize};

pub const MAX_SEARCH_K: usize = 10_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DocId(pub u64);

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConfigFingerprint(pub [u8; 32]);

impl ConfigFingerprint {
    pub const ZERO: Self = ConfigFingerprint([0u8; 32]);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FusionStrategy {
    Rrf,
    WeightedScore,
    GeometricMean,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalFailurePolicy {
    DropSignal,
    Abort,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContextChunk {
    pub chunk_id: u64,
    pub doc_id: DocId,
    pub text: String,
    pub score: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MemoryLink {
    pub target: DocId,
    pub weight: f32,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignalFusionStrategies {
    pub default: FusionStrategy,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FusionWeights {
    pub weights: Vec<f32>,
}
