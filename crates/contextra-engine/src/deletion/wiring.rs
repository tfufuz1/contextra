//! Facade wiring entry point for deletion subsystem (§7).

#![forbid(unsafe_code)]

use std::path::PathBuf;
use std::sync::Arc;

use contextra_store::LsmStorage;
use contextra_types::Result;

use super::proof_issue::DpiDeletionRuntime;
use super::purge_driver::{DpiPurgeDriver, DpiPurgeDriverConfig};

/// Execution ring mode for deletion proof generation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RingMode {
    /// Sovereign ring requiring strict cryptographic deletion proofs and proof keys (fail-closed).
    #[default]
    Sovereign,
    /// Fast ring where proofs are skipped with explicit (non-silent) diagnostic tracing.
    Fast,
}

/// Dependencies required to initialize [`DeletionWiring`].
pub struct DeletionWiringDeps {
    /// Active LSM storage handle.
    pub storage: Arc<LsmStorage>,
    /// Database directory path for proof persistence.
    pub db_dir: PathBuf,
    /// Ed25519 signer key configuration path (optional).
    pub signer_cfg_path: Option<PathBuf>,
    /// Unique 16-byte database instance identifier.
    pub db_instance_id: [u8; 16],
    /// Ring mode controlling key enforcement.
    pub ring_mode: RingMode,
}

impl std::fmt::Debug for DeletionWiringDeps {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DeletionWiringDeps")
            .field("db_dir", &self.db_dir)
            .field("signer_cfg_path", &self.signer_cfg_path)
            .field("db_instance_id", &self.db_instance_id)
            .finish()
    }
}

/// Facade wiring root for engine deletion runtime and purge driver (§7).
pub struct DeletionWiring {
    runtime: Arc<DpiDeletionRuntime>,
    storage: Arc<LsmStorage>,
    ring_mode: RingMode,
}

impl DeletionWiring {
    /// Builds a new [`DeletionWiring`] instance from supplied dependencies.
    ///
    /// # Errors
    ///
    /// Returns [`ContextraError`](contextra_types::ContextraError) if runtime initialization fails.
    pub fn build(deps: DeletionWiringDeps) -> Result<Self> {
        let runtime = DpiDeletionRuntime::open(
            &deps.db_dir,
            deps.signer_cfg_path.as_deref(),
            deps.db_instance_id,
        )?;

        Ok(Self {
            runtime: Arc::new(runtime),
            storage: deps.storage,
            ring_mode: deps.ring_mode,
        })
    }

    /// Enforces key requirements for the active ring mode (§2).
    ///
    /// # Errors
    /// Returns [`ContextraError::InvalidInput`] if in Sovereign mode and `proof_key` is missing or empty.
    pub fn validate_proof_key(&self, proof_key: Option<&[u8]>) -> Result<bool> {
        match self.ring_mode {
            RingMode::Sovereign => {
                match proof_key {
                    Some(key) if key.len() >= 32 && !key.iter().all(|&b| b == 0) => Ok(true),
                    _ => Err(contextra_types::ContextraError::invalid_input(
                        "Sovereign ring requires a valid proof key (at least 32 non-zero bytes) for deletion proof generation",
                    )),
                }
            }
            RingMode::Fast => {
                tracing::warn!(
                    "Fast-ring execution active: DeletionProof generation explicitly skipped"
                );
                Ok(false)
            }
        }
    }

    /// Returns active ring mode.
    #[must_use]
    pub fn ring_mode(&self) -> RingMode {
        self.ring_mode
    }

    /// Accesses the underlying deletion runtime.
    #[must_use]
    pub fn runtime(&self) -> &Arc<DpiDeletionRuntime> {
        &self.runtime
    }

    /// Creates a new [`DpiPurgeDriver`] attached to this deletion wiring.
    #[must_use]
    pub fn purge_driver(&self) -> DpiPurgeDriver<'_> {
        DpiPurgeDriver::new(&self.storage, &self.runtime, DpiPurgeDriverConfig::default())
    }
}
