// FILE-CONTEXT
// STAND: 2026-10-14
// ZWECK: Implementation of ErasureBackend trait for LsmStorage (§5.11, F-05, REG-15).
// INVARIANTEN: Zero-panic, physical scope derivation without downcasts, fail-closed errors.

use contextra_core::{ContextraError, Result, TxId};
use contextra_ports::erasure::{ErasureBackend, LogicalScope, PhysicalScope, PurgeOutput};
use contextra_ports::BoxFuture;

use crate::compaction::purge_filter::PurgeScope;
use crate::lsm::engine::LsmStorage;
use crate::tenant_codec::TenantKeyCodec;

impl ErasureBackend for LsmStorage {
    fn physical_scope(&self, scope: &LogicalScope) -> Result<PhysicalScope> {
        match scope {
            LogicalScope::Document { doc_id, tenant_id } => {
                let codec = TenantKeyCodec::new(*tenant_id);
                let prefix = codec.scan_prefix().to_vec();
                let doc_key = format!("doc:{}", doc_id.0).into_bytes();
                let mut full_key = prefix.clone();
                full_key.extend_from_slice(&doc_key);
                Ok(PhysicalScope::new(vec![full_key], vec![prefix]))
            }
            LogicalScope::Collection {
                collection_id,
                tenant_id,
            } => {
                let codec = TenantKeyCodec::new(*tenant_id);
                let col_prefix = codec.collection_prefix(collection_id);
                Ok(PhysicalScope::new(vec![], vec![col_prefix]))
            }
            LogicalScope::Keys(keys) => Ok(PhysicalScope::new(keys.clone(), vec![])),
            LogicalScope::Prefixes(prefixes) => {
                Ok(PhysicalScope::new(vec![], prefixes.clone()))
            }
        }
    }

    fn purge<'a>(
        &'a self,
        scope: &'a PhysicalScope,
        as_of: TxId,
    ) -> BoxFuture<'a, Result<PurgeOutput>> {
        Box::pin(async move {
            let purge_scope = if !scope.prefixes.is_empty() {
                PurgeScope::Prefixes(scope.prefixes.clone())
            } else {
                PurgeScope::Keys(scope.keys.clone())
            };

            let report = self
                .purge_keys(purge_scope, as_of.inner(), None)
                .await
                .map_err(|blocked| {
                    ContextraError::Storage(format!("Purge compaction blocked: {:?}", blocked))
                })?;

            Ok(PurgeOutput {
                records_purged: report.records_purged,
                bytes_reclaimed: report.bytes_reclaimed,
                sstables_replaced: report.sstables_replaced,
                obsolete_files: report.obsolete_files,
            })
        })
    }
}
