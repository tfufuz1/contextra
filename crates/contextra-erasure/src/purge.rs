//! Purge-Modul (SPEC §11.4).
//!
//! Steuert das Purge-Fenster, die Schichtreihenfolge und die Plattenreserve.

pub mod cx;
pub mod order;
pub mod reserve;
pub mod window;

pub use cx::PurgeCx;
pub use order::{LayerPurgeOrder, PurgePriority};
pub use reserve::{check_disk_reserve, ReserveCheck};
pub use window::{run_window, WindowOutcome};
