// FILE-CONTEXT
// STAND: 2026-10-14
// ZWECK: Implementation of ErasureBackend trait for LsmStorage (§5.11, F-05, REG-15).
// INVARIANTEN: Zero-panic, physical scope derivation without downcasts, fail-closed errors.

use contextra_ports::erasure::{
    ErasureBackend, FenceGuard, FreeSpaceProbe, LogicalScope, PhysicalScope, PurgeError, PurgeId,
    PurgeOutput,
};
use contextra_types::{ContextraError, TxId};

use crate::compaction::purge_filter::PurgeScope;
use crate::lsm::engine::LsmStorage;
use crate::tenant_codec::TenantKeyCodec;

impl ErasureBackend for LsmStorage {
    fn physical_scope(
        &self,
        scope: &LogicalScope,
    ) -> std::result::Result<PhysicalScope, ContextraError> {
        match scope {
            LogicalScope::Document {
                doc_id,
                collection_id,
                tenant_id,
            } => {
                let codec = TenantKeyCodec::new(*tenant_id);
                let col_prefix = codec.collection_prefix(collection_id);
                let doc_key =
                    format!("t:{}:{}:doc:{}", tenant_id.0, collection_id.0, doc_id.0).into_bytes();
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

    fn acquire_fence(
        &self,
        _scope: &PhysicalScope,
        id: PurgeId,
    ) -> std::result::Result<FenceGuard, ContextraError> {
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

        let handle = tokio::runtime::Handle::try_current().map_err(|e| {
            PurgeError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?;

        let report = handle
            .block_on(async {
                self.purge_keys(purge_scope, as_of.inner(), Some(probe))
                    .await
            })
            .map_err(|blocked| {
                PurgeError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Purge compaction blocked: {:?}", blocked),
                ))
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
