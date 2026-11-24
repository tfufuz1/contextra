//! # contextra-router
//!
//! Intelligentes Routing von Anfragen an geeignete Small Language Models (SLM).
//! Bewertet Modell-Profile (z. B. Token-Limits, Domänen-Affinitäten), kalibriert Konfidenzwerte
//! über konforme Vorhersagen (Conformal Prediction) und steuert Multi-Armed-Bandit-Verfahren (LinUCB / Thompson Sampling).
//!
//! **Abhängigkeiten:**
//! - Upstream: `contextra-types`, `contextra-ports`, `contextra-adapt`, `contextra-wire` (optional).
//! - Downstream: MCP-Server (`contextra-mcp`).
//!
//! **Hauptkomponenten:**
//! - [`RouterEngine`]: Primärer Routing-Endpunkt zur Entscheidung für das beste Modell.
//! - [`RouterEngineBuilder`]: Ergonomischer Builder zur Konstruktion von `RouterEngine`.
//! - [`SlmProfile`]: Spezifikation der Fähigkeiten und Begrenzungen eines Modells.
//! - [`ConformalCalibrator`]: Statistische Fehlerüberwachung und Kalibrierung.
//! - [`ArmRegistry`]: Zuordnung von Modellprofilen zu Bandit-Armen.
//! - [`dispatch_to_slm`]: Ausführung des eigentlichen SLM-Aufrufs.
//!
//! **Entkopplung:**
//! Bandit-, Lyapunov- und Off-Policy-Typen leben in `contextra-adapt` und werden dort direkt
//! importiert. Dieser Crate re-exportiert sie **nicht** mehr — Konsumenten zeigen direkt auf
//! `contextra_adapt::{BanditProfileState, LyapunovDriftWatcher, OffPolicyEvaluator, …}`.

// ANCHOR[contextra-router:LIB-001]
#![deny(dead_code)]
#![forbid(unsafe_code)]

pub mod arm_registry;
pub mod builder;
#[cfg(feature = "subprocess-dispatch")]
pub mod dispatch;
pub mod outcome;
pub mod ports_local;
pub mod profile;
pub mod router;
pub mod seam;
pub mod serde_helpers;

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

// ANCHOR[contextra-router:LIB-002]
pub use arm_registry::{ArmRegistry, ArmRegistryError};
pub use builder::RouterEngineBuilder;
#[cfg(feature = "subprocess-dispatch")]
pub use dispatch::dispatch_to_slm;
pub use outcome::{DecisionId, DecisionIdGenerator, RoutingOutcome};
#[allow(deprecated)]
pub use profile::{ConformalCalibrator, ProfileCalibrator, SlmProfile};
pub use router::{DefaultRouterEngine, RouterEngine, RoutingDecision};
pub use seam::{CalibrationSource, NoCalibrationSource, StdFsCalibrationSource, VfsCalibrationSource};

#[cfg(feature = "flow-corrected-thompson")]
pub use fc_ts_dispatch::{deterministic_fc_ts_rng, select_profile_fc_ts, FcTsDispatchError};
#[cfg(any(feature = "bandit-routing", feature = "flow-corrected-thompson"))]
pub use routing_strategy::RoutingStrategy;
#[cfg(feature = "cloud-egress-guard")]
pub use transport::Transport;
