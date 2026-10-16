// FILE-CONTEXT
// STAND: 2026-10-14
// ZWECK: Implementation of ErasureBackend trait for LsmStorage (§5.11, F-05, REG-15).
// INVARIANTEN: Zero-panic, physical scope derivation without downcasts, fail-closed errors.

use contextra_types::{ContextraError, TxId};
use contextra_ports::erasure::{
    DisposeError, ErasureBackend, FenceGuard, FreeSpaceProbe, LogicalScope, PhysicalScope, PurgeError, PurgeId, PurgeOutput,
};

use crate::compaction::purge_filter::PurgeScope;
use crate::lsm::engine::LsmStorage;
use crate::lsm::purge::dispose::dispose_obsolete_files;
use crate::tenant_codec::TenantKeyCodec;

impl ErasureBackend for LsmStorage {
    fn physical_scope(&self, scope: &LogicalScope) -> std::result::Result<PhysicalScope, ContextraError> {
        match scope {
            LogicalScope::Document {
                doc_id,
                collection_id,
                tenant_id,
            } => {
                let codec = TenantKeyCodec::new(*tenant_id);
                let col_prefix = codec.collection_prefix(collection_id);
                let doc_key = format!("t:{}:{}:doc:{}", tenant_id.0, collection_id.0, doc_id.0).into_bytes();
                PhysicalScope::try_new(*tenant_id, vec![doc_key], vec![col_prefix])
            }
            LogicalScope::Collection {
                collection_id,
                tenant_id,
            } => {
                let codec = TenantKeyCodec::new(*tenant_id);
                let col_prefix = codec.collection_prefix(collection_id);
                PhysicalScope::try_new(*tenant_id, vec![], vec![col_prefix])
            }
            LogicalScope::Tenant { tenant_id } => {
                let codec = TenantKeyCodec::new(*tenant_id);
                let tenant_prefix = codec.scan_prefix().to_vec();
                PhysicalScope::try_new(*tenant_id, vec![], vec![tenant_prefix])
            }
        }
    }

    fn acquire_fence(&self, _scope: &PhysicalScope, id: PurgeId) -> std::result::Result<FenceGuard, ContextraError> {
        self.erasure_fence.acquire_scope_fence();
        Ok(FenceGuard { purge_id: id })
    }

    fn purge_physical(
        &self,
        scope: &PhysicalScope,
        as_of: TxId,
        probe: &dyn FreeSpaceProbe,
    ) -> std::result::Result<PurgeOutput, PurgeError> {
        let purge_scope = if !scope.prefixes().is_empty() {
            PurgeScope::Prefixes(scope.prefixes().to_vec())
        } else {
            PurgeScope::Keys(scope.keys().to_vec())
        };

        let handle = tokio::runtime::Handle::try_current()
            .map_err(|e| PurgeError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

        let report = handle
            .block_on(async { self.purge_keys(purge_scope, as_of.inner(), Some(probe)).await })?;

        Ok(PurgeOutput {
            records_purged: report.records_purged,
            bytes_reclaimed: report.bytes_reclaimed,
            sstables_replaced: report.sstables_replaced,
            obsolete_files: report.obsolete_files,
            duration_ticks: 0,
        })
    }

    fn dispose(&self, files: &[std::path::PathBuf], secure_delete: bool) -> std::result::Result<(), DisposeError> {
        let storage_dir = &self.config.path;
        let obsolete_files = files.to_vec();

        let handle = tokio::runtime::Handle::try_current()
            .map_err(|e| DisposeError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

        let report = handle
            .block_on(async { dispose_obsolete_files(storage_dir, obsolete_files, secure_delete).await })
            .map_err(DisposeError::Io)?;

        if secure_delete && !report.refusals.is_empty() {
            let (path, refusal) = &report.refusals[0];
            return Err(DisposeError::ScrubRefused {
                path: path.to_string_lossy().to_string(),
                reason: format!("{:?}", refusal),
            });
        }

        Ok(())
    }

    fn snapshot_write_seq(&self, prefixes: &[Vec<u8>]) -> u64 {
        self.erasure_fence.snapshot_write_seq(prefixes)
    }
}
