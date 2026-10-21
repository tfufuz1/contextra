// FILE-CONTEXT
// STAND: 2026-10-21
// ZWECK: Implementation of ErasureBackend trait for LsmStorage (§5.11, F-05, REG-15).
// INVARIANTEN: Zero-panic, physical scope derivation without downcasts, fail-closed errors.

use std::sync::Arc;
use contextra_types::{ContextraError, TxId};
use contextra_ports::erasure::{
    ErasureBackend, FenceGuard, FreeSpaceProbe, LogicalScope, PhysicalScope, PurgeError, PurgeId, PurgeOutput,
};

use crate::compaction::purge_filter::PurgeScope;
use crate::lsm::engine::LsmStorage;
use crate::tenant_codec::TenantKeyCodec;

impl LsmStorage {
    /// Executes physical purge compaction asynchronously for `scope` as of `as_of`.
    pub async fn purge_physical_async(
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

        let report = self
            .purge_keys(purge_scope, as_of.inner(), Some(probe))
            .await?;

        Ok(PurgeOutput {
            records_purged: report.records_purged,
            bytes_reclaimed: report.bytes_reclaimed,
            sstables_replaced: report.sstables_replaced,
            obsolete_files: report.obsolete_files,
            duration_ticks: 0,
        })
    }
}

impl ErasureBackend for LsmStorage {
    fn physical_scope(&self, scope: &LogicalScope) -> std::result::Result<PhysicalScope, ContextraError> {
        match scope {
            LogicalScope::Document {
                doc_id,
                collection_id,
                tenant_id,
            } => {
                let doc_key = format!("t:{}:{}:doc:{}", tenant_id.0, collection_id.0, doc_id.0).into_bytes();
                PhysicalScope::try_new(*tenant_id, vec![doc_key], vec![])
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

    fn acquire_fence(&self, scope: &PhysicalScope, id: PurgeId) -> std::result::Result<FenceGuard, ContextraError> {
        let mut fence_targets = Vec::new();
        fence_targets.extend_from_slice(scope.prefixes());
        fence_targets.extend_from_slice(scope.keys());

        if fence_targets.is_empty() {
            return Err(ContextraError::InvalidInput(
                "Cannot acquire fence for empty scope".to_string(),
            ));
        }

        let fen_guard = self
            .erasure_fence
            .acquire_scope_fence_for(fence_targets)
            .map_err(|e| ContextraError::InvalidInput(format!("Fence acquisition failed: {:?}", e)))?;

        let fence_id = fen_guard.id();
        std::mem::forget(fen_guard);

        let fence = Arc::clone(&self.erasure_fence);
        Ok(FenceGuard::with_release(id, move || {
            fence.release_prefix_fence(fence_id);
        }))
    }

    fn purge_physical(
        &self,
        scope: &PhysicalScope,
        as_of: TxId,
        probe: &dyn FreeSpaceProbe,
    ) -> std::result::Result<PurgeOutput, PurgeError> {
        if tokio::runtime::Handle::try_current().is_ok() {
            return Err(PurgeError::Write {
                file: "purge_physical".to_string(),
                source_details: "ErasureBackend::purge_physical called synchronously from within an active Tokio runtime context; use LsmStorage::purge_physical_async instead".to_string(),
            });
        }

        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| PurgeError::Write {
                file: "purge_physical".to_string(),
                source_details: e.to_string(),
            })?;

        rt.block_on(self.purge_physical_async(scope, as_of, probe))
    }
}
