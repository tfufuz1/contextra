//! Löschkern: Saga, Gate, Journal, Tresor, Messung, signierter Bericht.
//!
//! Ring 3 (SPEC §5.2). Abhängigkeitsregel AR-3 (SPEC §5.3): Erlaubte Abhängigkeiten
//! sind ausschließlich `contextra-types`, `contextra-ports` und `contextra-crypto`.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(not(test), deny(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod error;
pub mod io;
pub mod vault;
pub mod gate;
pub mod journal;
pub mod scope;
pub mod saga;
pub mod purge;
pub mod measure;
pub mod report;
pub mod ledger;
pub mod restore;
