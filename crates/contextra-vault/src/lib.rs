#![forbid(unsafe_code)]
#![deny(missing_docs)]
//! # contextra-vault
//!
//! Ephemerer, speichergesicherter Chunk- und Kontext-Tresor (Ebene 1, Kern-Closure `{contextra-sys}`).
//!
//! Dieses Crate implementiert einen flüchtigen Zwischenspeicher für sensible flüchtige
//! In-Memory-Daten, basierend auf `contextra-sys::vault::LockedRegions`.

pub mod error;
pub mod vault;

#[cfg(feature = "workspace")]
pub mod compat;

pub use error::VaultError;
pub use vault::{
    CommitReceipt, SignalModality, VaultChunk, VaultChunkMetadata, VaultConfig, VaultDocId,
    VaultTenantId, VaultTxId, VolatileContextVault, VolatileDrainReceipt, *,
};
