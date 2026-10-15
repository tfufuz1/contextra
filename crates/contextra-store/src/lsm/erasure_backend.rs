// FILE-CONTEXT
// STAND: 2026-10-15
// ZWECK: Implementation of ErasureBackend trait for LsmStorage (§5.11, F-05, REG-15).
// INVARIANTEN: Zero-panic, physical scope derivation without downcasts, fail-closed errors.

use contextra_core::{Result, TxId};
use contextra_ports::erasure::{
    ErasureBackend, FenceGuard, FreeSpaceProbe, LogicalScope, PhysicalScope, PurgeError, PurgeId, PurgeOutput,
};

use crate::compaction::purge_filter::PurgeScope;
use crate::lsm::engine::LsmStorage;
use crate::tenant_codec::TenantKeyCodec;

impl ErasureBackend for LsmStorage {
    fn physical_scope(&self, scope: &LogicalScope) -> Result<PhysicalScope> {
        match scope {
            LogicalScope::Document {
                doc_id,
                tenant_id,
                collection_id,
            } => {
                let codec = TenantKeyCodec::new(*tenant_id);
                let col_prefix = codec.collection_prefix(collection_id);
                let doc_key = codec.encode_chunk_key(collection_id, *doc_id);
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
                let prefix = codec.scan_prefix().to_vec();
                PhysicalScope::try_new(*tenant_id, vec![], vec![prefix])
            }
        }
    }

    fn acquire_fence(&self, _scope: &PhysicalScope, id: PurgeId) -> Result<FenceGuard> {
        Ok(FenceGuard { purge_id: id })
    }

    fn purge_physical(
        &self,
        scope: &PhysicalScope,
        as_of: TxId,
        _probe: &dyn FreeSpaceProbe,
    ) -> std::result::Result<PurgeOutput, PurgeError> {
        let purge_scope = if !scope.prefixes().is_empty() {
            PurgeScope::Prefixes(scope.prefixes().to_vec())
        } else {
            PurgeScope::Keys(scope.keys().to_vec())
        };

        let report = match tokio::runtime::Handle::try_current() {
            Ok(handle) => tokio::task::block_in_place(|| {
                handle.block_on(async {
                    self.purge_keys(purge_scope, as_of.inner(), None).await
                })
            }),
            Err(_) => {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .build()
                    .map_err(|e| PurgeError::Manifest(format!("Failed to build runtime: {}", e)))?;
                rt.block_on(async {
                    self.purge_keys(purge_scope, as_of.inner(), None).await
                })
            }
        }
        .map_err(|blocked| {
            PurgeError::Manifest(format!("Purge compaction blocked: {:?}", blocked))
        })?;

        Ok(PurgeOutput {
            records_purged: report.records_purged,
            bytes_reclaimed: report.bytes_reclaimed,
            sstables_replaced: report.sstables_replaced,
            obsolete_files: report.obsolete_files,
            duration_ticks: 0,
        })
    }
}
