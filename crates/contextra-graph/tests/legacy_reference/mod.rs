//! Legacy Reference types snapshot from contextra-graph at commit 98abece.
//! Herkunfts-Commit: 98abece

use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
pub struct LegacyEntityId(pub u64);

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
pub struct LegacyDocId(pub u64);

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
pub struct LegacyTxId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LegacyTlHfdParams {
    pub sigma: f32,
    pub delta: f32,
    pub gamma: f32,
    pub max_iterations: u32,
    pub max_top_k_expansion: usize,
    pub max_hyperedge_sort_size: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LegacyPprAlgorithm {
    #[default]
    Auto,
    DensePowerIteration,
    ForwardPush,
    ShadowMode,
    TlHfd(LegacyTlHfdParams),
    ShadowModeTlHfd(LegacyTlHfdParams),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LegacyPprConfig {
    pub damping_factor: f32,
    pub max_iterations: u32,
    pub convergence_epsilon: f32,
    pub algorithm: LegacyPprAlgorithm,
    pub warn_on_non_convergence: bool,
}
