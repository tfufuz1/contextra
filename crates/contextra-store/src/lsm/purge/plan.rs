// FILE-CONTEXT
// STAND: 2026-10-12
// ZWECK: Purge-Phasen Zustandsautomat und Idempotenz-Spezifikation (Spezifikation C5.1/C5.6).
//
//! # Purge Phase State Machine and Idempotency
//!
//! | Phase | Idempotenz-Strategie | Persistierter Zustand |
//! |---|---|---|
//! | `Admitted` | Re-executabel; Intent existiert bereits | `purge.intent` mit Phase `Admitted` |
//! | `Flush` | Re-executabel; `storage.flush()` ist idempotent | `purge.intent` mit Phase `Flush` nach Flush |
//! | `PurgeCompaction` | Idempotent via Manifest Atomizität & Versioning | `purge.intent` mit Phase `PurgeCompaction` |
//! | `Dispose` | Re-executabel; `PendingDeleteJournal` filtert fehlende Dateien | `pending_delete.journal` & Intent `Dispose` |
//! | `WalRetire` | Callback idempotent auf R0-Clog | `purge.intent` mit Phase `WalRetire` |
//! | `Witness` | Zero-Knowledge Proof Erzeugung idempotent | `purge.intent` mit Phase `Witness` |
//! | `Finalize` | Proof-Chain Update idempotent | `purge.intent` mit Phase `Finalize` |
//! | `Release` | Intent Löschung schließt Purge-Zyklus ab | `purge.intent` gelöscht |

use serde::{Deserialize, Serialize};

/// Phasen des Purge-Zustandsautomaten (C5.1 / C5.6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PurgePhase {
    /// Initial admission and logging of purge intent.
    Admitted,
    /// Memtable flush to stable SSTables.
    Flush,
    /// Purge-Compaction execution and manifest update.
    PurgeCompaction,
    /// Disposal of obsolete files via PendingDeleteJournal and scrubbing.
    Dispose,
    /// WAL log retirement callback execution.
    WalRetire,
    /// Witness deletion proof issuance.
    Witness,
    /// Finalization of deletion proof chain.
    Finalize,
    /// Release of erasure fence and cleanup of intent log.
    Release,

    // Backward-compatibility aliases for legacy P0..P7 code paths
    #[serde(alias = "P0Init")]
    P0Init,
    #[serde(alias = "P1Intent")]
    P1Intent,
    #[serde(alias = "P2FenceAcquired")]
    P2FenceAcquired,
    #[serde(alias = "P3Flush")]
    P3Flush,
    #[serde(alias = "P4PurgeCompaction")]
    P4PurgeCompaction,
    #[serde(alias = "P5Dispose")]
    P5Dispose,
    #[serde(alias = "P6WalCallback")]
    P6WalCallback,
    #[serde(alias = "P7Complete")]
    P7Complete,
}

impl PurgePhase {
    /// Returns `true` if this phase is completed relative to `target`.
    #[must_use]
    pub fn is_at_least(self, target: Self) -> bool {
        self.ordinal() >= target.ordinal()
    }

    /// Converts phase variant to u8 byte representation for compact binary serialization.
    #[must_use]
    pub fn to_u8(self) -> u8 {
        match self {
            Self::Admitted | Self::P0Init | Self::P1Intent => 0,
            Self::Flush | Self::P2FenceAcquired | Self::P3Flush => 1,
            Self::PurgeCompaction | Self::P4PurgeCompaction => 2,
            Self::Dispose | Self::P5Dispose => 3,
            Self::WalRetire | Self::P6WalCallback => 4,
            Self::Witness => 5,
            Self::Finalize => 6,
            Self::Release | Self::P7Complete => 7,
        }
    }

    /// Deserializes phase from u8 byte representation.
    #[must_use]
    pub fn from_u8(val: u8) -> Option<Self> {
        match val {
            0 => Some(Self::Admitted),
            1 => Some(Self::Flush),
            2 => Some(Self::PurgeCompaction),
            3 => Some(Self::Dispose),
            4 => Some(Self::WalRetire),
            5 => Some(Self::Witness),
            6 => Some(Self::Finalize),
            7 => Some(Self::Release),
            _ => None,
        }
    }

    fn ordinal(self) -> u8 {
        self.to_u8()
    }
}
