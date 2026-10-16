// FILE-CONTEXT
// STAND: 2026-10-14
// ZWECK: Implementation of ErasureBackend trait for LsmStorage (§5.11, F-05, REG-15).
// INVARIANTEN: Zero-panic, physical scope derivation without downcasts, fail-closed errors.

use std::sync::Arc;
use contextra_ports::erasure::{
    DisposeError, DisposeReport, ErasureBackend, ErasureError, FenceGuard, FreeSpaceProbe, LogicalScope, ObsoleteFile,
    PhysicalScope, PurgeError, PurgeId, PurgeOutput, ScopeOrigin,
};
use contextra_types::TxId;

use crate::compaction::purge_filter::PurgeScope;
use crate::lsm::engine::LsmStorage;
use crate::tenant_codec::TenantKeyCodec;

impl ErasureBackend for LsmStorage {
    fn physical_scope(&self, scope: &LogicalScope) -> std::result::Result<PhysicalScope, ErasureError> {
        let tenant_id = scope.tenant_id();
        let mut hasher = blake3::Hasher::new();
        hasher.update(&tenant_id.0.to_le_bytes());
        let tenant_digest: [u8; 32] = *hasher.finalize().as_bytes();

        let origin = ScopeOrigin::new(self.backend_id, tenant_digest);

        match scope {
            LogicalScope::Document {
                doc_id,
                collection_id,
                tenant_id,
            } => {
                let codec = TenantKeyCodec::new(*tenant_id);
                let col_prefix = codec.collection_prefix(collection_id);
                let doc_key = format!("t:{}:{}:doc:{}", tenant_id.0, collection_id.0, doc_id.0).into_bytes();
                PhysicalScope::try_new(*tenant_id, origin, vec![doc_key], vec![col_prefix])
            }
            LogicalScope::Collection {
                collection_id,
                tenant_id,
            } => {
                let codec = TenantKeyCodec::new(*tenant_id);
                let col_prefix = codec.collection_prefix(collection_id);
                PhysicalScope::try_new(*tenant_id, origin, vec![], vec![col_prefix])
            }
            LogicalScope::Tenant { tenant_id } => {
                let codec = TenantKeyCodec::new(*tenant_id);
                let tenant_prefix = codec.scan_prefix().to_vec();
                PhysicalScope::try_new(*tenant_id, origin, vec![], vec![tenant_prefix])
            }
        }
    }

    fn acquire_fence(&self, scope: &PhysicalScope, id: PurgeId) -> std::result::Result<FenceGuard, ErasureError> {
        if scope.origin().backend_id != self.backend_id {
            return Err(ErasureError::ForeignScope);
        }
        let prefixes = if !scope.prefixes().is_empty() {
            scope.prefixes().to_vec()
        } else if !scope.keys().is_empty() {
            scope.keys().to_vec()
        } else {
            return Err(ErasureError::MalformedScope);
        };
        let fence_prefixes = prefixes.clone();
        let fen_guard = self
            .erasure_fence
            .acquire_scope_fence_for(prefixes)
            .map_err(|_| ErasureError::Internal("Fence acquisition failed or conflict".to_string()))?;

        let fence_id = fen_guard.id();
        std::mem::forget(fen_guard);

        let fence_arc = Arc::clone(&self.erasure_fence);
        let release = Box::new(move || {
            fence_arc.release_prefix_fence(fence_id, &fence_prefixes);
        });

        Ok(FenceGuard::new(id, release))
    }

    fn purge_physical(
        &self,
        scope: &PhysicalScope,
        as_of: TxId,
        probe: &dyn FreeSpaceProbe,
        fence: &FenceGuard,
    ) -> std::result::Result<PurgeOutput, PurgeError> {
        if scope.origin().backend_id != self.backend_id {
            return Err(PurgeError::FenceConflict);
        }
        let _ = fence;

        let purge_scope = if !scope.prefixes().is_empty() {
            PurgeScope::Prefixes(scope.prefixes().to_vec())
        } else {
            PurgeScope::Keys(scope.keys().to_vec())
        };

        let handle = tokio::runtime::Handle::try_current()
            .map_err(|e| PurgeError::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))?;

        let report = tokio::task::block_in_place(|| {
            handle.block_on(async { self.purge_keys(purge_scope, as_of.inner(), Some(probe)).await })
        })?;

        Ok(PurgeOutput {
            records_purged: report.records_purged,
            bytes_reclaimed: report.bytes_reclaimed,
            sstables_replaced: report.sstables_replaced,
            obsolete_files: report.obsolete_files,
            duration_ticks: 0,
        })
    }

    fn dispose(&self, _files: &[ObsoleteFile]) -> std::result::Result<DisposeReport, DisposeError> {
        Err(DisposeError::ScrubRefused {
            file: String::new(),
            reason: "Not available in LsmStorage default backend".to_string(),
        })
    }

    fn snapshot_write_seq(&self, prefixes: &[Vec<u8>]) -> u64 {
        self.erasure_fence.snapshot_write_seq(prefixes)
    }
}
