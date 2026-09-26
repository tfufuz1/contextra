//! Adaptive controllers, PID regulators, bandits, and Lyapunov drift watchers for Contextra.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

#[cfg(feature = "bandit-routing")]
pub mod bandit;
pub mod decay_controller;
pub mod drift;
#[cfg(feature = "flow-corrected-thompson")]
pub mod flow_thompson;
pub mod homeostat;
#[cfg(feature = "rie-greedy-personalization")]
pub mod rie_greedy;
pub mod lyapunov;
pub mod off_policy;
pub mod offpolicy;
pub mod pid;
pub mod pid_latency_controller;
pub mod shadow_mode;

#[cfg(feature = "bandit-routing")]
pub use bandit::*;
pub use decay_controller::*;
pub use drift::*;
#[cfg(feature = "flow-corrected-thompson")]
pub use flow_thompson::{
    ring3_background_task_token, FcTsArmSet, FcTsConfig, FcTsError, FcTsRng,
    FlowCorrectedThompsonBandit, Ring3Token, SplitMix64,
};
pub use homeostat::*;
#[cfg(feature = "rie-greedy-personalization")]
pub use rie_greedy::{RieGreedyError, RieGreedyProfile};
pub use lyapunov::*;
pub use off_policy::*;
pub use pid::*;
pub use pid_latency_controller::*;
pub use shadow_mode::*;
