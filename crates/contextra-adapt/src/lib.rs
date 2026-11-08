//! Adaptive controllers, PID regulators, bandits, and Lyapunov drift watchers for Contextra.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![allow(
    clippy::indexing_slicing,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::disallowed_types
)]
#![cfg_attr(
    test,
    allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::panic,
        clippy::indexing_slicing,
        clippy::manual_range_contains,
        clippy::same_item_push
    )
)]

// --- Lokale Grundabstraktionen (B3, B4, B11) ---

pub mod clock;
pub mod error;
pub mod rng;
pub mod strategy;

pub use clock::{Clock, SystemClock};
pub use error::{AdaptError, Result};
pub use rng::{Rng, SeededRng};
pub use strategy::RetrievalStrategy;

// --- Baukasten-Zielmodule ---

/// Regelungstechnik & Latenzsteuerung (PID, Latency Controller, Homeostat, Lyapunov).
pub mod control {
    pub use crate::homeostat::*;
    pub use crate::lyapunov::*;
    pub use crate::pid::*;
    pub use crate::pid_latency_controller::*;
}

/// Bandit-Algorithmen (LinUCB, Sketched-Bandits, Flow-Corrected Thompson Sampling, RIE-Greedy).
pub mod bandit_algorithms {
    #[cfg(feature = "bandit-routing")]
    pub use crate::bandit::*;
    #[cfg(feature = "flow-corrected-thompson")]
    pub use crate::flow_thompson::*;
    #[cfg(feature = "rie-greedy-personalization")]
    pub use crate::rie_greedy::*;
}

/// Policy-Evaluation & Shadow Execution (Off-Policy IPS, Shadow Mode).
pub mod policy {
    pub use crate::offpolicy::*;
    pub use crate::shadow_mode::*;
}

/// Konzept- und Verteilungs-Drift (Catoni M-Estimator, Ensemble Drift Watcher, Policy Bridge).
pub mod drift_detection {
    pub use crate::drift::*;
}

/// Zeitliches und Score-basiertes Abklingen (Adaptive Decay Controller).
pub mod decay {
    pub use crate::decay_controller::*;
}

// --- Flache Modul-Deklarationen (für bestehende Pfade & Re-Exporte) ---

#[cfg(feature = "bandit-routing")]
pub mod bandit;
pub mod decay_controller;
pub mod drift;
#[cfg(feature = "flow-corrected-thompson")]
pub mod flow_thompson;
pub mod homeostat;
pub mod lyapunov;
#[deprecated(note = "Verwende crate::offpolicy statt des veralteten crate::off_policy")]
pub mod off_policy;
pub mod offpolicy;
pub mod pid;
pub mod pid_latency_controller;
#[cfg(feature = "rie-greedy-personalization")]
pub mod rie_greedy;
pub mod shadow_mode;

#[cfg(feature = "bandit-routing")]
pub use bandit::*;
pub use decay_controller::*;
pub use drift::*;
#[cfg(feature = "flow-corrected-thompson")]
pub use flow_thompson::*;
pub use homeostat::*;
pub use lyapunov::*;
#[allow(deprecated)]
pub use off_policy::*;
pub use pid::*;
pub use pid_latency_controller::*;
#[cfg(feature = "rie-greedy-personalization")]
pub use rie_greedy::{RieGreedyError, RieGreedyProfile};
pub use shadow_mode::*;
