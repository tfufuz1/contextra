//! Purge-Modul (SPEC §11.4).
//!
//! Steuert das Purge-Fenster, die Schichtreihenfolge und die Plattenreserve.
pub mod window;
pub mod order;
pub mod reserve;
pub mod cx;

pub use window::{run_window, WindowOutcome};
pub use cx::PurgeCx;
pub use order::{LayerPurgeOrder, PurgePriority};
pub use reserve::{check_disk_reserve, ReserveCheck};
