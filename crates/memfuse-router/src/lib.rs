#![forbid(unsafe_code)]

pub mod dispatch;
pub mod outcome;
pub mod profile;
pub mod router;
pub mod serde_helpers;

#[deprecated(
    since = "0.1.0",
    note = "Moved to memfuse_adapt as part of Ring-Modell Phase 1b"
)]
pub use memfuse_adapt::bandit;
#[deprecated(
    since = "0.1.0",
    note = "Moved to memfuse_adapt as part of Ring-Modell Phase 1b"
)]
pub use memfuse_adapt::lyapunov;
#[deprecated(
    since = "0.1.0",
    note = "Moved to memfuse_adapt as part of Ring-Modell Phase 1b"
)]
pub use memfuse_adapt::offpolicy;

#[cfg(feature = "cloud-egress-guard")]
pub mod guarded_payload;
#[cfg(feature = "bandit-routing")]
pub mod routing_strategy;
#[cfg(feature = "cloud-egress-guard")]
pub mod transport;

#[cfg(all(test, feature = "bandit-routing"))]
mod bandit_regret_tests;
#[cfg(test)]
mod tests;

pub use dispatch::dispatch_to_slm;
pub use memfuse_adapt::{DriftReason, LyapunovDriftWatcher, LyapunovResult};
pub use memfuse_adapt::{OffPolicyEvaluator, OffPolicyStats};
pub use outcome::{DecisionId, RoutingOutcome};
pub use profile::SlmProfile;
pub use router::{DefaultRouterEngine, RouterEngine, RoutingDecision};

#[cfg(feature = "cloud-egress-guard")]
pub use guarded_payload::{GuardedPayload, Sanitized, Unsanitized};
#[cfg(feature = "bandit-routing")]
pub use memfuse_adapt::{BanditError, BanditImplementation, BanditPolicy, BanditProfileState};
#[cfg(feature = "bandit-routing")]
pub use routing_strategy::RoutingStrategy;
#[cfg(feature = "cloud-egress-guard")]
pub use transport::Transport;

// REVIEW-PASS[1/2] (TS: 2026-09-16T16:17:21Z) (SESSION: cec8b8e9) (PRÜFER-KONTEXT: FRESH)
