#![forbid(unsafe_code)]

pub mod arm_registry;
pub mod dispatch;
pub mod outcome;
pub mod ports_local;
pub mod profile;
pub mod router;
pub mod serde_helpers;

#[deprecated(
    since = "0.1.0",
    note = "Moved to contextra_adapt as part of Ring-Modell Phase 1b"
)]
pub use contextra_adapt::bandit;
#[deprecated(
    since = "0.1.0",
    note = "Moved to contextra_adapt as part of Ring-Modell Phase 1b"
)]
pub use contextra_adapt::lyapunov;
#[deprecated(
    since = "0.1.0",
    note = "Moved to contextra_adapt as part of Ring-Modell Phase 1b"
)]
pub use contextra_adapt::offpolicy;

#[deprecated(
    since = "0.1.0",
    note = "Moved to contextra_privacy as part of Ring-Modell Phase 1b"
)]
#[cfg(feature = "cloud-egress-guard")]
pub use contextra_privacy::guarded_payload;

#[cfg(feature = "flow-corrected-thompson")]
pub mod fc_ts_dispatch;
#[cfg(any(feature = "bandit-routing", feature = "flow-corrected-thompson"))]
pub mod routing_strategy;
#[cfg(feature = "cloud-egress-guard")]
pub mod transport;

#[cfg(all(test, feature = "bandit-routing"))]
mod bandit_regret_tests;
#[cfg(test)]
mod tests;

pub use arm_registry::{ArmRegistry, ArmRegistryError};
pub use contextra_adapt::{DriftReason, LyapunovDriftWatcher, LyapunovResult};
pub use contextra_adapt::{OffPolicyEvaluator, OffPolicyStats};
pub use dispatch::dispatch_to_slm;
pub use outcome::{DecisionId, DecisionIdGenerator, RoutingOutcome};
pub use profile::SlmProfile;
pub use router::{DefaultRouterEngine, RouterEngine, RoutingDecision};

#[cfg(feature = "bandit-routing")]
pub use contextra_adapt::{BanditImplementation, BanditPolicy, BanditProfileState};
#[deprecated(
    since = "0.1.0",
    note = "Moved to contextra_privacy as part of Ring-Modell Phase 1b"
)]
#[cfg(feature = "cloud-egress-guard")]
pub use contextra_privacy::guarded_payload::{GuardedPayload, Sanitized, Unsanitized};
#[cfg(feature = "flow-corrected-thompson")]
pub use fc_ts_dispatch::{deterministic_fc_ts_rng, select_profile_fc_ts, FcTsDispatchError};
#[cfg(any(feature = "bandit-routing", feature = "flow-corrected-thompson"))]
pub use routing_strategy::RoutingStrategy;
#[cfg(feature = "cloud-egress-guard")]
pub use transport::Transport;

// REVIEW-PASS[1/2] (TS: 2026-09-16T16:17:21Z) (SESSION: cec8b8e9) (PRÜFER-KONTEXT: FRESH)
