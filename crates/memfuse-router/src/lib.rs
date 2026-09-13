#![forbid(unsafe_code)]

pub mod dispatch;
pub mod lyapunov;
pub mod outcome;
pub mod profile;
pub mod router;
pub mod serde_helpers;

#[cfg(feature = "cloud-egress-guard")]
pub mod transport;
#[cfg(feature = "bandit-routing")]
pub mod routing_strategy;
#[cfg(feature = "cloud-egress-guard")]
pub mod guarded_payload;
#[cfg(feature = "bandit-routing")]
pub mod bandit;

#[cfg(all(test, feature = "bandit-routing"))]
mod bandit_regret_tests;
#[cfg(test)]
mod tests;

pub use dispatch::dispatch_to_slm;
pub use lyapunov::{DriftReason, LyapunovDriftWatcher, LyapunovResult};
pub use outcome::{DecisionId, RoutingOutcome};
pub use profile::SlmProfile;
pub use router::{RouterEngine, RoutingDecision};

#[cfg(feature = "cloud-egress-guard")]
pub use transport::Transport;
#[cfg(feature = "bandit-routing")]
pub use routing_strategy::RoutingStrategy;
#[cfg(feature = "cloud-egress-guard")]
pub use guarded_payload::{GuardedPayload, Sanitized, Unsanitized};
#[cfg(feature = "bandit-routing")]
pub use bandit::{BanditImplementation, BanditProfileState};
