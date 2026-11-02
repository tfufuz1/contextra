//! Löschkern: Saga, Gate, Journal, Tresor, Messung, signierter Bericht.
//!
//! Ring 3. Erlaubte Abhängigkeiten: `types`, `ports`, `crypto` (AR-3).
//! Spezifikation: SPEC 4.3.0, Abschnitte 11, 12, 13; Anhang F.3, F.9, F.10.

#![forbid(unsafe_code)]

pub mod error;
pub mod gate;
pub mod journal;
pub mod ledger;
pub mod measure;
pub mod purge;
pub mod report;
pub mod restore;
pub mod saga;
pub mod scope;
pub mod vault;

pub use error::ErasureError;
