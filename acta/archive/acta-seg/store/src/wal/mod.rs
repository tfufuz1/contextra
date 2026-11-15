//! Write-Ahead Log (WAL) for durability and crash recovery with HMAC chaining.
//!
//! # Architektur & Invarianten
//! - INV-FSYNC-1: Nach Fehlern wird das WAL-Handle fail-closed vergiftet (`PoisonCause`).
//! - INV-STORE-1: Alle Mutationen werden sequentiell und mit HMAC-Integritätsprüfung persistiert.
//! - Lock-Ordnung: `commit_mutex` -> `wal.truncate_lock` -> `memtable_shard_lock` -> `sstable_manifest_lock`.

// Re-exports an oberster Stelle
pub use acta_crypto::crypto::KeyManager;

pub mod config;
pub mod core;
pub mod encode;
pub mod flusher;
pub mod fs;
pub mod hmac;
pub mod io;
pub mod poison;
pub mod replay;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod open_heal_tests;

// Re-exporte der Schnittstellen
pub use config::WalConfig;
pub use core::*;
pub use encode::*;
pub use flusher::*;
pub use hmac::*;
pub use poison::PoisonCause;
pub use replay::*;
