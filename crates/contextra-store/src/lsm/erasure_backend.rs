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

        // DEADLOCK & RUNTIME SAFETY ARGUMENTATION:
        // 1. Calling `purge_keys` requires executing an async task to completion from within a synchronous
        //    port trait method (`ErasureBackend::purge_physical`).
        // 2. Calling `Handle::block_on` directly while inside an active async execution context causes a
        //    Tokio runtime panic ("Cannot start a runtime from within a runtime").
        // 3. To guarantee zero panics regardless of whether the caller is outside a runtime, inside a CurrentThread
        //    runtime, or inside a MultiThread runtime async task, we spawn a dedicated OS thread via `std::thread::scope`
        //    when an outer Tokio runtime handle is present (or create a local current-thread runtime if outside Tokio).
        // 4. Deadlock safety: `ErasureBackend::purge_physical` does not acquire or hold any synchronous `parking_lot`
        //    or OS mutexes prior to spawning the worker thread. The worker thread executes `self.purge_keys`,
        //    acquiring LSM internal locks in strict lock-hierarchy order (`collections` -> `kv_locks` -> `embedder`)
        //    without holding any lock across thread boundaries or `.await` points. Thus, blocking the caller thread
        //    on thread join cannot cause lock inversion or deadlock.
        let run_purge = || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .map_err(|e| {
                    PurgeError::Io(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        e.to_string(),
                    ))
                })?;
            rt.block_on(async {
                self.purge_keys(purge_scope, as_of.inner(), Some(probe))
                    .await
            })
            .map_err(|blocked| {
                PurgeError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Purge compaction blocked: {:?}", blocked),
                ))
            })
        };

        let report = if tokio::runtime::Handle::try_current().is_ok() {
            std::thread::scope(|s| {
                s.spawn(run_purge).join().map_err(|_| {
                    PurgeError::Io(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Purge worker thread panicked",
                    ))
                })?
            })?
        } else {
            run_purge()?
        };

        Ok(PurgeOutput {
            records_purged: report.records_purged,
            bytes_reclaimed: report.bytes_reclaimed,
            sstables_replaced: report.sstables_replaced,
            obsolete_files: report.obsolete_files,
            duration_ticks: 0,
        })
    }
}
