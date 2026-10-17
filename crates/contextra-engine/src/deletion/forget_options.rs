//! Forget options and deletion outcome handles (§C9, WP-1.1).

//!
//! # Observation vs Guarantee (INV-D10)
//!
//! Statements about system deletion assurance levels (S0..S4) reflect observed
//! software-level erasure operations. Physical storage media characteristics
//! (e.g., SSD wear-leveling remanence, controller translation layers, hardware flash blocks)
//! remain outside any software guarantee unless cryptographic erasure (S4) with keyring destruction
//! is available.
//!
//! Process-level hardening (`harden_runtime`, e.g., setting `RLIMIT_CORE=0`) is intentionally
//! omitted from this layer as runtime process modification requires baseline ADR approval.

#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::time::Duration;

use contextra_types::{ContextraError, Result};
use serde::{Deserialize, Serialize};

// Re-export WitAssurance from contextra-crypto as Assurance per C9.
pub use contextra_crypto::deletion_proof::claim_text::WitAssurance;

/// Alias for [`WitAssurance`].
pub type Assurance = WitAssurance;

/// Purge mode specifying physical cleanup requirements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PurgeMode {
    /// No immediate physical purge (logical tombstone committal only).
    None,
    /// Physical purge of SSTables and WAL segments.
    Purge,
    /// Physical purge with media overwrite/scrubbing.
    PurgeWithScrub,
}

/// Wait policy for synchronous or asynchronous deletion completion.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Wait {
    /// Non-blocking, return immediately after triggering operation.
    No,
    /// Block until completion or specified duration timeout.
    Until(Duration),
    /// Block indefinitely until completion.
    Forever,
}

/// Verification depth for deletion proof validation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum VerifyDepth {
    /// Validate deletion footprint only.
    Footprint,
    /// Perform full sweep reverse index and storage verification.
    FullSweep,
}

/// Options controlling deletion execution, assurance, and verification (§C9).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ForgetOptions {
    /// Requested assurance level (default: S1 per ADR D4/ADR-11).
    pub assurance: WitAssurance,
    /// Requested physical purge mode (default: None).
    pub purge: PurgeMode,
    /// Target path for purge operations when required (default: None).
    pub purge_path: Option<PathBuf>,
    /// Wait mode for completion (default: No).
    pub wait: Wait,
    /// Verification depth (default: Footprint).
    pub verify_depth: VerifyDepth,
}

impl Default for ForgetOptions {
    fn default() -> Self {
        Self {
            assurance: WitAssurance::S1,
            purge: PurgeMode::None,
            purge_path: None,
            wait: Wait::No,
            verify_depth: VerifyDepth::Footprint,
        }
    }
}

impl ForgetOptions {
    /// Creates a new `ForgetOptions` with standard default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the target assurance level.
    #[must_use]
    pub fn with_assurance(mut self, assurance: WitAssurance) -> Self {
        self.assurance = assurance;
        self
    }

    /// Sets the physical purge mode.
    #[must_use]
    pub fn with_purge(mut self, purge: PurgeMode) -> Self {
        self.purge = purge;
        self
    }

    /// Sets the purge target path.
    #[must_use]
    pub fn with_purge_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.purge_path = Some(path.into());
        self
    }

    /// Sets the wait policy.
    #[must_use]
    pub fn with_wait(mut self, wait: Wait) -> Self {
        self.wait = wait;
        self
    }

    /// Sets the verification depth.
    #[must_use]
    pub fn with_verify_depth(mut self, verify_depth: VerifyDepth) -> Self {
        self.verify_depth = verify_depth;
        self
    }

    /// Validates options configuration consistency.
    ///
    /// # Errors
    ///
    /// Returns [`ContextraError::InvalidInput`] if:
    /// - Assurance level S4 is requested (keyring integration pending).
    /// - `Wait::Until(0)` duration is requested.
    /// - Assurance level >= S2 is requested without physical purge mode.
    /// - `PurgeMode::PurgeWithScrub` is requested without a purge path.
    pub fn validate(&self) -> Result<()> {
        if self.assurance == WitAssurance::S4 {
            return Err(ContextraError::invalid_input(
                "S4 requires keyring, not available",
            ));
        }

        if matches!(self.wait, Wait::Until(d) if d.is_zero()) {
            return Err(ContextraError::invalid_input(
                "Wait::Until(0) duration is invalid",
            ));
        }

        if self.assurance >= WitAssurance::S2 && self.purge == PurgeMode::None {
            return Err(ContextraError::invalid_input(
                "Assurance >= S2 requires a purge mode",
            ));
        }

        if self.purge == PurgeMode::PurgeWithScrub && self.purge_path.is_none() {
            return Err(ContextraError::invalid_input(
                "PurgeWithScrub requires a purge path",
            ));
        }

        Ok(())
    }
}

/// Logical deletion proof handle.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FonLogicalHandle {
    /// Unique proof identifier (16-byte truncated prefix).
    pub proof_id: [u8; 16],
    /// File system path where the proof record is stored.
    pub proof_path: PathBuf,
    /// Full 32-byte proof identifier for exact stage linking.
    pub full_proof_id: [u8; 32],
}

impl std::fmt::Debug for FonLogicalHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FonLogicalHandle")
            .field("proof_id", &self.proof_id)
            .field("full_proof_id", &self.full_proof_id)
            .field("proof_path", &"[REDACTED_PATH]")
            .finish()
    }
}

/// Handle tracking physical purge execution status.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PurgeHandle {
    /// Physical purge is queued or in progress.
    Pending {
        /// Identifier for the pending purge job.
        purge_id: u64,
    },
    /// Physical purge finalized and verified.
    Final(FonLogicalHandle),
    /// Physical purge blocked requiring intervention.
    Blocked {
        /// Reason for physical purge blockage.
        reason: String,
        /// Recommended retry or recovery action.
        retry_hint: String,
    },
}

impl std::fmt::Debug for PurgeHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Pending { purge_id } => f
                .debug_struct("PurgeHandle::Pending")
                .field("purge_id", purge_id)
                .finish(),
            Self::Final(handle) => f.debug_tuple("PurgeHandle::Final").field(handle).finish(),
            Self::Blocked { reason, retry_hint } => f
                .debug_struct("PurgeHandle::Blocked")
                .field("reason", reason)
                .field("retry_hint", retry_hint)
                .finish(),
        }
    }
}

/// Complete outcome of a forget operation (§C9).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForgetOutcome {
    /// Logical deletion handle / proof reference.
    pub logical: FonLogicalHandle,
    /// Physical purge execution status handle, if physical purge was requested.
    pub physical: Option<PurgeHandle>,
}
