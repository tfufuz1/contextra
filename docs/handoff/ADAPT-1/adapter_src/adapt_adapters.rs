//! Verbatim ausgelagerte Übergabeadapter für `contextra-adapt` (MISSION ADAPT-1).
//!
//! Gemäß SPEC v4.0 §2 (Klasse A, B1/B2/B3/E-05):
//! Der autarke Baustein enthält keine Abhängigkeiten zu `contextra-types` oder `contextra-ports`.
//! Diese Adapter werden vom Integrations-Prompt in Klasse C (`contextra-engine` / `ENG-2`) übernommen.

use contextra_adapt::bandit::BanditError;
use contextra_adapt::error::AdaptError;
use contextra_adapt::flow_thompson::{
    DiagonalApproximationBandit, FcTsError, FcTsSamplingDistribution, OffPolicyError,
};
use contextra_adapt::rie_greedy::RieGreedyError;
use contextra_adapt::strategy::RetrievalStrategy;

// --- RetrievalStrategy Konvertierungen (aus src/strategy.rs) ---

impl From<RetrievalStrategy> for contextra_types::RetrievalStrategy {
    fn from(strat: RetrievalStrategy) -> Self {
        match strat {
            RetrievalStrategy::Vector => contextra_types::RetrievalStrategy::Vector,
            RetrievalStrategy::Text => contextra_types::RetrievalStrategy::Text,
            RetrievalStrategy::Graph => contextra_types::RetrievalStrategy::Graph,
            RetrievalStrategy::Hybrid => contextra_types::RetrievalStrategy::Hybrid,
            RetrievalStrategy::Global => contextra_types::RetrievalStrategy::Global {
                community_level: 0,
                dynamic_community_selection: false,
            },
        }
    }
}

impl From<contextra_types::RetrievalStrategy> for RetrievalStrategy {
    fn from(strat: contextra_types::RetrievalStrategy) -> Self {
        match strat {
            contextra_types::RetrievalStrategy::Vector => RetrievalStrategy::Vector,
            contextra_types::RetrievalStrategy::Text => RetrievalStrategy::Text,
            contextra_types::RetrievalStrategy::Graph => RetrievalStrategy::Graph,
            contextra_types::RetrievalStrategy::Hybrid => RetrievalStrategy::Hybrid,
            contextra_types::RetrievalStrategy::Global { .. } => RetrievalStrategy::Global,
        }
    }
}

// --- Fehlerkonvertierungen nach ContextraError (aus src/error.rs) ---

impl From<AdaptError> for contextra_types::ContextraError {
    fn from(err: AdaptError) -> Self {
        match err {
            AdaptError::DimensionMismatch { expected, actual } => {
                contextra_types::ContextraError::InvalidInput(format!(
                    "Dimension mismatch: expected {expected}, got {actual}"
                ))
            }
            AdaptError::InvalidConfig(msg) => contextra_types::ContextraError::InvalidInput(format!(
                "Invalid configuration: {msg}"
            )),
            AdaptError::NonFinite(msg) => {
                contextra_types::ContextraError::Internal(format!("Non-finite encountered: {msg}"))
            }
            AdaptError::PrecisionMatrixDriftDetected {
                updates_since_reset,
                denominator,
            } => contextra_types::ContextraError::Internal(format!(
                "Sherman-Morrison precision matrix drifted at step {updates_since_reset}: denom={denominator}"
            )),
            AdaptError::ZeroPropensityViolation(msg) => {
                contextra_types::ContextraError::InvalidInput(format!(
                    "Zero propensity violation: {msg}"
                ))
            }
            AdaptError::InvalidInput(msg) => contextra_types::ContextraError::InvalidInput(msg),
        }
    }
}

// --- Fehlerkonvertierungen für Bandits (aus src/bandit.rs, flow_thompson.rs, rie_greedy.rs) ---

