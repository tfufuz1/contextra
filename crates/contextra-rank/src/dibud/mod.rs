//! # DiBud: Discrete Budgeted Anytime Streaming Fusion (`dibud`)
//!
//! **Ring:** Ring 0 (Ranking & Kalibrierung)
//!
//! DiBud (Discrete Budgeted RRF) implementiert eine inkrementelle Streaming-Rangfusion,
//! die jederzeit unter Einhaltung harter Ressourcenbeschränkungen (Zeit- und Kanalbudgets)
//! abbrechbar ist und dabei ein zertifiziertes Präfix (Certified Prefix) garantiert.

// ANCHOR[contextra-rank:DIBUD-MOD-001]
pub mod driver;
pub mod state;
pub mod types;

pub use driver::{fuse_exact_prefix, fuse_exact_prefix_async};
pub use state::{DiBudFusionState, DiBudStep};
pub use types::{BudgetedChannel, DiBudOutcome, FusionBudget};
