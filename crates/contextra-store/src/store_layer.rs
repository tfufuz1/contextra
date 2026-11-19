//! Erasure-chain adapter for the LSM and WAL persistence layer.
//!
//! This adapter deliberately owns no key material and never decrypts records.
//! It translates the coordinator's identity targets into the store's physical
//! tenant key prefixes, establishes the in-memory fence, runs the idempotent
//! rewrite, and performs an independent raw-file witness scan.

use crate::lsm::LsmStorage;
use contextra_erasure::chain::{
    EraseTarget, ErasureLayer, LayerFuture, LayerScope, PurgeCx, PurgeOutcome,
};
use contextra_erasure::error::ErasureError;
use contextra_erasure::types_local::{Armed, KeyCoverage, LayerId, Scanner};
use parking_lot::Mutex;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Stable ID for the LSM/WAL storage erasure layer.
const STORE_LAYER_ID: LayerId = LayerId(*b"contextra-store!");

/// Adapter implementing the erasure chain's S1/S1b responsibilities.
///
/// `LayerScope` intentionally carries only an opaque identifier. Prefixes are
/// retained in this in-process registry, so neither journals nor diagnostics
/// receive cleartext target keys.
pub struct StoreLayer {
    storage: Arc<LsmStorage>,
    scopes: Mutex<BTreeMap<[u8; 16], Vec<u8>>>,
}

impl StoreLayer {
    /// Binds an LSM instance to the erasure chain.
    pub fn new(storage: Arc<LsmStorage>) -> Self {
        Self {
            storage,
            scopes: Mutex::new(BTreeMap::new()),
        }
    }

    fn prefix_for(target: &EraseTarget) -> Result<Vec<u8>, ErasureError> {
        match target {
            EraseTarget::Document {
                tenant,
                collection,
                doc,
            } => Ok(format!("t:{}:{}:chunk:{doc}", tenant.inner(), collection.0).into_bytes()),
            EraseTarget::Collection { tenant, collection } => {
                Ok(format!("t:{}:{}:", tenant.inner(), collection.0).into_bytes())
            }
            EraseTarget::Tenant { tenant } => Ok(format!("t:{}:", tenant.inner()).into_bytes()),
            // P-record keys are deliberately opaque to Store. A subject purge
            // is resolved by db into the exact document keys after crypto
            // shredding; guessing a prefix here could erase another subject.
            EraseTarget::Subject { .. } => Err(ErasureError::Storage(
                "subject target requires db-resolved document scope".into(),
            )),
        }
    }

    fn scope_id(prefix: &[u8]) -> [u8; 16] {
        let digest = blake3::hash(prefix);
        let mut id = [0u8; 16];
        id.copy_from_slice(&digest.as_bytes()[..16]);
        id
    }

    fn prefix_for_scope(&self, scope: &LayerScope) -> Result<Vec<u8>, ErasureError> {
        self.scopes
            .lock()
            .get(&scope.scope_id)
            .cloned()
            .ok_or_else(|| ErasureError::Storage("unknown store erase scope".into()))
    }

    fn files_below(root: &Path) -> Result<Vec<PathBuf>, ErasureError> {
        let mut result = Vec::new();
        let mut pending = vec![root.to_path_buf()];
        while let Some(dir) = pending.pop() {
            for entry in std::fs::read_dir(&dir)? {
                let entry = entry?;
                let ty = entry.file_type()?;
                if ty.is_dir() {
                    pending.push(entry.path());
                } else if ty.is_file() {
                    result.push(entry.path());
                }
            }
        }
        result.sort();
        Ok(result)
    }

    /// Performs the store-owned, raw-byte part of a witness measurement.
    ///
    /// Completing a [`Scanner`] is deliberately left to the chain coordinator:
    /// the `ErasureLayer::witness` contract receives it by mutable reference so
    /// every participating layer can account for its files before the
    /// coordinator mints the final witness.  In particular, this scanner must
    /// never be consumed here.
    fn raw_witness(&self, prefix: &[u8], scanner: &mut Scanner<Armed>) -> Result<u64, ErasureError> {
        let mut matches = 0_u64;
        for path in Self::files_below(self.storage.data_path())? {
            let bytes = std::fs::read(path)?;
            let byte_count = u64::try_from(bytes.len()).map_err(|_| {
                ErasureError::Storage("store witness file length exceeds u64".into())
            })?;
            let file_matches = u64::try_from(
                bytes
                    .windows(prefix.len())
                    .filter(|window| *window == prefix)
                    .count(),
            )
            .map_err(|_| ErasureError::Storage("store witness match count exceeds u64".into()))?;
            scanner.record(1, byte_count);
            matches = matches.saturating_add(file_matches);
        }
        Ok(matches)
    }
}

impl ErasureLayer for StoreLayer {
    fn id(&self) -> LayerId {
        STORE_LAYER_ID
    }

    fn key_coverage(&self) -> KeyCoverage {
        // Individual record encryption is owned by db/crypto; Store only owns
        // ciphertext persistence and physical residue removal.
        KeyCoverage::None
    }

    fn resolve<'a>(&'a self, target: &'a EraseTarget) -> LayerFuture<'a, LayerScope> {
        Box::pin(async move {
            let prefix = Self::prefix_for(target)?;
            let scope_id = Self::scope_id(&prefix);
            self.scopes.lock().insert(scope_id, prefix);
            Ok(LayerScope {
                scope_id,
                affected_units: Vec::new(),
            })
        })
    }

    fn on_gate<'a>(&'a self, _scope: &'a LayerScope) -> LayerFuture<'a, ()> {
        Box::pin(async move {
            // Cache data must not outlive a gate. The actual ReadGate is owned
            // by db as the single read-path check site.
            self.storage.block_cache().clear();
            Ok(())
        })
    }

    fn purge<'a>(
        &'a self,
        scope: &'a LayerScope,
        _cx: &'a PurgeCx,
    ) -> LayerFuture<'a, PurgeOutcome> {
        Box::pin(async move {
            let prefix = self.prefix_for_scope(scope)?;
            let report = self
                .storage
                .purge_prefix(&prefix)
                .await
                .map_err(|error| ErasureError::Storage(error.to_string()))?;
            Ok(PurgeOutcome {
                purged_units: report.purged_records,
            })
        })
    }

    fn witness<'a>(
        &'a self,
        scope: &'a LayerScope,
        scanner: &'a mut Scanner<Armed>,
    ) -> LayerFuture<'a, u64> {
        Box::pin(async move {
            let prefix = self.prefix_for_scope(scope)?;
            self.raw_witness(&prefix, scanner)
        })
    }
}