impl From<BanditError> for contextra_types::ContextraError {
    fn from(err: BanditError) -> Self {
        match err {
            BanditError::DimensionMismatch { expected, actual } => {
                contextra_types::ContextraError::InvalidInput(format!(
                    "Dimension mismatch: expected {expected}, actual {actual}"
                ))
            }
            BanditError::InvalidConfig(msg) => {
                contextra_types::ContextraError::InvalidInput(format!("Invalid config: {msg}"))
            }
            BanditError::NonFiniteVariance(val) => {
                contextra_types::ContextraError::Internal(format!("Non-finite variance: {val}"))
            }
            BanditError::PrecisionMatrixDriftDetected {
                updates_since_reset,
                denominator,
            } => contextra_types::ContextraError::Internal(format!(
                "Precision matrix drifted at step {updates_since_reset}: denominator={denominator}"
            )),
        }
    }
}

impl From<FcTsError> for contextra_types::ContextraError {
    fn from(err: FcTsError) -> Self {
        match err {
            FcTsError::DimensionMismatch { expected, actual } => {
                contextra_types::ContextraError::InvalidInput(format!(
                    "Dimension mismatch: expected {expected}, actual {actual}"
                ))
            }
            FcTsError::NonFinite => contextra_types::ContextraError::InvalidInput(
                "Non-finite encountered".to_string(),
            ),
            FcTsError::InvalidConfig(msg) => contextra_types::ContextraError::InvalidInput(
                format!("Invalid configuration: {msg}"),
            ),
        }
    }
}

impl From<OffPolicyError> for contextra_types::ContextraError {
    fn from(err: OffPolicyError) -> Self {
        contextra_types::ContextraError::InvalidInput(err.to_string())
    }
}

impl From<RieGreedyError> for contextra_types::ContextraError {
    fn from(err: RieGreedyError) -> Self {
        match err {
            RieGreedyError::DimensionMismatch { expected, actual } => {
                contextra_types::ContextraError::InvalidInput(format!(
                    "Embedding dimension mismatch: expected {expected}, actual {actual}"
                ))
            }
            RieGreedyError::NonFinite => contextra_types::ContextraError::InvalidInput(
                "Non-finite numerical value encountered".to_string(),
            ),
            RieGreedyError::InvalidConfig(msg) => contextra_types::ContextraError::InvalidInput(
                format!("Invalid configuration: {msg}"),
            ),
        }
    }
}

// --- Clock Port Implementierung (aus src/clock.rs) ---

impl contextra_adapt::Clock for contextra_ports::SystemClock {
    fn now_unix_nanos(&self) -> u64 {
        contextra_ports::Clock::now_unix_nanos(self)
    }

    fn monotonic_nanos(&self) -> u64 {
        contextra_ports::Clock::monotonic_nanos(self)
    }
}

// --- TenantId & Helper Hilfskonvertierungen ---

/// Leitet einen PRNG-Seed aus TenantId ab.
pub fn derive_bandit_seed(tenant_id: &contextra_types::TenantId, base_seed: u64) -> u64 {
    contextra_adapt::bandit::SketchMatrix::derive_seed_u64(tenant_id.as_u64(), base_seed)
}

/// Erstellt eine `DiagonalApproximationBandit`-Instanz aus `contextra_types::RetrievalStrategy`-Paaren.
pub fn diagonal_bandit_from_types(
    pairs: impl IntoIterator<Item = (contextra_types::RetrievalStrategy, f32)>,
) -> DiagonalApproximationBandit {
    DiagonalApproximationBandit::from_pairs(pairs.into_iter().map(|(s, p)| (s.into(), p)))
}

/// Erstellt eine `FcTsSamplingDistribution`-Instanz aus `contextra_types::RetrievalStrategy`-Paaren.
pub fn fcts_distribution_from_types(
    pairs: impl IntoIterator<Item = (contextra_types::RetrievalStrategy, f32)>,
) -> FcTsSamplingDistribution {
    FcTsSamplingDistribution::from_pairs(pairs.into_iter().map(|(s, p)| (s.into(), p)))
}
