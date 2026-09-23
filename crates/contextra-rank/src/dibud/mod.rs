//! DiBud (Dynamic Budgeted RRF) fusion state machine, drivers, and types.

pub mod driver;
pub mod state;
pub mod types;

pub use driver::{fuse_exact_prefix, fuse_exact_prefix_async};
pub use state::{DiBudFusionState, DiBudStep};
pub use types::{BudgetedChannel, DiBudOutcome, FusionBudget};
