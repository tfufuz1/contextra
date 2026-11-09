//! Handoff-Archiv: Historische Erasure-Typen und -Ports vor Typfamilien-Konsolidierung (PORTS-1).
//!
//! Gemäß Soll-Zustand §3 und Prinzip 8 existiert für das Konzept `LayerWitness`
//! genau eine normative Wahrheit in `contextra-ports`: `crate::scanner::LayerWitness` (SPEC §12.7.4).
//!
//! Die hier dokumentierte Vorläuferdefinition aus `erasure.rs` wurde archiviert,
//! um lückenlose Traceability und Parität sicherzustellen.

use crate::BoxFuture;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Archivierte historische LayerWitness-Definition aus erasure.rs
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LegacyLayerWitness {
    /// Digest or commitment over the verified non-presence.
    witness_hash: [u8; 32],
}

impl LegacyLayerWitness {
    /// Creates an empty/zeroed witness.
    pub const fn empty() -> Self {
        Self {
            witness_hash: [0u8; 32],
        }
    }

    /// Creates a witness from a hash.
    pub const fn from_hash(witness_hash: [u8; 32]) -> Self {
        Self { witness_hash }
    }

    /// Returns the witness digest.
    pub fn witness_hash(&self) -> &[u8; 32] {
        &self.witness_hash
    }
}
