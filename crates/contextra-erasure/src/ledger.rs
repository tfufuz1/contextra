//! Ledger-Modul (SPEC §11.8).
//!
//! Ledger-Einträge, Hash-Kette und In-Memory/Datei-Sink außerhalb des Datenverzeichnisses.
pub mod entry;
pub mod sink;

pub use entry::{LedgerEntry, GENESIS_HASH};
pub use sink::InMemoryLedger;
