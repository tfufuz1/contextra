//! Löschkern: Saga, Gate, Journal, Tresor, Messung, signierter Bericht.
//!
//! Ring 3 (SPEC §5.1, §5.2, §11). Abhängigkeitsregel AR-3 (SPEC §5.3): Erlaubte Abhängigkeiten
//! sind ausschließlich `contextra-types`, `contextra-ports` und `contextra-crypto`.

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

pub mod backend;
pub mod crypto_local;
#[cfg(feature = "signing")]
pub mod deletion_proof;
pub mod erasure_domain;
pub mod error;
pub mod gate;
pub mod io;
pub mod journal;
pub mod ledger;
pub mod measure;
pub mod purge;
pub mod replace;
pub mod report;
pub mod restore;
pub mod restricted;
pub mod saga;
pub mod scope;
pub mod supersede;
#[cfg(test)]
pub mod test_mocks;
pub mod types_local;
pub mod vault;

#[cfg(feature = "signing")]
pub mod ed25519_proof;
#[cfg(feature = "vault")]
pub mod kv_shredding;
#[cfg(feature = "signing")]
pub mod orchestrator;
#[cfg(feature = "signing")]
pub mod revocation_log;
